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

use crate::{Node, content_style, header_style};
use snarkvm::prelude::Network;

use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::Line,
    widgets::{Axis, Block, Borders, Chart, Dataset, GraphType, LegendPosition, Paragraph},
};
use std::{collections::VecDeque, time::Instant};

#[derive(Debug, Clone)]
pub struct SyncDataPoint {
    pub timestamp: f64,
    pub blocks_per_second: f64,
    pub local_block_height: u32,
    pub peer_block_height: u32,
}

pub(crate) struct SyncMetrics {
    pub(super) data_points: VecDeque<SyncDataPoint>,
    start_time: Instant,
    max_data_points: usize,
    last_block_height: Option<u32>,
    last_peer_height: Option<u32>,
    last_update_time: Option<Instant>,
}

impl SyncMetrics {
    pub fn new() -> Self {
        Self {
            data_points: VecDeque::new(),
            start_time: Instant::now(),
            max_data_points: 120, // Keep 2 minutes of data at 1 second intervals
            last_block_height: None,
            last_update_time: None,
            last_peer_height: None,
        }
    }

    pub fn update_data<N: Network>(&mut self, node: &Node<N>) {
        let now = Instant::now();
        let timestamp = now.duration_since(self.start_time).as_secs_f64();

        // Get current block height
        let local_height = node.ledger().map(|l| l.latest_height()).unwrap_or(0);
        let blocks_behind = node.num_blocks_behind().unwrap_or(0);
        let sync_speed = node.get_sync_speed();
        let peer_height = local_height + blocks_behind;

        // Create new data point
        let data_point = SyncDataPoint {
            timestamp,
            local_block_height: local_height,
            peer_block_height: peer_height,
            blocks_per_second: sync_speed,
        };

        // Add data point
        self.data_points.push_back(data_point);

        // Remove old data points
        while self.data_points.len() > self.max_data_points {
            self.data_points.pop_front();
        }

        // Update last values
        self.last_block_height = Some(local_height);
        self.last_peer_height = Some(peer_height);
        self.last_update_time = Some(now);
    }

    pub fn draw<N: Network>(&mut self, f: &mut Frame, area: Rect, _node: &Node<N>) {
        let chunks = Layout::default()
            .constraints([
                Constraint::Min(15), // Blocks per second chart
                Constraint::Min(15), // Block height chart
            ])
            .split(area);

        self.draw_sync_speed_chart(f, chunks[0]);
        self.draw_block_height_chart(f, chunks[1]);
    }

    fn draw_sync_speed_chart(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).style(header_style()).title("Sync Speed (Blocks/s)");

        if self.data_points.is_empty() {
            let placeholder = Paragraph::new("Collecting data...").style(content_style()).block(block);
            f.render_widget(placeholder, area);
            return;
        }

        // Prepare data for chart
        let data: Vec<(f64, f64)> =
            self.data_points.iter().map(|point| (point.timestamp, point.blocks_per_second)).collect();

        let x_min = self.data_points.front().map(|p| p.timestamp).unwrap_or(0.0);
        let x_max = self.data_points.back().map(|p| p.timestamp).unwrap_or(x_min + 20.0).max(x_min + 20.0); // Show at least 20 seconds

        let y_max = self.data_points.iter().map(|p| p.blocks_per_second).fold(0.0f64, f64::max).max(1.0); // Minimum scale of 1 blocks/s

        let datasets = vec![
            Dataset::default()
                .name("Blocks/s")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .graph_type(GraphType::Line)
                .data(&data),
        ];

        let chart = Chart::new(datasets)
            .block(block)
            .x_axis(Axis::default().style(Style::default().fg(Color::Gray)).bounds([x_min, x_max]).labels(vec![
                Line::from(format!("{:.0}s ago", x_max - x_min)),
                Line::from(format!("{:.0}s ago", (x_max - x_min) / 2.0)),
                Line::from("now"),
            ]))
            .y_axis(Axis::default().style(Style::default().fg(Color::Gray)).bounds([0.0, y_max]).labels(vec![
                Line::from("0"),
                Line::from(format!("{:.1}", y_max / 2.0)),
                Line::from(format!("{y_max:.1}")),
            ]))
            .legend_position(Some(LegendPosition::TopRight));

        f.render_widget(chart, area);
    }

    /// Draws block height chart showing both local and peer heights.
    fn draw_block_height_chart(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).style(header_style()).title("Block Heights");

        if self.data_points.is_empty() {
            let placeholder = Paragraph::new("Collecting data...").style(content_style()).block(block);
            f.render_widget(placeholder, area);
            return;
        }

        // Prepare data for chart
        let local_height_data: Vec<(f64, f64)> =
            self.data_points.iter().map(|p| (p.timestamp, p.local_block_height as f64)).collect();

        let peer_height_data: Vec<(f64, f64)> =
            self.data_points.iter().map(|p| (p.timestamp, p.peer_block_height as f64)).collect();

        // Calculate bounds
        let x_min = self.data_points.front().map(|p| p.timestamp).unwrap_or(0.0);
        let x_max = self.data_points.back().map(|p| p.timestamp).unwrap_or(x_min + 20.0).max(x_min + 20.0); // Show at least 20 seconds
        let y_max = self
            .data_points
            .iter()
            .map(|p| p.local_block_height.max(p.peer_block_height))
            .max()
            .map(|x| x as f64)
            .unwrap_or(100.0); // Minimum scale of 100

        let datasets = vec![
            Dataset::default()
                .name("Local Height")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Cyan))
                .graph_type(GraphType::Line)
                .data(&local_height_data),
            Dataset::default()
                .name("Network Height")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Magenta))
                .graph_type(GraphType::Line)
                .data(&peer_height_data),
        ];

        let chart = Chart::new(datasets)
            .block(block)
            .x_axis(Axis::default().style(Style::default().fg(Color::Gray)).bounds([x_min, x_max]).labels(vec![
                Line::from(format!("{:.0}s ago", x_max - x_min)),
                Line::from(format!("{:.0}s ago", (x_max - x_min) / 2.0)),
                Line::from("now"),
            ]))
            .y_axis(Axis::default().style(Style::default().fg(Color::Gray)).bounds([0.0, y_max]).labels(vec![
                Line::from("0"),
                Line::from(format!("{:.0}", y_max / 2.0)),
                Line::from(format!("{y_max:.0}")),
            ]))
            .legend_position(Some(LegendPosition::TopRight));

        f.render_widget(chart, area);
    }
}
