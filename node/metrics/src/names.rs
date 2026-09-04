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

pub(super) const COUNTER_NAMES: [&str; 5] = [
    bft::LEADERS_ELECTED,
    consensus::STALE_UNCONFIRMED_TRANSACTIONS,
    consensus::STALE_UNCONFIRMED_SOLUTIONS,
    tcp::WRITE_TIMEOUT_DISCONNECTS,
    tcp::WRITE_ERROR_DISCONNECTS,
];

pub(super) const GAUGE_NAMES: [&str; 33] = [
    bft::CONNECTED,
    bft::CONNECTED_STAKE,
    bft::CONNECTED_STAKE_WITH_MATCHING_SHA,
    bft::CONNECTING,
    bft::LAST_STORED_ROUND,
    bft::PROPOSAL_ROUND,
    bft::CERTIFIED_BATCHES,
    bft::HEIGHT,
    bft::LAST_COMMITTED_ROUND,
    bft::IS_SYNCED,
    bft::GATEWAY_OUTBOUND_QUEUE_CAPACITY,
    bft::GATEWAY_OUTBOUND_QUEUE_DEPTH_MAX,
    bft::GATEWAY_OUTBOUND_QUEUE_HIGH_WATER_MARK,
    bft::GATEWAY_SENDS_IN_FLIGHT,
    bft::GATEWAY_SENDS_RATE_LIMITED,
    blocks::SOLUTIONS,
    blocks::TRANSACTIONS,
    blocks::ACCEPTED_DEPLOY,
    blocks::ACCEPTED_EXECUTE,
    blocks::REJECTED_DEPLOY,
    blocks::REJECTED_EXECUTE,
    blocks::ABORTED_TRANSACTIONS,
    blocks::ABORTED_SOLUTIONS,
    blocks::PROOF_TARGET,
    blocks::COINBASE_TARGET,
    blocks::CUMULATIVE_PROOF_TARGET,
    consensus::COMMITTED_CERTIFICATES,
    consensus::UNCONFIRMED_SOLUTIONS,
    consensus::UNCONFIRMED_TRANSACTIONS,
    router::CONNECTED,
    router::CANDIDATE,
    router::RESTRICTED,
    tcp::TCP_TASKS,
];

pub(super) const HISTOGRAM_NAMES: [&str; 10] = [
    bft::COMMIT_ROUNDS_LATENCY,
    bft::COMMIT_LEADER_CERTIFICATE_LATENCY,
    bft::BATCH_CERTIFICATION_LATENCY,
    bft::GATEWAY_OUTBOUND_QUEUE_DEPTH,
    consensus::CERTIFICATE_COMMIT_LATENCY,
    consensus::BLOCK_LATENCY,
    consensus::BLOCK_LAG,
    consensus::PREPARE_ADVANCE_TO_NEXT_QUORUM_BLOCK_LATENCY,
    consensus::CHECK_NEXT_BLOCK_LATENCY,
    consensus::ADVANCE_TO_NEXT_BLOCK_LATENCY,
];

pub mod bft {
    pub const BATCH_CERTIFICATION_LATENCY: &str = "snarkos_bft_batch_certification_latency_secs";
    pub const COMMIT_LEADER_CERTIFICATE_LATENCY: &str = "snarkos_bft_commit_leader_certificate_latency_secs";
    pub const COMMIT_ROUNDS_LATENCY: &str = "snarkos_bft_commit_rounds_latency_secs"; // <-- This one doesn't even make sense.
    pub const CONNECTED: &str = "snarkos_bft_connected_total";
    pub const CONNECTED_STAKE: &str = "snarkos_bft_connected_stake_as_percentage";
    pub const CONNECTED_STAKE_WITH_MATCHING_SHA: &str = "snarkos_bft_connected_stake_with_matching_sha_as_percentage";
    pub const CONNECTING: &str = "snarkos_bft_connecting_total";
    pub const LAST_STORED_ROUND: &str = "snarkos_bft_last_stored_round";
    pub const LEADERS_ELECTED: &str = "snarkos_bft_leaders_elected_total";
    pub const PROPOSAL_ROUND: &str = "snarkos_bft_primary_proposal_round";
    pub const CERTIFIED_BATCHES: &str = "snarkos_bft_primary_certified_batches";
    pub const HEIGHT: &str = "snarkos_bft_height_total";
    pub const LAST_COMMITTED_ROUND: &str = "snarkos_bft_last_committed_round";
    pub const IS_SYNCED: &str = "snarkos_bft_is_synced";

