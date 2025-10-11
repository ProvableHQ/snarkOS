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

use crate::data::{ProcessedTimingData, TimingData};
use anyhow::{Context, Result};
use plotters::prelude::*;
use std::collections::HashMap;

/// Color mapping for different event types
pub struct EventColors;

impl EventColors {
    /// Get color for a specific event type
    pub fn get_color(event_type: &str) -> RGBColor {
        match event_type {
            "proposal_created" => RGBColor(34, 139, 34),    // Forest Green
            "proposal_seen" => RGBColor(50, 205, 50),       // Lime Green
            "certificate_added" => RGBColor(255, 140, 0),   // Dark Orange
            "subdag_processing" => RGBColor(214, 39, 40),   // Red
            "check_next_block" => RGBColor(148, 103, 189),  // Purple
            "advance_to_next_block" => RGBColor(140, 86, 75), // Brown
            _ => RGBColor(128, 128, 128),                   // Gray for unknown types
        }
    }

    /// Get display name for event type
    pub fn get_display_name(event_type: &str) -> &str {
        match event_type {
            "proposal_created" => "Proposal Created",
            "proposal_seen" => "Proposal Seen",
            "certificate_added" => "Certificate Added",
            "subdag_processing" => "Subdag Processing",
            "check_next_block" => "Check Next Block",
            "advance_to_next_block" => "Advance to Next Block",
            _ => event_type,
        }
    }
}

/// Generate a scatter plot chart showing timing events as dots and subdag stages as bars
pub fn generate_scatter_chart(
    data: &ProcessedTimingData,
    output_path: &str,
    width: u32,
    height: u32,
) -> Result<()> {
    if data.events.is_empty() && data.subdag_timings.is_empty() {
        return Err(anyhow::anyhow!("No timing data to visualize"));
    }

    let all_rounds = data.get_all_rounds();
    if all_rounds.is_empty() {
        return Err(anyhow::anyhow!("No rounds found in timing data"));
    }

    let (min_time, max_time) = data.get_time_range();
    let start_timestamp = min_time;
    let total_duration = max_time - min_time;

    let min_round = *all_rounds.first().unwrap() as f64;
    let max_round = *all_rounds.last().unwrap() as f64;
    let time_range = max_time - min_time;
    let time_padding = time_range * 0.05; // 5% padding
    let round_padding = (max_round - min_round) * 0.05; // 5% padding

    // Create the drawing backend
    let root = SVGBackend::new(output_path, (width, height)).into_drawing_area();
    root.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&root)
        .caption("Consensus Events Timeline", ("sans-serif", 40))
        .margin(20)
        .x_label_area_size(60)
        .y_label_area_size(80)
        .build_cartesian_2d(
            (min_round - round_padding)..(max_round + round_padding),
            (min_time - time_padding)..(max_time + time_padding),
        )?;

    chart
        .configure_mesh()
        .x_desc("Round")
        .y_desc("Time (seconds since epoch)")
        .draw()?;

    // Define the chronological order of event types for the legend
    let chronological_events = [
        ("proposal_created", "Proposal Created"),
        ("proposal_seen", "Proposal Seen"),
        ("certificate_added", "Certificate Added"),
        ("subdag_processing", "Subdag Processing"),
        ("check_next_block", "Check Next Block"),
        ("advance_to_next_block", "Advance to Next Block"),
    ];

    // Track which event types are actually present in the data
    let mut event_types_present = std::collections::HashSet::new();

    // Draw events as dots
    for event in &data.events {
        let color = EventColors::get_color(&event.event_type);
        let x = event.round as f64;
        let y = event.timestamp;

        // Draw a circle for each event
        chart.draw_series(std::iter::once(Circle::new((x, y), 4, color.filled())))?;

        // Mark this event type as present
        event_types_present.insert(event.event_type.as_str());
    }

    // Draw subdag stages as bars
    for (&(low_round, high_round), subdag_data) in &data.subdag_timings {
        for (stage_name, timing) in subdag_data {
            if !timing.is_complete() {
                continue;
            }

            let x_center = (low_round + high_round) as f64 / 2.0;
            let width = (high_round - low_round + 1) as f64 * 0.8;
            let y_start = timing.start_time;
            let height = timing.duration().unwrap();

            let color = EventColors::get_color(stage_name);

            // Draw the bar
            chart.draw_series(std::iter::once(Rectangle::new(
                [(x_center - width / 2.0, y_start), (x_center + width / 2.0, y_start + height)],
                color.filled(),
            )))?;

            // Mark this stage as present
            event_types_present.insert(stage_name.as_str());
        }
    }

    // Create legend entries in chronological order
    for (event_key, display_name) in chronological_events.iter() {
        if event_types_present.contains(event_key) {
            let color = EventColors::get_color(event_key);
            
            // Create a dummy series just for the legend entry
            let legend_series = if event_key.starts_with("subdag") || event_key.starts_with("check") || event_key.starts_with("advance") {
                // For subdag stages, use rectangle legend
                let series = chart.draw_series(std::iter::empty::<Rectangle<(f64, f64)>>())?;
                series.label(*display_name).legend(move |(x, y)| {
                    Rectangle::new([(x, y), (x + 10, y + 10)], color.filled())
                });
                series
            } else {
                // For events, use circle legend
                let series = chart.draw_series(std::iter::empty::<Circle<(f64, f64), i32>>())?;
                series.label(*display_name).legend(move |(x, y)| {
                    Circle::new((x + 5, y + 5), 4, color.filled())
                });
                series
            };
        }
    }

    // Draw legend
    chart
        .configure_series_labels()
        .background_style(&WHITE.mix(0.8))
        .border_style(&BLACK)
        .draw()?;

    root.present()?;
    println!("Chart saved to {}", output_path);

    Ok(())
}

