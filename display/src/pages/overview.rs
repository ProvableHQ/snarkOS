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

use crate::{PeerStats, content_style, header_style};

use snarkos_node::{
    Node,
    router::{Peer, PeerPoolHandling, messages::NodeType},
    tcp::P2P,
};

use snarkvm::console::network::Network;

use std::{
    collections::HashMap,
    net::IpAddr,
    time::{Duration, Instant},
};

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, Row, Table},
};

pub(crate) struct Overview;

impl Overview {
    /// Formats bytes per second with appropriate units (bit/s, KB/s, or Mbit/s)
    fn format_traffic_rate(bytes_per_second: f64) -> String {
        if bytes_per_second < 1024.0 {
            format!("{bytes_per_second:.1} B/s")
        } else if bytes_per_second < (1024.0 * 1024.0) {
            format!("{:.1} KB/s", bytes_per_second / 1024.0)
        } else {
            format!("{:.1} MB/s", bytes_per_second / (1024.0 * 1024.0))
        }
    }

    /// Formats total bytes with appropriate units (B, KB, MB, GB)
    fn format_total_bytes(bytes: u64) -> String {
        if bytes < 1024 {
            format!("{bytes} B")
        } else if bytes < 1024 * 1024 {
            format!("{:.1} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }

    fn draw_latest_block<N: Network>(&self, f: &mut Frame, area: Rect, node: &Node<N>) {
        let text = if let Some(ledger) = node.ledger() {
            let block = ledger.latest_block();
            Text::raw(format!("Hash: {} | Height: {}", block.hash(), block.height()))
        } else {
            Text::raw("N/A")
        };

        let paragraph = Paragraph::new(text)
            .style(content_style())
            .block(Block::default().borders(Borders::ALL).style(header_style()).title("Latest Block"));
        f.render_widget(&paragraph, area);
    }

    fn draw_node_info<N: Network>(&self, f: &mut Frame, area: Rect, node: &Node<N>) {
        let node_type_str = match node.node_type() {
            NodeType::Validator => "Validator",
            NodeType::Prover => "Prover",
            NodeType::Client => "Client",
        };

        let node_address = format!("{}", node.router().address());

        let text = Text::raw(format!("Type: {node_type_str} | Address: {node_address}"));

        let paragraph = Paragraph::new(text)
            .style(content_style())
            .block(Block::default().borders(Borders::ALL).style(header_style()).title("Node Info"));
        f.render_widget(&paragraph, area);
    }

    /// Draws a table containing all connected and connecting peers.
    fn draw_peer_table<N: Network>(
        &self,
        f: &mut Frame,
        area: Rect,
        node: &Node<N>,
        previous_peer_stats: &mut HashMap<IpAddr, PeerStats>,
    ) {
        let header = [
            "Network Address",
            "Aleo Address",
            "State",
            "Node Type",
            "↓ Speed",
            "↑ Speed",
            "↓ Total",
            "↑ Total",
            "Last Seen",
        ];
        let constraints = [
            Constraint::Length(20),
            Constraint::Min(30),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(12),
        ];

        let rows: Vec<_> = node
            .router()
            .get_peers()
            .into_iter()
            .filter(|peer| !peer.is_candidate()) // Too many candidate peers for overview.
            .map(|peer| {
                let state = match peer {
                    Peer::Candidate(_) => "candidate",
                    Peer::Connecting(_) => "connecting",
                    Peer::Connected(_) => "connected",
                }.to_string();

                let node_type = if let Some(node_type) = peer.node_type() {
                    node_type.to_string()
                } else {
                    "unknown".to_string()
                };

                let (aleo_address, last_seen) = match &peer {
                    Peer::Connected(p) => (
                        format!("{}", p.aleo_addr),
                        format!("{:.1}s ago", p.last_seen.elapsed().as_secs_f64())
                    ),
                    _ => ("N/A".to_string(), "N/A".to_string()),
                };

                // Get traffic statistics from TCP layer
                let (download_speed, upload_speed, download_total, upload_total) = if let Some(stats) = node.router().tcp().known_peers().get(peer.listener_addr().ip()) {
                    let now = Instant::now();
                    let (_, received_bytes) = stats.received();
                    let (_, sent_bytes) = stats.sent();

                    // Calculate instantaneous speeds using previous measurements
                    let (download_speed_str, upload_speed_str) = if let Some(prev_stats) = previous_peer_stats.get(&peer.listener_addr().ip()) {
                        let time_diff = now.duration_since(prev_stats.timestamp).as_secs_f64().max(0.1);
                        let received_diff = received_bytes.saturating_sub(prev_stats.received_bytes) as f64;
                        let sent_diff = sent_bytes.saturating_sub(prev_stats.sent_bytes) as f64;

                        let download_speed = received_diff / time_diff;
                        let upload_speed = sent_diff / time_diff;

                        (Self::format_traffic_rate(download_speed), Self::format_traffic_rate(upload_speed))
                    } else {
                        ("N/A".to_string(), "N/A".to_string())
                    };

                    // Update previous stats for next calculation
                    previous_peer_stats.insert(peer.listener_addr().ip(), PeerStats {
                        timestamp: now,
                        received_bytes,
                        sent_bytes,
                    });

                    (download_speed_str, upload_speed_str, Self::format_total_bytes(received_bytes), Self::format_total_bytes(sent_bytes))
                } else {
                    ("N/A".to_string(), "N/A".to_string(), "N/A".to_string(), "N/A".to_string())
                };

                Row::new([
                    format!("{}", peer.listener_addr()),
                    aleo_address,
                    state,
                    node_type,
                    download_speed,
                    upload_speed,
                    download_total,
                    upload_total,
                    last_seen
                ]).style(content_style())
            })
            .collect();

        let peer_table = Table::new(rows, constraints)
            .style(content_style())
            .header(Row::new(header).style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)))
            .block(Block::default().borders(Borders::ALL).style(header_style()).title("Peers"));

        f.render_widget(peer_table, area);
    }

    fn draw_sync_info<N: Network>(&self, f: &mut Frame, area: Rect, node: &Node<N>) {
        let is_synced = node.is_block_synced();
        let blocks_behind = node.num_blocks_behind();
        let blocks_behind_str = if let Some(num) = blocks_behind { num.to_string() } else { "?".to_string() };
        let current_speed = node.get_sync_speed();

        // Estimate time remaining
        let eta_text = if let Some(blocks_behind) = blocks_behind
            && blocks_behind > 0
            && current_speed > 0.0
        {
            let seconds_remaining = blocks_behind as f64 / current_speed;
            let duration = Duration::from_secs_f64(seconds_remaining);
            let hours = duration.as_secs() / 3600;
            let minutes = (duration.as_secs() % 3600) / 60;
            if hours > 0 { format!("| ETA: {hours}h {minutes}m") } else { format!("ETA: {minutes}m") }
        } else if !is_synced {
            "| ETA: ?".to_string()
        } else {
            "".to_string()
        };

        let sync_status = if is_synced { "✓ Synced" } else { "⚠  Syncing" };

        let text = Text::raw(format!(
            "Status: {sync_status} | Blocks Behind: {blocks_behind_str} | \
             Current Speed: {current_speed:.1} blocks/s {eta_text}",
        ));

        let paragraph = Paragraph::new(text)
            .style(content_style())
            .block(Block::default().borders(Borders::ALL).style(header_style()).title("Sync Status"));

        f.render_widget(paragraph, area);
    }

    pub(crate) fn draw<N: Network>(
        &self,
        f: &mut Frame,
        area: Rect,
        node: &Node<N>,
        previous_peer_stats: &mut HashMap<IpAddr, PeerStats>,
    ) {
        // Initialize the layout of the page.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Node Info
                Constraint::Length(3), // Latest Block
                Constraint::Length(3), // Sync Status
                Constraint::Min(8),    // Peers table
            ])
            .split(area);

        self.draw_node_info(f, chunks[0], node);
        self.draw_latest_block(f, chunks[1], node);
        self.draw_sync_info(f, chunks[2], node);
        self.draw_peer_table(f, chunks[3], node, previous_peer_stats);
    }
}