    /// The depth of the per-connection queue used to send outbound messages, sampled once per
    /// peer per metrics tick.
    pub const GATEWAY_OUTBOUND_QUEUE_DEPTH: &str = "snarkos_bft_gateway_outbound_queue_depth";
    /// The deepest per-connection outbound queue at the most recent metrics tick.
    pub const GATEWAY_OUTBOUND_QUEUE_DEPTH_MAX: &str = "snarkos_bft_gateway_outbound_queue_depth_max";
    /// The deepest per-connection outbound queue observed since the node started, measured at
    /// every send rather than sampled.
    pub const GATEWAY_OUTBOUND_QUEUE_HIGH_WATER_MARK: &str = "snarkos_bft_gateway_outbound_queue_high_water_mark";
    /// The capacity of each per-connection outbound queue, i.e. the depth at which a send fails
    /// and the peer is disconnected.
    pub const GATEWAY_OUTBOUND_QUEUE_CAPACITY: &str = "snarkos_bft_gateway_outbound_queue_capacity";
    /// Outbound sends that could not be queued, labelled by `reason`. The `queue_full` label is
    /// the per-connection outbound queue being at capacity.
    pub const GATEWAY_SEND_FAILURES: &str = "snarkos_bft_gateway_send_failures_total";
    /// Outbound sends that entered the rate limiter's retry loop, labelled by `limit`.
    pub const GATEWAY_RATE_LIMITED_SENDS: &str = "snarkos_bft_gateway_rate_limited_sends_total";
    /// Iterations of the rate limiter's retry loop, labelled by `limit`. Each iteration is one
    /// backoff sleep.
    pub const GATEWAY_RATE_LIMIT_SLEEPS: &str = "snarkos_bft_gateway_rate_limit_sleeps_total";
    /// Time an outbound send spent in the rate limiter's retry loop, labelled by `limit`.
    pub const GATEWAY_RATE_LIMIT_DELAY: &str = "snarkos_bft_gateway_rate_limit_delay_secs";
    /// Outbound sends currently in `Transport::send`. Nearly every call site spawns a task per
    /// send, so this is also the number of outstanding send tasks.
    pub const GATEWAY_SENDS_IN_FLIGHT: &str = "snarkos_bft_gateway_sends_in_flight";
    /// Outbound sends currently parked in the rate limiter's retry loop.
    pub const GATEWAY_SENDS_RATE_LIMITED: &str = "snarkos_bft_gateway_sends_rate_limited";
}

pub mod blocks {
    pub const TRANSACTIONS: &str = "snarkos_blocks_transactions_total";
    pub const SOLUTIONS: &str = "snarkos_blocks_solutions_total";
    pub const ACCEPTED_DEPLOY: &str = "snarkos_blocks_accepted_deploy";
    pub const ACCEPTED_EXECUTE: &str = "snarkos_blocks_accepted_execute";
    pub const REJECTED_DEPLOY: &str = "snarkos_blocks_rejected_deploy";
    pub const REJECTED_EXECUTE: &str = "snarkos_blocks_rejected_execute";
    pub const ABORTED_TRANSACTIONS: &str = "snarkos_blocks_aborted_transactions";
    pub const ABORTED_SOLUTIONS: &str = "snarkos_blocks_aborted_solutions";
    pub const PROOF_TARGET: &str = "snarkos_blocks_proof_target";
    pub const COINBASE_TARGET: &str = "snarkos_blocks_coinbase_target";
    pub const CUMULATIVE_PROOF_TARGET: &str = "snarkos_blocks_cumulative_proof_target";
}

