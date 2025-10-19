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

use std::{collections::HashMap, sync::Arc, time::SystemTime};

#[cfg(feature = "locktick")]
use locktick::parking_lot::RwLock;
#[cfg(not(feature = "locktick"))]
use parking_lot::RwLock;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

/// Global storage for round-based event data
static ROUND_EVENTS: Lazy<Arc<RwLock<HashMap<u64, RoundEvents>>>> = Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Global storage for subdag-based timing data (using lowest/highest rounds from subdag)
static SUBDAG_TIMINGS: Lazy<Arc<RwLock<HashMap<(u64, u64), SubdagTimings>>>> =
    Lazy::new(|| Arc::new(RwLock::new(HashMap::new())));

/// Consensus stages that occur per round
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusStage {
    ProposalSeen,
    ProposalCreated,
    CertificateAdded,
}

/// Block processing stages that occur per subdag
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubdagStage {
    SubdagProcessing,
    PrepareAdvanceToNextQuorumBlock,
    CheckNextBlock,
    AdvanceToNextBlock,
}

/// Timing event for a specific consensus stage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingEvent {
    pub round: u64,
    pub timestamp: SystemTime,
    pub event_type: String,
}

/// Timing data for round-based events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundEvents {
    pub round: u64,
    pub proposal_seen: Vec<TimingEvent>,
    pub proposal_created: Vec<TimingEvent>,
    pub certificate_added: Vec<TimingEvent>,
}

/// Timing data for a specific subdag (identified by lowest and highest rounds)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubdagTimings {
    pub lowest_round: u64,
    pub highest_round: u64,
    pub subdag_processing: Option<(SystemTime, Option<SystemTime>)>,
    pub prepare_advance_to_next_quorum_block: Option<(SystemTime, Option<SystemTime>)>,
    pub check_next_block: Option<(SystemTime, Option<SystemTime>)>,
    pub advance_to_next_block: Option<(SystemTime, Option<SystemTime>)>,
}

/// Combined timing data for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimingSnapshot {
    pub timestamp: SystemTime,
    pub round_events: HashMap<String, RoundEvents>,
    pub subdag_timings: HashMap<String, SubdagTimings>,
}

impl RoundEvents {
    fn new(round: u64) -> Self {
        Self { round, proposal_seen: Vec::new(), proposal_created: Vec::new(), certificate_added: Vec::new() }
    }

    fn add_event(&mut self, stage: ConsensusStage, timestamp: SystemTime) {
        let event = TimingEvent {
            round: self.round,
            timestamp,
            event_type: match stage {
                ConsensusStage::ProposalSeen => "proposal_seen".to_string(),
                ConsensusStage::ProposalCreated => "proposal_created".to_string(),
                ConsensusStage::CertificateAdded => "certificate_added".to_string(),
            },
        };

        match stage {
            ConsensusStage::ProposalSeen => self.proposal_seen.push(event),
            ConsensusStage::ProposalCreated => self.proposal_created.push(event),
            ConsensusStage::CertificateAdded => self.certificate_added.push(event),
        }
    }
}

impl SubdagTimings {
    fn new(lowest_round: u64, highest_round: u64) -> Self {
        Self {
            lowest_round,
            highest_round,
            subdag_processing: None,
            prepare_advance_to_next_quorum_block: None,
            check_next_block: None,
            advance_to_next_block: None,
        }
    }

    fn start_stage(&mut self, stage: SubdagStage) {
        let now = SystemTime::now();
        match stage {
            SubdagStage::SubdagProcessing => {
                self.subdag_processing = Some((now, None));
            }
            SubdagStage::PrepareAdvanceToNextQuorumBlock => {
                self.prepare_advance_to_next_quorum_block = Some((now, None));
            }
            SubdagStage::CheckNextBlock => {
                self.check_next_block = Some((now, None));
            }
            SubdagStage::AdvanceToNextBlock => {
                self.advance_to_next_block = Some((now, None));
            }
        }
    }

    fn end_stage(&mut self, stage: SubdagStage) {
        let now = SystemTime::now();
        match stage {
            SubdagStage::SubdagProcessing => {
                if let Some((start, _)) = self.subdag_processing {
                    self.subdag_processing = Some((start, Some(now)));
                }
            }
            SubdagStage::PrepareAdvanceToNextQuorumBlock => {
                if let Some((start, _)) = self.prepare_advance_to_next_quorum_block {
                    self.prepare_advance_to_next_quorum_block = Some((start, Some(now)));
                }
            }
            SubdagStage::CheckNextBlock => {
                if let Some((start, _)) = self.check_next_block {
                    self.check_next_block = Some((start, Some(now)));
                }
            }
            SubdagStage::AdvanceToNextBlock => {
                if let Some((start, _)) = self.advance_to_next_block {
                    self.advance_to_next_block = Some((start, Some(now)));
                }
            }
        }
    }
}

