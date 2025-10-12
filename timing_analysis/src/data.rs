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

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Represents a SystemTime timestamp from the JSON data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonSystemTime {
    pub secs_since_epoch: u64,
    pub nanos_since_epoch: u32,
}

impl From<JsonSystemTime> for SystemTime {
    fn from(jst: JsonSystemTime) -> Self {
        UNIX_EPOCH + Duration::new(jst.secs_since_epoch, jst.nanos_since_epoch)
    }
}

impl From<JsonSystemTime> for f64 {
    fn from(jst: JsonSystemTime) -> Self {
        jst.secs_since_epoch as f64 + (jst.nanos_since_epoch as f64 / 1_000_000_000.0)
    }
}

/// Timing data for a single consensus stage
#[derive(Debug, Clone)]
pub struct TimingData {
    pub stage_name: String,
    pub start_time: f64,
    pub end_time: Option<f64>,
}

impl TimingData {
    pub fn new(stage_name: String, start_time: f64, end_time: Option<f64>) -> Self {
        Self {
            stage_name,
            start_time,
            end_time,
        }
    }

    /// Get the duration of this timing stage
    pub fn duration(&self) -> Option<f64> {
        self.end_time.map(|end| end - self.start_time)
    }

    /// Check if this timing stage is complete (has both start and end times)
    pub fn is_complete(&self) -> bool {
        self.end_time.is_some()
    }
}

/// Timing event for a specific consensus stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTimingEvent {
    pub round: u64,
    pub timestamp: JsonSystemTime,
    pub event_type: String,
    pub is_local: Option<bool>, // Some(true) for local events, Some(false) for peer events, None for unknown
}

/// Round-based events from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRoundEvents {
    pub round: u64,
    pub proposal_seen: Vec<JsonTimingEvent>,
    pub proposal_created: Vec<JsonTimingEvent>,
    pub certificate_added: Vec<JsonTimingEvent>,
}

/// Raw subdag timing data from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawSubdagTiming {
    pub lowest_round: u64,
    pub highest_round: u64,
    pub subdag_processing: Option<(JsonSystemTime, Option<JsonSystemTime>)>,
    pub check_next_block: Option<(JsonSystemTime, Option<JsonSystemTime>)>,
    pub advance_to_next_block: Option<(JsonSystemTime, Option<JsonSystemTime>)>,
}

/// Raw timing snapshot from JSON file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawTimingSnapshot {
    pub timestamp: JsonSystemTime,
    pub round_events: HashMap<String, JsonRoundEvents>,
    pub subdag_timings: HashMap<String, RawSubdagTiming>,
}

/// Processed timing event
#[derive(Debug, Clone)]
pub struct TimingEvent {
    pub round: u64,
    pub timestamp: f64,
    pub event_type: String,
    pub is_local: Option<bool>,
}

impl TimingEvent {
    pub fn new(round: u64, timestamp: f64, event_type: String, is_local: Option<bool>) -> Self {
        Self {
            round,
            timestamp,
            event_type,
            is_local,
        }
    }

    /// Check if this is a local event (created by this node)
    pub fn is_local_event(&self) -> bool {
        self.is_local.unwrap_or(false)
    }
}

/// Processed timing data for analysis
#[derive(Debug, Clone)]
pub struct ProcessedTimingData {
    pub snapshot_timestamp: f64,
    pub events: Vec<TimingEvent>,
    pub subdag_timings: IndexMap<(u64, u64), HashMap<String, TimingData>>,
}

impl ProcessedTimingData {
    /// Create a new empty processed timing data
    pub fn new(snapshot_timestamp: f64) -> Self {
        Self {
            snapshot_timestamp,
            events: Vec::new(),
            subdag_timings: IndexMap::new(),
        }
    }

    /// Add an event to the processed data
    pub fn add_event(&mut self, event: TimingEvent) {
        self.events.push(event);
    }

