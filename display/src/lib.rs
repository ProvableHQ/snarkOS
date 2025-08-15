// Copyright (c) 2019-2025 Provable Inc.
// This file is part of the snarkOS library.

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at:

// http://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![forbid(unsafe_code)]

mod pages;
use pages::*;

mod tabs;
use tabs::Tabs;

use snarkos_node::{Node, network::PeerPoolHandling};
use snarkos_utilities::Stoppable;

use snarkvm::prelude::Network;

use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame,
    Terminal,
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Tabs as TabsTui},
};
use std::{
    collections::HashMap,
    io,
    net::IpAddr,
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use tokio::sync::mpsc::Receiver;

#[derive(Clone, Debug)]
pub struct PeerStats {
    timestamp: Instant,
    received_bytes: u64,
    sent_bytes: u64,
}

pub struct Display<N: Network> {
    /// An instance of the node.
    node: Node<N>,
    /// The tick rate of the display.
    tick_rate: Duration,
    /// The state of the tabs.
    tabs: Tabs,
    /// The overview tab.
    overview: Overview,
    /// The logs tab.
    logs: Logs,
    /// The metrics tab.
    io_metrics: IoMetrics,
    /// The sync metrics tab.
    sync_metrics: SyncMetrics,
    /// Previous peer statistics for calculating instantaneous speeds.
    previous_peer_stats: HashMap<IpAddr, PeerStats>,
}

fn header_style() -> Style {
    Style::default().fg(Color::Cyan)
}

fn content_style() -> Style {
    Style::default().fg(Color::White)
}

impl<N: Network> Display<N> {
    /// Initializes a new display.
    pub fn start(node: Node<N>, log_receiver: Receiver<Vec<u8>>, stoppable: Arc<dyn Stoppable>) -> Result<()> {
        // Initialize the display.
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Initialize the display.
        let mut display = Self {
            node,
            tick_rate: Duration::from_secs(1),
            tabs: Tabs::new(PAGES.to_vec()),
            overview: Overview::new(),
            logs: Logs::new(log_receiver),
            io_metrics: IoMetrics::new(),
            sync_metrics: SyncMetrics::new(),
            previous_peer_stats: HashMap::new(),
        };

        // Render the display.
        let res = display.render(&mut terminal, stoppable);

        // Terminate the display.
        disable_raw_mode()?;
        execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
        terminal.show_cursor()?;

        // Exit.
        if let Err(err) = res {
            println!("{err:?}")
        }

        Ok(())
    }
}

impl<N: Network> Display<N> {
    /// Renders the display.
    fn render<B: Backend>(&mut self, terminal: &mut Terminal<B>, stoppable: Arc<dyn Stoppable>) -> io::Result<()> {
        let mut last_tick = Instant::now();
        loop {
            terminal.draw(|f| self.draw(f))?;

            // Set the timeout duration.
            let timeout = self.tick_rate.checked_sub(last_tick.elapsed()).unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout)? {
                if let Event::Key(key) = event::read()? {
                    match key.code {
                        KeyCode::Esc => {
                            stoppable.stop();
                            return Ok(());
                        }
                        KeyCode::Left => self.tabs.previous(),
                        KeyCode::Right => self.tabs.next(),
                        KeyCode::Up => {
                            match self.tabs.index {
                                0 => {
                                    // Scroll up in peer table for overview tab
                                    self.overview.scroll_up();
                                }
                                1 => self.io_metrics.scale_up(),
                                2 => self.sync_metrics.scale_up(),
                                _ => {}
                            }
                        }
                        KeyCode::Down => {
                            match self.tabs.index {
                                0 => {
                                    // Scroll down in peer table for overview tab
                                    let peer_count =
                                        self.node.router().map(|r| r.get_peers().len()).unwrap_or_default();
                                    self.overview.scroll_down(peer_count);
                                }
                                1 => self.io_metrics.scale_down(),
                                2 => self.sync_metrics.scale_down(),
                                _ => {}
                            }
                        }
                        KeyCode::Char('j') => {
                            // Alternative down scrolling for peer table (overview tab only)
                            if self.tabs.index == 0 {
                                let peer_count = self.node.router().map(|r| r.get_peers().len()).unwrap_or_default();
                                self.overview.scroll_down(peer_count);
                            }
                        }
                        KeyCode::Char('k') => {
                            // Alternative up scrolling for peer table (overview tab only)
                            if self.tabs.index == 0 {
                                self.overview.scroll_up();
                            }
                        }
                        _ => {}
                    }
                }
            }

            if last_tick.elapsed() >= self.tick_rate {
                thread::sleep(Duration::from_millis(50));
                last_tick = Instant::now();
            }
        }
    }

    /// Draws the display.
    fn draw(&mut self, f: &mut Frame) {
        /* Layout */

        // Initialize the layout of the page.
        let chunks = Layout::default()
            .margin(1)
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(1)].as_ref())
            .split(f.area());

        /* Tabs */

        // Initialize the tabs.
        let block = Block::default().style(Style::default().bg(Color::Black).fg(Color::White));
        f.render_widget(block, f.area());
        let titles: Vec<_> = self
            .tabs
            .titles
            .iter()
            .map(|t| {
                let (first, rest) = t.split_at(1);
                Line::from(vec![
                    Span::styled(first, Style::default().fg(Color::Yellow)),
                    Span::styled(rest, Style::default().fg(Color::Green)),
                ])
            })
            .collect();
        let tabs = TabsTui::new(titles)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Welcome to Aleo.")
                    .style(Style::default().add_modifier(Modifier::BOLD)),
            )
            .select(self.tabs.index)
            .style(header_style())
            .highlight_style(Style::default().add_modifier(Modifier::BOLD).bg(Color::White));
        f.render_widget(tabs, chunks[0]);

        /* Pages */

        // Update sync metrics data regardless of which tab is selected
        self.sync_metrics.update_data(&self.node);

        // Initialize the page.
        match self.tabs.index {
            0 => self.overview.draw(f, chunks[1], &self.node, &mut self.previous_peer_stats),
            1 => self.io_metrics.draw(f, chunks[1], &self.node),
            2 => self.sync_metrics.draw(f, chunks[1], &self.node),
            3 => self.logs.draw(f, chunks[1]),
            _ => unreachable!(),
        };

        let help_text = if self.tabs.index == 0 {
            "Use ← → to switch tabs, ↑ ↓ j k to scroll peer table, ESC to quit"
        } else {
            "Use ← → to switch tabs, ↑ ↓ to scale metrics, ESC to quit"
        };
        let help = Paragraph::new(help_text).style(content_style());
        f.render_widget(help, chunks[2]);
    }
}
