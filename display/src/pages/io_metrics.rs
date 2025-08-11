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

use crate::{content_style, header_style};

use snarkos_node::Node;
use snarkos_node_tcp::P2P;
use snarkvm::prelude::Network;

use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::Line,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, LegendPosition, Paragraph},
};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    hash::{DefaultHasher, Hash, Hasher},
    net::SocketAddr,
    time::Instant,
};

struct PeerStats {
    last_update: Instant,
    sent_bytes: u64,
    received_bytes: u64,
}

#[derive(Clone, Debug)]
pub struct PeerSpeedData {
    pub listener_addr: SocketAddr,
    pub download_speed: f64, // bytes per second
    pub upload_speed: f64,   // bytes per second
}

#[derive(Clone, Debug)]
pub struct MetricDataPoint {
    pub timestamp: f64,
    pub peer_speeds: Vec<PeerSpeedData>,
}

pub(crate) struct IoMetrics {
    data_points: VecDeque<MetricDataPoint>,
    previous_peer_stats: HashMap<SocketAddr, PeerStats>,
    start_time: Instant,
    max_data_points: usize,
}

impl IoMetrics {
    pub fn new() -> Self {
        Self {
            data_points: VecDeque::new(),
            previous_peer_stats: HashMap::new(),
            start_time: Instant::now(),
            max_data_points: 300, // Keep 300 data points (5 minute(s) of history)
        }
    }

    /// Returns a consistent color index for a given IP address
    fn get_peer_color_index(listener_addr: &SocketAddr) -> usize {
        let mut hasher = DefaultHasher::new();
        listener_addr.hash(&mut hasher);
        hasher.finish() as usize
    }

    pub fn update_data<N: Network>(&mut self, node: &Node<N>) {
        let elapsed_secs = self.start_time.elapsed().as_secs_f64();

        // Check that at least a second elapsed
        if let Some(back) = self.data_points.back()
            && (back.timestamp + 1.0) > elapsed_secs
        {
            return;
        }

        let now = Instant::now();

        // Collect peer speed data
        let mut peer_speeds = Vec::new();
        for peer in node.router().get_connected_peers() {
            let listener_addr = peer.listener_addr;

            if let Some(stats) = node.router().tcp().known_peers().get(listener_addr.ip()) {
                let (_, current_received_bytes) = stats.received();
                let (_, current_sent_bytes) = stats.sent();

                // Calculate download speed if we have previous data
                if let Some(last_stats) = self.previous_peer_stats.get(&listener_addr) {
                    let time_diff = now.duration_since(last_stats.last_update).as_secs_f64().max(0.1);
                    let down_bytes_diff = current_received_bytes.saturating_sub(last_stats.received_bytes) as f64;
                    let download_speed = down_bytes_diff / time_diff;
                    let up_bytes_diff = current_sent_bytes.saturating_sub(last_stats.sent_bytes) as f64;
                    let upload_speed = up_bytes_diff / time_diff;

                    peer_speeds.push(PeerSpeedData { listener_addr, download_speed, upload_speed });
                }

                // Update previous stats for next calculation
                self.previous_peer_stats.insert(listener_addr, PeerStats {
                    last_update: now,
                    received_bytes: current_received_bytes,
                    sent_bytes: current_sent_bytes,
                });
            }
        }

        let data_point = MetricDataPoint { timestamp: elapsed_secs, peer_speeds };

        self.data_points.push_back(data_point);

        // Keep only the last max_data_points
        while self.data_points.len() > self.max_data_points {
            self.data_points.pop_front();
        }
    }