    /// Get the overall time range of all timing data
    pub fn get_time_range(&self) -> (f64, f64) {
        let mut all_times = Vec::new();

        // Collect all timestamps from events
        for event in &self.events {
            all_times.push(event.timestamp);
        }

        // Collect all timestamps from subdag timings
        for subdag_data in self.subdag_timings.values() {
            for timing in subdag_data.values() {
                all_times.push(timing.start_time);
                if let Some(end_time) = timing.end_time {
                    all_times.push(end_time);
                }
            }
        }

        if all_times.is_empty() {
            return (0.0, 1.0);
        }

        let min_time = all_times.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max_time = all_times.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

        (min_time, max_time)
    }

    /// Get all rounds that have timing data
    pub fn get_all_rounds(&self) -> Vec<u64> {
        let mut rounds = std::collections::HashSet::new();

        // Add rounds from events
        for event in &self.events {
            rounds.insert(event.round);
        }

        // Add rounds from subdag timings
        for &(low, high) in self.subdag_timings.keys() {
            for round in low..=high {
                rounds.insert(round);
            }
        }

        let mut sorted_rounds: Vec<u64> = rounds.into_iter().collect();
        sorted_rounds.sort();
        sorted_rounds
    }

    /// Get all event types
    pub fn get_all_event_types(&self) -> Vec<String> {
        let mut event_types = std::collections::HashSet::new();
        for event in &self.events {
            event_types.insert(event.event_type.clone());
        }
        let mut sorted_types: Vec<String> = event_types.into_iter().collect();
        sorted_types.sort();
        sorted_types
    }

    /// Get events for a specific round
    pub fn get_events_for_round(&self, round: u64) -> Vec<&TimingEvent> {
        self.events.iter().filter(|e| e.round == round).collect()
    }

    /// Get events by type
    pub fn get_events_by_type(&self, event_type: &str) -> Vec<&TimingEvent> {
        self.events.iter().filter(|e| e.event_type == event_type).collect()
    }

    /// Filter data to only include events and subdags within the specified round range
    pub fn filter_by_round_range(&self, start_round: Option<u64>, end_round: Option<u64>) -> ProcessedTimingData {
        let mut filtered = ProcessedTimingData::new(self.snapshot_timestamp);

        // Filter events
        for event in &self.events {
            let include_event = match (start_round, end_round) {
                (Some(start), Some(end)) => event.round >= start && event.round <= end,
                (Some(start), None) => event.round >= start,
                (None, Some(end)) => event.round <= end,
                (None, None) => true,
            };

            if include_event {
                filtered.add_event(event.clone());
            }
        }

        // Filter subdag timings
        for (&(low_round, high_round), subdag_data) in &self.subdag_timings {
            // Include subdag if any part of its range overlaps with the filter range
            let include_subdag = match (start_round, end_round) {
                (Some(start), Some(end)) => {
                    // Include if subdag range overlaps with filter range
                    !(high_round < start || low_round > end)
                },
                (Some(start), None) => high_round >= start,
                (None, Some(end)) => low_round <= end,
                (None, None) => true,
            };

            if include_subdag {
                filtered.subdag_timings.insert((low_round, high_round), subdag_data.clone());
            }
        }

        filtered
    }
}

