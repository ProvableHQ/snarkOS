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

mod data;
mod visualization;

use anyhow::{Context, Result};
use clap::{Arg, Command};
use data::load_timing_data;
use visualization::{generate_scatter_chart, generate_text_visualization, print_summary};

fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let matches = Command::new("timing_analysis")
        .version("0.1.0")
        .author("The Aleo Team <hello@aleo.org>")
        .about("Analyze consensus timing data from snarkOS JSON files")
        .arg(
            Arg::new("json-file")
                .long("json-file")
                .value_name("FILE")
                .help("Path to the consensus_timing_block.json file")
                .required(true),
        )
        .arg(
            Arg::new("output")
                .long("output")
                .value_name("FILE")
                .help("Output file name for the graph")
                .default_value("consensus_timing_analysis.svg"),
        )
        .arg(
            Arg::new("width")
                .long("width")
                .value_name("PIXELS")
                .help("Width of the output image in pixels")
                .default_value("1200"),
        )
        .arg(
            Arg::new("height")
                .long("height")
                .value_name("PIXELS")
                .help("Height of the output image in pixels")
                .default_value("800"),
        )
        .arg(
            Arg::new("text-only")
                .long("text-only")
                .help("Only show text-based visualization (no chart generation)")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let json_file = matches.get_one::<String>("json-file").unwrap();
    let output_file = matches.get_one::<String>("output").unwrap();
    let width: u32 = matches.get_one::<String>("width").unwrap().parse()
        .context("Width must be a valid number")?;
    let height: u32 = matches.get_one::<String>("height").unwrap().parse()
        .context("Height must be a valid number")?;
    let text_only = matches.get_flag("text-only");

    // Check if the JSON file exists
    if !std::path::Path::new(json_file).exists() {
        return Err(anyhow::anyhow!("JSON file '{}' not found", json_file));
    }

    println!("Loading timing data from: {}", json_file);

    // Load and parse the timing data
    let timing_data = load_timing_data(json_file)
        .context("Failed to load timing data")?;

    // Print summary statistics
    print_summary(&timing_data);

    // Generate visualization
    if text_only {
        generate_text_visualization(&timing_data)
            .context("Failed to generate text visualization")?;
    } else {
        // Try to generate the chart
        match generate_scatter_chart(&timing_data, output_file, width, height) {
            Ok(()) => {
                println!("Chart successfully saved to: {}", output_file);
            }
            Err(e) => {
                eprintln!("Failed to generate chart: {}", e);
                println!("Falling back to text-based visualization:");
                generate_text_visualization(&timing_data)
                    .context("Failed to generate text visualization")?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::{JsonSystemTime, JsonRoundEvents, JsonTimingEvent, RawSubdagTiming, RawTimingSnapshot};
    use std::collections::HashMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_test_json_file() -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        
        let test_data = RawTimingSnapshot {
            timestamp: JsonSystemTime {
                secs_since_epoch: 1640995200,
                nanos_since_epoch: 0,
            },
            round_events: {
                let mut map = HashMap::new();
                map.insert("100".to_string(), JsonRoundEvents {
                    round: 100,
                    proposal_seen: vec![JsonTimingEvent {
                        round: 100,
                        timestamp: JsonSystemTime { secs_since_epoch: 1640995200, nanos_since_epoch: 0 },
                        event_type: "proposal_seen".to_string(),
                        is_local: Some(false),
                    }],
                    proposal_created: vec![JsonTimingEvent {
                        round: 100,
                        timestamp: JsonSystemTime { secs_since_epoch: 1640995201, nanos_since_epoch: 500_000_000 },
                        event_type: "proposal_created".to_string(),
                        is_local: Some(true),
                    }],
                    certificate_added: vec![JsonTimingEvent {
                        round: 100,
                        timestamp: JsonSystemTime { secs_since_epoch: 1640995202, nanos_since_epoch: 200_000_000 },
                        event_type: "certificate_added".to_string(),
                        is_local: None,
                    }],
                });
                map
            },
            subdag_timings: {
                let mut map = HashMap::new();
                map.insert("100-102".to_string(), RawSubdagTiming {
                    lowest_round: 100,
                    highest_round: 102,
                    subdag_processing: Some((
                        JsonSystemTime { secs_since_epoch: 1640995207, nanos_since_epoch: 500_000_000 },
                        Some(JsonSystemTime { secs_since_epoch: 1640995208, nanos_since_epoch: 200_000_000 })
                    )),
                    check_next_block: Some((
                        JsonSystemTime { secs_since_epoch: 1640995208, nanos_since_epoch: 200_000_000 },
                        Some(JsonSystemTime { secs_since_epoch: 1640995208, nanos_since_epoch: 800_000_000 })
                    )),
                    advance_to_next_block: Some((
                        JsonSystemTime { secs_since_epoch: 1640995208, nanos_since_epoch: 800_000_000 },
                        Some(JsonSystemTime { secs_since_epoch: 1640995209, nanos_since_epoch: 500_000_000 })
                    )),
                });
                map
            },
        };

        let json_str = serde_json::to_string_pretty(&test_data).unwrap();
        file.write_all(json_str.as_bytes()).unwrap();
        file.flush().unwrap();
        
        file
    }

    #[test]
    fn test_load_timing_data_integration() {
        let test_file = create_test_json_file();
        let file_path = test_file.path().to_str().unwrap();
        
        let result = load_timing_data(file_path);
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert_eq!(data.events.len(), 3);
        assert_eq!(data.subdag_timings.len(), 1);
        
        // Verify event data
        let proposal_created_events = data.get_events_by_type("proposal_created");
        assert_eq!(proposal_created_events.len(), 1);
        assert_eq!(proposal_created_events[0].round, 100);
        
        let proposal_seen_events = data.get_events_by_type("proposal_seen");
        assert_eq!(proposal_seen_events.len(), 1);
        assert_eq!(proposal_seen_events[0].is_local, Some(false));
        
        // Verify subdag timing data
        let subdag_100_102 = data.subdag_timings.get(&(100, 102)).unwrap();
        assert!(subdag_100_102.contains_key("subdag_processing"));
        assert!(subdag_100_102.contains_key("check_next_block"));
        assert!(subdag_100_102.contains_key("advance_to_next_block"));
    }

    #[test]
    fn test_nonexistent_file() {
        let result = load_timing_data("nonexistent_file.json");
        assert!(result.is_err());
    }
}