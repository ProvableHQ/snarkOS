// Copyright (c) 2019-2026 Provable Inc.
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
    pub outstanding_block_requests: usize,
}

pub(crate) struct SyncMetrics {
    pub(super) data_points: VecDeque<SyncDataPoint>,
    start_time: Instant,
    max_data_points: usize,
    last_block_height: Option<u32>,
    last_peer_height: Option<u32>,
    last_update_time: Option<Instant>,
    time_scale_seconds: f64,
}

impl SyncMetrics {
    pub fn new() -> Self {
        Self {
            data_points: VecDeque::new(),
            start_time: Instant::now(),
            max_data_points: 900, // Keep 15 minutes of data at 1 second intervals
            last_block_height: None,
            last_update_time: None,
            last_peer_height: None,
            time_scale_seconds: 60.0, // Default to 60 seconds
        }
    }

    /// Increase the time scale (zoom out)
    pub fn scale_up(&mut self) {
        self.time_scale_seconds = (self.time_scale_seconds * 2.0).min(900.0); // Max 15 minutes
    }

    /// Decrease the time scale (zoom in)
    pub fn scale_down(&mut self) {
        self.time_scale_seconds = (self.time_scale_seconds / 2.0).max(10.0); // Min 10 seconds
    }

    pub fn update_data<N: Network>(&mut self, node: &Node<N>) {
        let now = Instant::now();
        let timestamp = now.duration_since(self.start_time).as_secs_f64();

        // Get current block height and other metrics.
        let local_height = node.ledger().map(|l| l.latest_height()).unwrap_or(0);
        let sync_speed = node.get_sync_speed();
        let peer_height = node.greatest_peer_block_height().unwrap_or(local_height);
        let outstanding_requests = node.num_outstanding_block_requests();

        // Create new data point.
        self.data_points.push_back(SyncDataPoint {
            timestamp,
            local_block_height: local_height,
            peer_block_height: peer_height,
            blocks_per_second: sync_speed,
            outstanding_block_requests: outstanding_requests,
        });

        // Remove old data points (if needed).
        while self.data_points.len() > self.max_data_points {
            self.data_points.pop_front();
        }

        // Update last values.
        self.last_block_height = Some(local_height);
        self.last_peer_height = Some(peer_height);
        self.last_update_time = Some(now);
    }

    pub fn draw(&mut self, f: &mut Frame, area: Rect) {
        let chunks = Layout::default()
            .constraints([
                Constraint::Min(12), // Blocks per second chart
                Constraint::Min(12), // Block height chart
                Constraint::Min(12), // Outstanding block requests chart
            ])
            .split(area);

        self.draw_sync_speed_chart(f, chunks[0]);
        self.draw_block_height_chart(f, chunks[1]);
        self.draw_outstanding_requests_chart(f, chunks[2]);
    }

    fn draw_sync_speed_chart(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).style(header_style()).title("Sync Speed (Blocks/s)");

        if self.data_points.is_empty() {
            let placeholder = Paragraph::new("Collecting data...").style(content_style()).block(block);
            f.render_widget(placeholder, area);
            return;
        }

        // Calculate bounds based on time scale
        let current_time = self.data_points.back().map(|p| p.timestamp).unwrap_or(0.0);
        let x_max = current_time;
        let x_min = current_time - self.time_scale_seconds;

        // Prepare data for chart, filtered by time scale
        let data: Vec<(f64, f64)> = self
            .data_points
            .iter()
            .filter(|point| point.timestamp >= x_min)
            .map(|point| (point.timestamp, point.blocks_per_second))
            .collect();