/// Parse raw timing data into processed format
pub fn parse_timing_data(raw: RawTimingSnapshot) -> Result<ProcessedTimingData> {
    let snapshot_timestamp = raw.timestamp.into();
    let mut processed = ProcessedTimingData::new(snapshot_timestamp);

    // Process round events
    for (round_str, round_events) in raw.round_events {
        let round_num: u64 = round_str.parse()
            .with_context(|| format!("Failed to parse round number: {}", round_str))?;

        // Process each event type
        for event in round_events.proposal_seen {
            processed.add_event(TimingEvent::new(
                round_num,
                event.timestamp.into(),
                "proposal_seen".to_string(),
                event.is_local,
            ));
        }

        for event in round_events.proposal_created {
            processed.add_event(TimingEvent::new(
                round_num,
                event.timestamp.into(),
                "proposal_created".to_string(),
                event.is_local,
            ));
        }

        for event in round_events.certificate_added {
            processed.add_event(TimingEvent::new(
                round_num,
                event.timestamp.into(),
                "certificate_added".to_string(),
                event.is_local,
            ));
        }
    }

    // Process subdag timings (unchanged)
    for (subdag_str, raw_subdag) in raw.subdag_timings {
        let parts: Vec<&str> = subdag_str.split('-').collect();
        if parts.len() != 2 {
            continue;
        }

        let low_round: u64 = parts[0].parse()
            .with_context(|| format!("Failed to parse low round: {}", parts[0]))?;
        let high_round: u64 = parts[1].parse()
            .with_context(|| format!("Failed to parse high round: {}", parts[1]))?;

        let subdag_key = (low_round, high_round);
        let mut subdag_data = HashMap::new();

        // Process each stage
        if let Some((start, end)) = raw_subdag.subdag_processing {
            let start_time: f64 = start.into();
            let end_time = end.map(|e| e.into());
            subdag_data.insert(
                "subdag_processing".to_string(),
                TimingData::new("subdag_processing".to_string(), start_time, end_time),
            );
        }

        if let Some((start, end)) = raw_subdag.check_next_block {
            let start_time: f64 = start.into();
            let end_time = end.map(|e| e.into());
            subdag_data.insert(
                "check_next_block".to_string(),
                TimingData::new("check_next_block".to_string(), start_time, end_time),
            );
        }

        if let Some((start, end)) = raw_subdag.advance_to_next_block {
            let start_time: f64 = start.into();
            let end_time = end.map(|e| e.into());
            subdag_data.insert(
                "advance_to_next_block".to_string(),
                TimingData::new("advance_to_next_block".to_string(), start_time, end_time),
            );
        }

        if !subdag_data.is_empty() {
            processed.subdag_timings.insert(subdag_key, subdag_data);
        }
    }

    Ok(processed)
}