/// Generate a text-based visualization as a fallback
pub fn generate_text_visualization(data: &ProcessedTimingData) -> Result<()> {
    if data.events.is_empty() && data.subdag_timings.is_empty() {
        println!("No timing data to visualize");
        return Ok(());
    }

    println!("Consensus Events Timeline (Text View)");
    println!("=====================================");
    println!();

    let (min_time, max_time) = data.get_time_range();
    let rounds = data.get_all_rounds();

    println!("Time range: {:.3} - {:.3} seconds", min_time, max_time);
    println!("Round range: {} - {}", rounds.first().unwrap(), rounds.last().unwrap());
    println!("Total events: {}", data.events.len());
    println!();

    // Group events by round for better readability
    for &round in &rounds {
        let round_events = data.get_events_for_round(round);
        if round_events.is_empty() {
            continue;
        }

        println!("Round {}:", round);
        for event in round_events {
            let relative_time = event.timestamp - min_time;
            let local_marker = if event.is_local_event() { " (local)" } else { "" };
            println!("  [{:8.3}s] {}{}", 
                relative_time, 
                EventColors::get_display_name(&event.event_type),
                local_marker
            );
        }
        println!();
    }

    // Display subdag stages
    if !data.subdag_timings.is_empty() {
        println!("Subdag stages:");
        for (&(low_round, high_round), subdag_data) in &data.subdag_timings {
            println!("Rounds {}-{}:", low_round, high_round);
            
            for stage_name in &["subdag_processing", "check_next_block", "advance_to_next_block"] {
                if let Some(timing) = subdag_data.get(*stage_name) {
                    if timing.is_complete() {
                        let relative_start = timing.start_time - min_time;
                        let duration = timing.duration().unwrap();
                        println!("  [{:8.3}s] {} (duration: {:.3}s)", 
                            relative_start,
                            EventColors::get_display_name(stage_name),
                            duration
                        );
                    }
                }
            }
            println!();
        }
    }

    // Summary by event type
    println!("Event Type Summary:");
    println!("==================");
    let event_types = data.get_all_event_types();
    for event_type in event_types {
        let events = data.get_events_by_type(&event_type);
        let local_count = events.iter().filter(|e| e.is_local_event()).count();
        let remote_count = events.len() - local_count;
        
        println!("{}: {} total", EventColors::get_display_name(&event_type), events.len());
        if event_type.starts_with("proposal") {
            println!("  - Local: {}, Remote: {}", local_count, remote_count);
        }
    }

    Ok(())
}