/// Record a consensus event for a specific round
pub fn record_event(round: u64, stage: ConsensusStage) {
    let now = SystemTime::now();
    let mut round_events = ROUND_EVENTS.write();
    let round_event = round_events.entry(round).or_insert_with(|| RoundEvents::new(round));
    round_event.add_event(stage, now);
}

/// Record a consensus event with custom timestamp for a specific round
pub fn record_event_with_timestamp(round: u64, stage: ConsensusStage, timestamp: SystemTime) {
    let mut events = ROUND_EVENTS.write();
    let round_events = events.entry(round).or_insert_with(|| RoundEvents::new(round));
    round_events.add_event(stage, timestamp);
}

/// Record the start of a subdag processing stage
pub fn start_subdag_stage(lowest_round: u64, highest_round: u64, stage: SubdagStage) {
    let key = (lowest_round, highest_round);
    let mut timings = SUBDAG_TIMINGS.write();
    let subdag_timing = timings.entry(key).or_insert_with(|| SubdagTimings::new(lowest_round, highest_round));
    subdag_timing.start_stage(stage);
}

/// Record the end of a subdag processing stage
pub fn end_subdag_stage(lowest_round: u64, highest_round: u64, stage: SubdagStage) {
    let key = (lowest_round, highest_round);
    let mut timings = SUBDAG_TIMINGS.write();
    if let Some(subdag_timing) = timings.get_mut(&key) {
        subdag_timing.end_stage(stage);
    }
}

/// Get event data for a specific round
pub fn get_round_events(round: u64) -> Option<RoundEvents> {
    ROUND_EVENTS.read().get(&round).cloned()
}

/// Get timing data for a specific subdag
pub fn get_subdag_timings(lowest_round: u64, highest_round: u64) -> Option<SubdagTimings> {
    let key = (lowest_round, highest_round);
    SUBDAG_TIMINGS.read().get(&key).cloned()
}

/// Get a snapshot of all current timing data
pub fn get_timing_snapshot() -> TimingSnapshot {
    let round_events = ROUND_EVENTS.read();
    let subdag_timings = SUBDAG_TIMINGS.read();

    // Convert keys to strings for JSON serialization
    let round_events_str: HashMap<String, RoundEvents> =
        round_events.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();

    let subdag_timings_str: HashMap<String, SubdagTimings> =
        subdag_timings.iter().map(|((low, high), v)| (format!("{low}-{high}"), v.clone())).collect();

    TimingSnapshot { timestamp: SystemTime::now(), round_events: round_events_str, subdag_timings: subdag_timings_str }
}

/// Export current timing state to a JSON file
pub fn export_to_json(file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let snapshot = get_timing_snapshot();
    let json = serde_json::to_string_pretty(&snapshot)?;
    std::fs::write(file_path, json)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{thread, time::Duration};

    #[test]
    fn test_event_recording() {
        let round = 12345;

        record_event(round, ConsensusStage::ProposalCreated);
        record_event(round, ConsensusStage::ProposalSeen);

        let events = get_round_events(round).unwrap();
        assert_eq!(events.proposal_created.len(), 1);
        assert_eq!(events.proposal_seen.len(), 1);
        assert_eq!(events.certificate_added.len(), 0);
    }

    #[test]
    fn test_subdag_timing() {
        let (low, high) = (100, 105);

        start_subdag_stage(low, high, SubdagStage::SubdagProcessing);
        thread::sleep(Duration::from_millis(10));
        end_subdag_stage(low, high, SubdagStage::SubdagProcessing);

        let timings = get_subdag_timings(low, high).unwrap();
        assert!(timings.subdag_processing.is_some());

        if let Some((start, Some(end))) = timings.subdag_processing {
            assert!(end > start);
        } else {
            panic!("Expected complete timing data");
        }
    }

    #[test]
    fn test_json_export() {
        let round = 54321;
        let (low, high) = (200, 210);

        // Add some timing data
        record_event(round, ConsensusStage::ProposalCreated);
        thread::sleep(Duration::from_millis(5));
        record_event(round, ConsensusStage::ProposalCreated);

        start_subdag_stage(low, high, SubdagStage::SubdagProcessing);
        thread::sleep(Duration::from_millis(5));
        end_subdag_stage(low, high, SubdagStage::SubdagProcessing);

        // Test JSON export
        let snapshot = get_timing_snapshot();
        let json_result = serde_json::to_string_pretty(&snapshot);
        assert!(json_result.is_ok(), "JSON serialization should succeed");

        let json_str = json_result.unwrap();
        assert!(json_str.contains(&round.to_string()), "JSON should contain round data");
        assert!(json_str.contains(&format!("{low}-{high}")), "JSON should contain subdag data");
    }
}