/// Load and parse timing data from a JSON file
pub fn load_timing_data(file_path: &str) -> Result<ProcessedTimingData> {
    let file_content = std::fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path))?;

    let raw_data: RawTimingSnapshot = serde_json::from_str(&file_content)
        .with_context(|| format!("Failed to parse JSON from file: {}", file_path))?;

    parse_timing_data(raw_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_json_system_time_conversion() {
        let jst = JsonSystemTime {
            secs_since_epoch: 1640995200,
            nanos_since_epoch: 500_000_000,
        };

        let as_f64: f64 = jst.into();
        assert!((as_f64 - 1640995200.5).abs() < 1e-6);
    }

    #[test]
    fn test_timing_data() {
        let timing = TimingData::new("test_stage".to_string(), 100.0, Some(101.5));
        
        assert_eq!(timing.stage_name, "test_stage");
        assert_eq!(timing.start_time, 100.0);
        assert_eq!(timing.end_time, Some(101.5));
        assert_eq!(timing.duration(), Some(1.5));
        assert!(timing.is_complete());

        let incomplete_timing = TimingData::new("test_stage".to_string(), 100.0, None);
        assert!(!incomplete_timing.is_complete());
        assert_eq!(incomplete_timing.duration(), None);
    }

    #[test]
    fn test_parse_timing_data() {
        let raw = RawTimingSnapshot {
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
                    certificate_added: vec![],
                });
                map
            },
            subdag_timings: {
                let mut map = HashMap::new();
                map.insert("100-102".to_string(), RawSubdagTiming {
                    lowest_round: 100,
                    highest_round: 102,
                    subdag_processing: Some((
                        JsonSystemTime { secs_since_epoch: 1640995202, nanos_since_epoch: 0 },
                        Some(JsonSystemTime { secs_since_epoch: 1640995203, nanos_since_epoch: 0 })
                    )),
                    check_next_block: None,
                    advance_to_next_block: None,
                });
                map
            },
        };

        let processed = parse_timing_data(raw).unwrap();
        
        assert_eq!(processed.snapshot_timestamp, 1640995200.0);
        assert_eq!(processed.events.len(), 2);
        assert_eq!(processed.subdag_timings.len(), 1);

        let proposal_seen = processed.events.iter().find(|e| e.event_type == "proposal_seen").unwrap();
        assert_eq!(proposal_seen.round, 100);
        assert_eq!(proposal_seen.timestamp, 1640995200.0);
        assert_eq!(proposal_seen.is_local, Some(false));
        
        let proposal_created = processed.events.iter().find(|e| e.event_type == "proposal_created").unwrap();
        assert_eq!(proposal_created.round, 100);
        assert_eq!(proposal_created.timestamp, 1640995201.5);
        assert_eq!(proposal_created.is_local, Some(true));
    }

    #[test]
    fn test_get_time_range() {
        let mut processed = ProcessedTimingData::new(1640995200.0);
        
        // Add some events
        processed.add_event(TimingEvent::new(100, 100.0, "proposal_created".to_string(), Some(true)));
        processed.add_event(TimingEvent::new(101, 101.5, "proposal_seen".to_string(), Some(false)));

        // Add some subdag timing data
        let mut subdag_data = HashMap::new();
        subdag_data.insert(
            "subdag_processing".to_string(),
            TimingData::new("subdag_processing".to_string(), 102.0, Some(104.0)),
        );
        processed.subdag_timings.insert((100, 102), subdag_data);

        let (min_time, max_time) = processed.get_time_range();
        assert_eq!(min_time, 100.0);
        assert_eq!(max_time, 104.0);
    }

    #[test]
    fn test_get_all_rounds() {
        let mut processed = ProcessedTimingData::new(1640995200.0);
        
        // Add event for round 100
        processed.add_event(TimingEvent::new(100, 100.0, "proposal_created".to_string(), Some(true)));

        // Add subdag spanning rounds 102-105
        let mut subdag_data = HashMap::new();
        subdag_data.insert(
            "subdag_processing".to_string(),
            TimingData::new("subdag_processing".to_string(), 102.0, Some(104.0)),
        );
        processed.subdag_timings.insert((102, 105), subdag_data);

        let rounds = processed.get_all_rounds();
        assert_eq!(rounds, vec![100, 102, 103, 104, 105]);
    }

    #[test]
    fn test_filter_by_round_range() {
        let mut processed = ProcessedTimingData::new(1640995200.0);
        
        // Add events for different rounds
        processed.add_event(TimingEvent::new(100, 100.0, "proposal_created".to_string(), Some(true)));
        processed.add_event(TimingEvent::new(150, 150.0, "proposal_seen".to_string(), Some(false)));
        processed.add_event(TimingEvent::new(200, 200.0, "certificate_added".to_string(), None));

        // Add subdag data
        let mut subdag_data = HashMap::new();
        subdag_data.insert(
            "subdag_processing".to_string(),
            TimingData::new("subdag_processing".to_string(), 102.0, Some(104.0)),
        );
        processed.subdag_timings.insert((100, 105), subdag_data.clone());
        processed.subdag_timings.insert((180, 185), subdag_data);

        // Test filtering with both start and end
        let filtered = processed.filter_by_round_range(Some(120), Some(180));
        assert_eq!(filtered.events.len(), 1); // Only round 150 event
        assert_eq!(filtered.subdag_timings.len(), 1); // Only the (180, 185) subdag
        
        // Test filtering with only start
        let filtered = processed.filter_by_round_range(Some(150), None);
        assert_eq!(filtered.events.len(), 2); // Rounds 150 and 200
        assert_eq!(filtered.subdag_timings.len(), 1); // Only the (180, 185) subdag
        
        // Test filtering with only end
        let filtered = processed.filter_by_round_range(None, Some(150));
        assert_eq!(filtered.events.len(), 2); // Rounds 100 and 150
        assert_eq!(filtered.subdag_timings.len(), 1); // Only the (100, 105) subdag
        
        // Test no filtering
        let filtered = processed.filter_by_round_range(None, None);
        assert_eq!(filtered.events.len(), 3); // All events
        assert_eq!(filtered.subdag_timings.len(), 2); // All subdags
    }
}