    pub(crate) fn draw<N: Network>(&mut self, f: &mut Frame, area: Rect, node: &Node<N>) {
        // Update data before drawing
        self.update_data(node);

        // Initialize the layout of the page.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10), // Network metrics chart
                Constraint::Min(10), // Block height chart
            ])
            .split(area);

        self.draw_download_speed_chart(f, chunks[0]);
        self.draw_upload_speed_chart(f, chunks[1]);
    }

    /// Generic helper function to draw peer speed charts (download or upload).
    fn draw_peer_speed_chart<F>(&self, f: &mut Frame, area: Rect, title: &str, speed_extractor: F)
    where
        F: Fn(&PeerSpeedData) -> f64,
    {
        let block = Block::default().borders(Borders::ALL).style(header_style()).title(title);

        // Collect all unique peer IPs across all data points.
        // Use a BTreeSet, so that lines are always drawn in the same order.
        let mut all_peers: BTreeSet<_> = Default::default();
        for data_point in &self.data_points {
            for peer_speed in &data_point.peer_speeds {
                all_peers.insert(peer_speed.listener_addr);
            }
        }

        if all_peers.is_empty() {
            let placeholder = Paragraph::new("No traffic data available...").style(content_style()).block(block);
            f.render_widget(placeholder, area);
            return;
        }

        // Define colors for different peers
        let colors = [
            Color::Green,
            Color::Blue,
            Color::Red,
            Color::Yellow,
            Color::Magenta,
            Color::Cyan,
            Color::White,
            Color::LightGreen,
        ];

        // Collect data for all peers first
        let peer_data_map: BTreeMap<_, _> = all_peers
            .iter()
            .map(|listener_addr| {
                let peer_data: Vec<(f64, f64)> = self
                    .data_points
                    .iter()
                    .filter_map(|data_point| {
                        // Find this peer's speed in this data point
                        data_point
                            .peer_speeds
                            .iter()
                            .find(|ps| &ps.listener_addr == listener_addr)
                            .map(|ps| (data_point.timestamp, speed_extractor(ps) / 1024.0)) // Convert to KB/s
                    })
                    .collect();

                // Show legend for each connected peer, even if there is no data.
                (listener_addr, peer_data)
            })
            .collect();

        // Create datasets for each peer
        let datasets: Vec<_> = peer_data_map
            .iter()
            .map(|(listener_addr, peer_data)| {
                // Pick color based on listener_addr so it does not change
                // when new peers are added.
                let color_index = Self::get_peer_color_index(listener_addr);
                let color = colors[color_index % colors.len()];

                Dataset::default()
                    .name(listener_addr.to_string())
                    .marker(symbols::Marker::Dot)
                    .style(Style::default().fg(color))
                    .graph_type(GraphType::Line)
                    .data(peer_data)
            })
            .collect();

        // If no datasets, show a message indicating no peer data
        if datasets.is_empty() {
            let placeholder = Paragraph::new("No traffic data available...").style(content_style()).block(block);
            f.render_widget(placeholder, area);
            return;
        }

        // Calculate bounds
        let x_min = self.data_points.front().map(|p| p.timestamp).unwrap_or(0.0);
        let x_max = self.data_points.back().map(|p| p.timestamp).unwrap_or(x_min + 10.0).max(x_min + 10.0); // Minimum time scale of 10 seconds

        // Find max speed across all peers (in KB/s)
        let y_max = self.data_points
            .iter()
            .flat_map(|dp| &dp.peer_speeds)
            .map(|ps| speed_extractor(ps) / 1024.0) // Convert to KB/s
            .fold(0.0, f64::max)
            .max(10.0); // Minimum scale of 10 KB/s

        let chart = Chart::new(datasets)
            .block(block)
            .x_axis(Axis::default().style(Style::default().fg(Color::Gray)).bounds([x_min, x_max]).labels(vec![
                Line::from(format!("{:.0}s ago", x_max - x_min)),
                Line::from(format!("{:.0}s ago", (x_max - x_min) / 2.0)),
                Line::from("now"),
            ]))
            .y_axis(Axis::default().bounds([0.0, y_max]).labels(vec![
                Line::from("0"),
                Line::from(format!("{:.1}", y_max / 2.0)),
                Line::from(format!("{y_max:.1}")),
            ]))
            .hidden_legend_constraints((Constraint::Min(0), Constraint::Min(0))) // Ensure the legend is always shown.
            .legend_position(Some(LegendPosition::TopRight));

        f.render_widget(chart, area);
    }

    /// Draws peer download speeds chart.
    fn draw_download_speed_chart(&self, f: &mut Frame, area: Rect) {
        self.draw_peer_speed_chart(f, area, "Download Speed (KB/s)", |ps| ps.download_speed);
    }

    /// Draws peer upload speeds chart.
    fn draw_upload_speed_chart(&self, f: &mut Frame, area: Rect) {
        self.draw_peer_speed_chart(f, area, "Upload Speed (KB/s)", |ps| ps.upload_speed);
    }
}
