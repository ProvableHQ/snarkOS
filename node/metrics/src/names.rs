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

pub(super) const COUNTER_NAMES: [&str; 3] =
    [bft::LEADERS_ELECTED, consensus::STALE_UNCONFIRMED_TRANSACTIONS, consensus::STALE_UNCONFIRMED_SOLUTIONS];

pub(super) const GAUGE_NAMES: [&str; 35] = [
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
    tcp::QUEUED_INBOUND_MESSAGES,
    rayon_pool::THREADS,
    tokio_rt::WORKERS,
    tokio_rt::ALIVE_TASKS,
    tokio_rt::GLOBAL_QUEUE_DEPTH,
    tokio_rt::WORKER_BUSY_CORES,
    tokio_rt::WORKER_BUSY_RATIO_MAX,
    tokio_rt::WORKER_PARKS,
];

pub(super) const HISTOGRAM_NAMES: [&str; 10] = [
    bft::COMMIT_ROUNDS_LATENCY,
    bft::COMMIT_LEADER_CERTIFICATE_LATENCY,
    bft::BATCH_CERTIFICATION_LATENCY,
    consensus::CERTIFICATE_COMMIT_LATENCY,
    consensus::BLOCK_LATENCY,
    consensus::BLOCK_LAG,
    consensus::PREPARE_ADVANCE_TO_NEXT_QUORUM_BLOCK_LATENCY,
    consensus::CHECK_NEXT_BLOCK_LATENCY,
    consensus::ADVANCE_TO_NEXT_BLOCK_LATENCY,
    cpu::BLOCKING_WAIT_SECS,
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

pub mod cpu {
    /// Time a blocking task waited for a thread in the `tokio` blocking pool, aggregated over
    /// every `spawn_blocking!` call site.
    ///
    /// CPU-bound work reaches a core by way of that pool, so a non-zero wait here means work is
    /// queueing before it ever reaches rayon.
    pub const BLOCKING_WAIT_SECS: &str = "snarkos_cpu_blocking_wait_secs";
}

pub mod router {
    pub const CONNECTED: &str = "snarkos_router_connected_total";
    pub const CANDIDATE: &str = "snarkos_router_candidate_total";
    pub const RESTRICTED: &str = "snarkos_router_restricted_total";
}

pub mod tcp {
    /// The number of inbound messages that have been read off a socket and are waiting to be
    /// processed, across all connections.
    pub const QUEUED_INBOUND_MESSAGES: &str = "snarkos_tcp_queued_inbound_messages";
}

pub mod build {
    pub const BUILD_INFO: &str = "snarkos_build_info";
}

pub mod rayon_pool {
    /// Threads in the global rayon pool.
    ///
    /// The node sizes this pool from the core count the cgroup quota allows, which need not be the
    /// host's core count, so read the width here rather than inferring it from the machine.
    pub const THREADS: &str = "snarkos_rayon_threads";
}

/// Counters the `tokio` runtime keeps about itself, sampled into gauges.
///
/// The runtime reports monotonic totals; the series below that describe a rate hold the change over
/// one sampling interval, not the total.
pub mod tokio_rt {
    /// Worker threads the runtime was built with.
    pub const WORKERS: &str = "snarkos_tokio_workers";
    /// Tasks the runtime is currently tracking, whether running or waiting.
    pub const ALIVE_TASKS: &str = "snarkos_tokio_alive_tasks";
    /// Tasks sitting in the runtime's global injector queue, having found no worker to take them.
    pub const GLOBAL_QUEUE_DEPTH: &str = "snarkos_tokio_global_queue_depth";
    /// Cores' worth of work the worker threads got through, summed over every worker.
    ///
    /// This is the figure that sizes `worker_threads`: a value far below the worker count means the
    /// runtime is oversubscribed relative to the work it has.
    pub const WORKER_BUSY_CORES: &str = "snarkos_tokio_worker_busy_cores";
    /// Utilization of the busiest single worker, from 0 to 1.
    ///
    /// Read against `WORKER_BUSY_CORES` divided by `WORKERS`: a large gap means load is landing on
    /// a few workers rather than spreading.
    pub const WORKER_BUSY_RATIO_MAX: &str = "snarkos_tokio_worker_busy_ratio_max";
    /// Times workers went to sleep for want of work, summed over every worker.
    pub const WORKER_PARKS: &str = "snarkos_tokio_worker_parks";
    /// Threads alive in the blocking pool, idle ones included.
    ///
    /// Only reported when the node is built with `RUSTFLAGS="--cfg tokio_unstable"`.
    pub const BLOCKING_THREADS: &str = "snarkos_tokio_blocking_threads";
    /// Threads in the blocking pool waiting for a task.
    ///
    /// Only reported when the node is built with `RUSTFLAGS="--cfg tokio_unstable"`.
    pub const BLOCKING_THREADS_IDLE: &str = "snarkos_tokio_blocking_threads_idle";
    /// Tasks queued for the blocking pool because every thread is taken and the pool is at its cap.
    ///
    /// Only reported when the node is built with `RUSTFLAGS="--cfg tokio_unstable"`.
    pub const BLOCKING_QUEUE_DEPTH: &str = "snarkos_tokio_blocking_queue_depth";
    /// Tasks workers stole from each other, summed over every worker.
    ///
    /// Only reported when the node is built with `RUSTFLAGS="--cfg tokio_unstable"`.
    pub const WORKER_STEALS: &str = "snarkos_tokio_worker_steals";
    /// Times a worker woke up, found nothing to run, and parked again, summed over every worker.
    ///
    /// Only reported when the node is built with `RUSTFLAGS="--cfg tokio_unstable"`.
    pub const WORKER_NOOPS: &str = "snarkos_tokio_worker_noops";
}