        let y_max = self
            .data_points
            .iter()
            .filter(|p| p.timestamp >= x_min)
            .map(|p| p.blocks_per_second)
            .fold(0.0f64, f64::max)
            .max(1.0); // Minimum scale of 1 blocks/s

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
                Line::from(format!("{:.0}s ago", self.time_scale_seconds)),
                Line::from(format!("{:.0}s ago", self.time_scale_seconds / 2.0)),
                Line::from("now"),
            ]))
            .y_axis(Axis::default().style(Style::default().fg(Color::Gray)).bounds([0.0, y_max]).labels(vec![
                Line::from("0"),
                Line::from(format!("{:.1}", y_max / 2.0)),
                Line::from(format!("{y_max:.1}")),
            ]))
            .legend_position(None);

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

        // Calculate bounds based on time scale
        let current_time = self.data_points.back().map(|p| p.timestamp).unwrap_or(0.0);
        let x_max = current_time;
        let x_min = current_time - self.time_scale_seconds;

        // Prepare data for chart, filtered by time scale
        let local_height_data: Vec<(f64, f64)> = self
            .data_points
            .iter()
            .filter(|p| p.timestamp >= x_min)
            .map(|p| (p.timestamp, p.local_block_height as f64))
            .collect();

        let peer_height_data: Vec<(f64, f64)> = self
            .data_points
            .iter()
            .filter(|p| p.timestamp >= x_min)
            .map(|p| (p.timestamp, p.peer_block_height as f64))
            .collect();

        let y_max = self
            .data_points
            .iter()
            .filter(|p| p.timestamp >= x_min)
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
                .name("Peer Height")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Magenta))
                .graph_type(GraphType::Line)
                .data(&peer_height_data),
        ];

        let chart = Chart::new(datasets)
            .block(block)
            .x_axis(Axis::default().style(Style::default().fg(Color::Gray)).bounds([x_min, x_max]).labels(vec![
                Line::from(format!("{:.0}s ago", self.time_scale_seconds)),
                Line::from(format!("{:.0}s ago", self.time_scale_seconds / 2.0)),
                Line::from("now"),
            ]))
            .y_axis(Axis::default().style(Style::default().fg(Color::Gray)).bounds([0.0, y_max]).labels(vec![
                Line::from("0"),
                Line::from(format!("{:.0}", y_max / 2.0)),
                Line::from(format!("{y_max:.0}")),
            ]))
            .hidden_legend_constraints((Constraint::Min(0), Constraint::Min(0))) // Ensure the legend is always shown.
            .legend_position(Some(LegendPosition::TopLeft));

        f.render_widget(chart, area);
    }

    /// Draws outstanding block requests chart.
    fn draw_outstanding_requests_chart(&self, f: &mut Frame, area: Rect) {
        let block = Block::default().borders(Borders::ALL).style(header_style()).title("Outstanding Block Requests");

        if self.data_points.is_empty() {
            let placeholder = Paragraph::new("Collecting data...").style(content_style()).block(block);
            f.render_widget(placeholder, area);
            return;
        }

        // Calculate bounds based on time scale
        let current_time = self.data_points.back().map(|p| p.timestamp).unwrap_or(0.0);
        let x_max = current_time;
        let x_min = current_time - self.time_scale_seconds;

        // Prepare data for chart, filtered by time scale
        let data: Vec<(f64, f64)> = self
            .data_points
            .iter()
            .filter(|p| p.timestamp >= x_min)
            .map(|p| (p.timestamp, p.outstanding_block_requests as f64))
            .collect();

        let y_max = self
            .data_points
            .iter()
            .filter(|p| p.timestamp >= x_min)
            .map(|p| p.outstanding_block_requests)
            .max()
            .unwrap_or(1)
            .max(1) as f64; // Minimum scale of 1

        let datasets = vec![
            Dataset::default()
                .name("Outstanding Requests")
                .marker(symbols::Marker::Dot)
                .style(Style::default().fg(Color::Yellow))
                .graph_type(GraphType::Line)
                .data(&data),
        ];

        let chart = Chart::new(datasets)
            .block(block)
            .x_axis(Axis::default().style(Style::default().fg(Color::Gray)).bounds([x_min, x_max]).labels(vec![
                Line::from(format!("{:.0}s ago", self.time_scale_seconds)),
                Line::from(format!("{:.0}s ago", self.time_scale_seconds / 2.0)),
                Line::from("now"),
            ]))
            .y_axis(Axis::default().style(Style::default().fg(Color::Gray)).bounds([0.0, y_max]).labels(vec![
                Line::from("0"),
                Line::from(format!("{:.0}", y_max / 2.0)),
                Line::from(format!("{y_max:.0}")),
            ]))
            .legend_position(None);

        f.render_widget(chart, area);
    }
}