/// Print a statistical summary of the timing data
pub fn print_summary(data: &ProcessedTimingData) {
    println!("\n=== Consensus Timing Summary ===");
    println!("Total events: {}", data.events.len());
    println!("Total subdags with timing data: {}", data.subdag_timings.len());

    if !data.events.is_empty() {
        println!("\nEvent summary:");
        let event_types = data.get_all_event_types();
        for event_type in &event_types {
            let events = data.get_events_by_type(event_type);
            let local_count = events.iter().filter(|e| e.is_local_event()).count();
            let remote_count = events.len() - local_count;
            
            println!("  {}: {} total", EventColors::get_display_name(event_type), events.len());
            if event_type.starts_with("proposal") {
                println!("    - Local: {}, Remote: {}", local_count, remote_count);
            }
        }
    }

    if !data.subdag_timings.is_empty() {
        println!("\nSubdag-based stages:");
        for stage in &["subdag_processing", "check_next_block", "advance_to_next_block"] {
            let mut durations = Vec::new();
            let mut count = 0;

            for subdag_data in data.subdag_timings.values() {
                if let Some(timing) = subdag_data.get(*stage) {
                    if timing.is_complete() {
                        count += 1;
                        durations.push(timing.duration().unwrap());
                    }
                }
            }

            if count > 0 {
                let avg_duration = durations.iter().sum::<f64>() / durations.len() as f64;
                let display_name = EventColors::get_display_name(stage);
                println!("  {}: {} complete entries, avg duration: {:.3}s", 
                         display_name, count, avg_duration);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{ProcessedTimingData, TimingData};
    use std::collections::HashMap;
    use tempfile::NamedTempFile;

    #[test]
    fn test_event_colors() {
        let proposal_created_color = EventColors::get_color("proposal_created");
        let proposal_seen_color = EventColors::get_color("proposal_seen");
        let certificate_added_color = EventColors::get_color("certificate_added");
        let unknown_color = EventColors::get_color("unknown_event");

        // Colors should be different for different event types
        assert_ne!(proposal_created_color, proposal_seen_color);
        assert_ne!(proposal_seen_color, certificate_added_color);
        assert_eq!(unknown_color, RGBColor(128, 128, 128)); // Gray for unknown
    }

    #[test]
    fn test_display_names() {
        assert_eq!(EventColors::get_display_name("proposal_created"), "Proposal Created");
        assert_eq!(EventColors::get_display_name("proposal_seen"), "Proposal Seen");
        assert_eq!(EventColors::get_display_name("certificate_added"), "Certificate Added");
        assert_eq!(EventColors::get_display_name("unknown"), "unknown");
    }

    #[test]
    fn test_empty_data_handling() {
        let empty_data = ProcessedTimingData::new(1640995200.0);
        
        // Should return error for empty data
        let result = generate_scatter_chart(&empty_data, "test.svg", 800, 600);
        assert!(result.is_err());
        
        // Text visualization should handle empty data gracefully
        let result = generate_text_visualization(&empty_data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_single_event_data() {
        let mut data = ProcessedTimingData::new(1640995200.0);
        data.add_event(crate::data::TimingEvent::new(100, 1640995200.0, "proposal_created".to_string(), Some(true)));

        // Should work with single event
        let rounds = data.get_all_rounds();
        assert_eq!(rounds, vec![100]);
        
        let (min_time, max_time) = data.get_time_range();
        assert_eq!(min_time, 1640995200.0);
        assert_eq!(max_time, 1640995200.0);
    }

    #[test]
    fn test_multiple_events_data() {
        let mut data = ProcessedTimingData::new(1640995200.0);
        data.add_event(crate::data::TimingEvent::new(100, 1640995200.0, "proposal_created".to_string(), Some(true)));
        data.add_event(crate::data::TimingEvent::new(101, 1640995201.0, "proposal_seen".to_string(), Some(false)));
        data.add_event(crate::data::TimingEvent::new(100, 1640995202.0, "certificate_added".to_string(), None));

        let rounds = data.get_all_rounds();
        assert_eq!(rounds, vec![100, 101]);
        
        let (min_time, max_time) = data.get_time_range();
        assert_eq!(min_time, 1640995200.0);
        assert_eq!(max_time, 1640995202.0);

        let event_types = data.get_all_event_types();
        assert_eq!(event_types.len(), 3);
        assert!(event_types.contains(&"proposal_created".to_string()));
        assert!(event_types.contains(&"proposal_seen".to_string()));
        assert!(event_types.contains(&"certificate_added".to_string()));
    }
}
