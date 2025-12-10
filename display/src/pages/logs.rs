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

use crate::header_style;

use log::LevelFilter;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
};
use tui_logger::{TuiLoggerWidget, TuiWidgetEvent, TuiWidgetState};

pub(crate) struct Logs {
    state: TuiWidgetState,
    min_level: LevelFilter,
}

impl Logs {
    pub(crate) fn new() -> Self {
        Self { state: TuiWidgetState::new(), min_level: LevelFilter::Trace }
    }

    pub(crate) fn draw(&mut self, f: &mut Frame, area: Rect) {
        // Split area: logs widget takes most space, status bar at bottom
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(3), Constraint::Length(1)])
            .split(area);

        // Render the logs widget
        let logger_widget = TuiLoggerWidget::default()
            .block(Block::default().borders(Borders::ALL).style(header_style()).title("Logs"))
            .style_error(Style::default().fg(Color::Red))
            .style_warn(Style::default().fg(Color::Yellow))
            .style_info(Style::default().fg(Color::Cyan))
            .style_debug(Style::default().fg(Color::Green))
            .style_trace(Style::default().fg(Color::Gray))
            .output_file(false)
            .output_line(false)
            .output_target(false)
            .output_timestamp(None) // Disable tui_logger's timestamp since messages already have one
            .state(&self.state);
        f.render_widget(logger_widget, chunks[0]);

        // Check if we're in live mode or scrolling
        let is_live = self.state.inner.lock().opt_timestamp_bottom.is_none();
        let mode_text = if is_live { "LIVE" } else { "SCROLLING" };
        let mode_style = if is_live {
            Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)
        };

        let level_text = format!("{:?}", self.min_level);
        let level_style = match self.min_level {
            LevelFilter::Error => Style::default().fg(Color::Red),
            LevelFilter::Warn => Style::default().fg(Color::Yellow),
            LevelFilter::Info => Style::default().fg(Color::Cyan),
            LevelFilter::Debug => Style::default().fg(Color::Green),
            LevelFilter::Trace => Style::default().fg(Color::Gray),
            LevelFilter::Off => Style::default().fg(Color::DarkGray),
        };

        let status_line = Line::from(vec![
            Span::raw(" Mode: "),
            Span::styled(mode_text, mode_style),
            Span::raw(" | Level: ≥"),
            Span::styled(level_text, level_style),
            Span::raw(" (+/- to change)"),
        ]);
        let status = Paragraph::new(status_line);
        f.render_widget(status, chunks[1]);
    }

    /// Handle key events for scrolling logs
    /// Note: tui_logger only supports page-based scrolling, not line-by-line
    pub(crate) fn on_up(&mut self) {
        self.state.transition(TuiWidgetEvent::PrevPageKey);
    }

    pub(crate) fn on_down(&mut self) {
        self.state.transition(TuiWidgetEvent::NextPageKey);
    }

    pub(crate) fn on_page_up(&mut self) {
        self.state.transition(TuiWidgetEvent::PrevPageKey);
    }

    pub(crate) fn on_page_down(&mut self) {
        self.state.transition(TuiWidgetEvent::NextPageKey);
    }

    pub(crate) fn on_escape(&mut self) {
        // Return to bottom of logs (live mode)
        self.state.transition(TuiWidgetEvent::EscapeKey);
    }

    pub(crate) fn on_plus(&mut self) {
        // Increase verbosity (show more log levels)
        self.min_level = match self.min_level {
            LevelFilter::Off => LevelFilter::Error,
            LevelFilter::Error => LevelFilter::Warn,
            LevelFilter::Warn => LevelFilter::Info,
            LevelFilter::Info => LevelFilter::Debug,
            LevelFilter::Debug => LevelFilter::Trace,
            LevelFilter::Trace => LevelFilter::Trace,
        };
        self.state = TuiWidgetState::new().set_default_display_level(self.min_level);
    }

    pub(crate) fn on_minus(&mut self) {
        // Decrease verbosity (show fewer log levels)
        self.min_level = match self.min_level {
            LevelFilter::Off => LevelFilter::Off,
            LevelFilter::Error => LevelFilter::Off,
            LevelFilter::Warn => LevelFilter::Error,
            LevelFilter::Info => LevelFilter::Warn,
            LevelFilter::Debug => LevelFilter::Info,
            LevelFilter::Trace => LevelFilter::Debug,
        };
        self.state = TuiWidgetState::new().set_default_display_level(self.min_level);
    }
}