pub mod consensus {
    pub const ADVANCE_TO_NEXT_BLOCK_LATENCY: &str = "snarkos_consensus_advance_to_next_block_latency_secs";
    pub const CHECK_NEXT_BLOCK_LATENCY: &str = "snarkos_consensus_check_next_block_latency_secs";
    pub const PREPARE_ADVANCE_TO_NEXT_QUORUM_BLOCK_LATENCY: &str =
        "snarkos_consensus_prepare_advance_to_next_quorum_block_latency_secs";
    pub const CERTIFICATE_COMMIT_LATENCY: &str = "snarkos_consensus_certificate_commit_latency_secs";
    pub const COMMITTED_CERTIFICATES: &str = "snarkos_consensus_committed_certificates_total";
    pub const BLOCK_LATENCY: &str = "snarkos_consensus_block_latency_secs";
    pub const BLOCK_LAG: &str = "snarkos_consensus_block_lag_ms";
    /// Time spent in prepare_advance_to_next_quorum_block (block construction).
    pub const PREPARE_ADVANCE_SECS: &str = "snarkos_consensus_prepare_advance_secs";
    /// Time spent in check_next_block.
    pub const CHECK_NEXT_BLOCK_SECS: &str = "snarkos_consensus_check_next_block_secs";
    /// Time spent in advance_to_next_block (ledger write).
    pub const ADVANCE_TO_NEXT_BLOCK_SECS: &str = "snarkos_consensus_advance_to_next_block_secs";
    pub const UNCONFIRMED_TRANSACTIONS: &str = "snarkos_consensus_unconfirmed_transactions_total";
    pub const UNCONFIRMED_SOLUTIONS: &str = "snarkos_consensus_unconfirmed_solutions_total";
    pub const TRANSMISSION_LATENCY: &str = "snarkos_consensus_transmission_latency";
    pub const STALE_UNCONFIRMED_TRANSACTIONS: &str = "snarkos_consensus_stale_unconfirmed_transactions";
    pub const STALE_UNCONFIRMED_SOLUTIONS: &str = "snarkos_consensus_stale_unconfirmed_solutions";
    pub const VALIDATOR_CERTIFICATE_PARTICIPATION: &str = "snarkos_consensus_validator_certificate_participation";
    pub const VALIDATOR_SIGNATURE_PARTICIPATION: &str = "snarkos_consensus_validator_signature_participation";
    /// The garbage collection round the published participation scores were computed at.
    pub const VALIDATOR_PARTICIPATION_GC_ROUND: &str = "snarkos_consensus_validator_participation_gc_round";
    /// The number of telemetry updates dropped because the worker queue was full.
    pub const VALIDATOR_PARTICIPATION_DROPPED: &str = "snarkos_consensus_validator_participation_dropped_total";
}

pub mod router {
    pub const CONNECTED: &str = "snarkos_router_connected_total";
    pub const CANDIDATE: &str = "snarkos_router_candidate_total";
    pub const RESTRICTED: &str = "snarkos_router_restricted_total";
}

pub mod tcp {
    pub const TCP_TASKS: &str = "snarkos_tcp_tasks_total";
    /// Connections dropped because a single write to the socket exceeded `Writing::TIMEOUT`.
    pub const WRITE_TIMEOUT_DISCONNECTS: &str = "snarkos_tcp_write_timeout_disconnects_total";
    /// Connections dropped because a write to the socket failed for any other reason.
    pub const WRITE_ERROR_DISCONNECTS: &str = "snarkos_tcp_write_error_disconnects_total";
}

pub mod build {
    pub const BUILD_INFO: &str = "snarkos_build_info";
}
