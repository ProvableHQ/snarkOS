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

/// Raw round timing data from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawRoundTiming {
    pub round: u64,
    pub proposal_generation: Option<(JsonSystemTime, Option<JsonSystemTime>)>,
    pub certificate_generation: Option<(JsonSystemTime, Option<JsonSystemTime>)>,
    pub certificate_collection: Option<(JsonSystemTime, Option<JsonSystemTime>)>,
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
    pub round_timings: HashMap<String, RawRoundTiming>,
    pub subdag_timings: HashMap<String, RawSubdagTiming>,
}

/// Processed timing data for analysis
#[derive(Debug, Clone)]
pub struct ProcessedTimingData {
    pub snapshot_timestamp: f64,
    pub round_timings: IndexMap<u64, HashMap<String, TimingData>>,
    pub subdag_timings: IndexMap<(u64, u64), HashMap<String, TimingData>>,
}

impl ProcessedTimingData {
    /// Create a new empty processed timing data
    pub fn new(snapshot_timestamp: f64) -> Self {
        Self {
            snapshot_timestamp,
            round_timings: IndexMap::new(),
            subdag_timings: IndexMap::new(),
        }
    }

    /// Get the overall time range of all timing data
    pub fn get_time_range(&self) -> (f64, f64) {
        let mut all_times = Vec::new();

        // Collect all timestamps from round timings
        for round_data in self.round_timings.values() {
            for timing in round_data.values() {
                all_times.push(timing.start_time);
                if let Some(end_time) = timing.end_time {
                    all_times.push(end_time);
                }
            }
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

        // Add rounds from round timings
        for &round in self.round_timings.keys() {
            rounds.insert(round);
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
}

/// Parse raw timing data into processed format
pub fn parse_timing_data(raw: RawTimingSnapshot) -> Result<ProcessedTimingData> {
    let snapshot_timestamp = raw.timestamp.into();
    let mut processed = ProcessedTimingData::new(snapshot_timestamp);

    // Process round timings
    for (round_str, raw_round) in raw.round_timings {
        let round_num: u64 = round_str.parse()
            .with_context(|| format!("Failed to parse round number: {}", round_str))?;

        let mut round_data = HashMap::new();

        // Process each stage
        if let Some((start, end)) = raw_round.proposal_generation {
            let start_time: f64 = start.into();
            let end_time = end.map(|e| e.into());
            round_data.insert(
                "proposal_generation".to_string(),
                TimingData::new("proposal_generation".to_string(), start_time, end_time),
            );
        }

        if let Some((start, end)) = raw_round.certificate_generation {
            let start_time: f64 = start.into();
            let end_time = end.map(|e| e.into());
            round_data.insert(
                "certificate_generation".to_string(),
                TimingData::new("certificate_generation".to_string(), start_time, end_time),
            );
        }

        if let Some((start, end)) = raw_round.certificate_collection {
            let start_time: f64 = start.into();
            let end_time = end.map(|e| e.into());
            round_data.insert(
                "certificate_collection".to_string(),
                TimingData::new("certificate_collection".to_string(), start_time, end_time),
            );
        }

        if !round_data.is_empty() {
            processed.round_timings.insert(round_num, round_data);
        }
    }

    // Process subdag timings
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
            round_timings: {
                let mut map = HashMap::new();
                map.insert("100".to_string(), RawRoundTiming {
                    round: 100,
                    proposal_generation: Some((
                        JsonSystemTime { secs_since_epoch: 1640995200, nanos_since_epoch: 0 },
                        Some(JsonSystemTime { secs_since_epoch: 1640995201, nanos_since_epoch: 500_000_000 })
                    )),
                    certificate_generation: None,
                    certificate_collection: None,
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
        assert_eq!(processed.round_timings.len(), 1);
        assert_eq!(processed.subdag_timings.len(), 1);

        let round_100 = processed.round_timings.get(&100).unwrap();
        assert!(round_100.contains_key("proposal_generation"));
        
        let proposal_timing = round_100.get("proposal_generation").unwrap();
        assert_eq!(proposal_timing.start_time, 1640995200.0);
        assert_eq!(proposal_timing.end_time, Some(1640995201.5));
        assert!(proposal_timing.is_complete());
    }

    #[test]
    fn test_get_time_range() {
        let mut processed = ProcessedTimingData::new(1640995200.0);
        
        // Add some round timing data
        let mut round_data = HashMap::new();
        round_data.insert(
            "proposal_generation".to_string(),
            TimingData::new("proposal_generation".to_string(), 100.0, Some(101.5)),
        );
        processed.round_timings.insert(100, round_data);

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
        
        // Add round 100
        let mut round_data = HashMap::new();
        round_data.insert(
            "proposal_generation".to_string(),
            TimingData::new("proposal_generation".to_string(), 100.0, Some(101.5)),
        );
        processed.round_timings.insert(100, round_data);

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
}
