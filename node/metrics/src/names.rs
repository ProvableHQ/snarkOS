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

/// What a metric is, and whether an unlabelled series should be created for it at startup.
///
/// The `Labeled*` variants are for metrics that are only ever recorded with labels, such as
/// [`consensus::TRANSMISSION_LATENCY`]. Their series are defined by their label values, so an
/// unlabelled one would be an artefact that nothing ever updates. They still carry a description:
/// `HELP` text belongs to the metric name, not to an individual series.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MetricKind {
    Counter,
    Gauge,
    Histogram,
    LabeledCounter,
    LabeledGauge,
    LabeledHistogram,
}

/// A declared metric: its name, what it is, and the `HELP` text published for it.
#[derive(Clone, Copy, Debug)]
pub struct MetricDef {
    /// The name the metric is exported under.
    pub name: &'static str,
    /// What the metric is, and whether it is pre-registered.
    pub kind: MetricKind,
    /// The metric's `HELP` text, taken from the doc comment on its declaration.
    ///
    /// Each line arrives with the leading space that `#[doc]` attributes carry, so this needs
    /// trimming before it is published.
    pub description: &'static str,
}

/// Declares the node's metrics, generating the name constants and [`ALL_METRICS`] from one source.
///
/// Each entry is a [`MetricKind`] variant, the constant to bind, and the exported metric name,
/// preceded by a doc comment:
///
/// ```ignore
/// declare_metrics! {
///     pub mod bft {
///         /// The number of validators this node is connected to.
///         Gauge CONNECTED = "snarkos_bft_connected_total";
///     }
/// }
/// ```
///
/// The doc comment becomes the metric's `HELP` text in the Prometheus exposition, and it is
/// mandatory: a declaration without one fails to compile, so no metric can reach the exposition
/// undescribed.
macro_rules! declare_metrics {
    ($(
        $(#[$module_attr:meta])*
        pub mod $module:ident {
            $(
                $(#[doc = $description:literal])*
                $kind:ident $constant:ident = $name:literal;
            )*
        }
    )*) => {
        $(
            $(#[$module_attr])*
            pub mod $module {
                $(
                    $(#[doc = $description])*
                    pub const $constant: &str = $name;
                )*
            }
        )*

        $($(
            const _: () = assert!(
                !concat!($($description),*).is_empty(),
                concat!(
                    "the metric `",
                    stringify!($constant),
                    "` has no doc comment, so it has no HELP text to publish",
                ),
            );
        )*)*

        /// Every declared metric, in declaration order.
        pub(super) const ALL_METRICS: &[MetricDef] = &[
            $($(
                MetricDef {
                    name: $name,
                    kind: MetricKind::$kind,
                    description: concat!($($description),*),
                },
            )*)*
        ];
    };
}

declare_metrics! {
    pub mod bft {
        /// Seconds from proposing a batch to that batch being certified.
        Histogram BATCH_CERTIFICATION_LATENCY = "snarkos_bft_batch_certification_latency_secs";
        /// Seconds spent committing a leader certificate and the subdag beneath it.
        Histogram COMMIT_LEADER_CERTIFICATE_LATENCY = "snarkos_bft_commit_leader_certificate_latency_secs";
        /// Seconds a round waited, from the leader certificate timer being set, until the round
        /// was ready to advance.
        Histogram COMMIT_ROUNDS_LATENCY = "snarkos_bft_commit_rounds_latency_secs";
        /// The number of validators this node is connected to, excluding bootstrap clients.
        Gauge CONNECTED = "snarkos_bft_connected_total";
        /// The percentage of the committee's stake this node is connected to.
        Gauge CONNECTED_STAKE = "snarkos_bft_connected_stake_as_percentage";
        /// The percentage of the committee's stake this node is connected to that reports the
        /// same build commit hash as this node.
        Gauge CONNECTED_STAKE_WITH_MATCHING_SHA = "snarkos_bft_connected_stake_with_matching_sha_as_percentage";
        /// The number of peers this node is currently dialling, excluding bootstrap clients.
        Gauge CONNECTING = "snarkos_bft_connecting_total";
        /// The round that BFT storage was most recently advanced to.
        Gauge LAST_STORED_ROUND = "snarkos_bft_last_stored_round";
        /// The number of leaders this node has elected.
        Counter LEADERS_ELECTED = "snarkos_bft_leaders_elected_total";
        /// The round of the batch this node most recently proposed.
        Gauge PROPOSAL_ROUND = "snarkos_bft_primary_proposal_round";
        /// The number of this node's own batches that have been certified since it started.
        Gauge CERTIFIED_BATCHES = "snarkos_bft_primary_certified_batches";
        /// The height of the latest block in the ledger.
        Gauge HEIGHT = "snarkos_bft_height_total";
        /// The round of the latest block in the ledger.
        Gauge LAST_COMMITTED_ROUND = "snarkos_bft_last_committed_round";
        /// Whether this node considers itself synced with the network: 1 if synced, 0 otherwise.
        Gauge IS_SYNCED = "snarkos_bft_is_synced";
    }

    pub mod blocks {
        /// The number of transactions in the blocks this node has added since it started.
        Gauge TRANSACTIONS = "snarkos_blocks_transactions_total";
        /// The number of solutions in the blocks this node has added since it started.
        Gauge SOLUTIONS = "snarkos_blocks_solutions_total";
        /// The number of accepted deployment transactions in the blocks this node has added
        /// since it started.
        Gauge ACCEPTED_DEPLOY = "snarkos_blocks_accepted_deploy";
        /// The number of accepted execution transactions in the blocks this node has added since
        /// it started.
        Gauge ACCEPTED_EXECUTE = "snarkos_blocks_accepted_execute";
        /// The number of rejected deployment transactions in the blocks this node has added since
        /// it started.
        Gauge REJECTED_DEPLOY = "snarkos_blocks_rejected_deploy";
        /// The number of rejected execution transactions in the blocks this node has added since
        /// it started.
        Gauge REJECTED_EXECUTE = "snarkos_blocks_rejected_execute";
        /// The number of aborted transactions in the blocks this node has added since it started.
        Gauge ABORTED_TRANSACTIONS = "snarkos_blocks_aborted_transactions";
        /// The number of aborted solutions in the blocks this node has added since it started.
        Gauge ABORTED_SOLUTIONS = "snarkos_blocks_aborted_solutions";
        /// The proof target of the latest block.
        Gauge PROOF_TARGET = "snarkos_blocks_proof_target";
        /// The coinbase target of the latest block.
        Gauge COINBASE_TARGET = "snarkos_blocks_coinbase_target";
        /// The cumulative proof target of the latest block.
        Gauge CUMULATIVE_PROOF_TARGET = "snarkos_blocks_cumulative_proof_target";
    }

    pub mod consensus {
        /// Never recorded. Superseded by `snarkos_consensus_advance_to_next_block_secs`.
        Histogram ADVANCE_TO_NEXT_BLOCK_LATENCY = "snarkos_consensus_advance_to_next_block_latency_secs";
        /// Never recorded. Superseded by `snarkos_consensus_check_next_block_secs`.
        Histogram CHECK_NEXT_BLOCK_LATENCY = "snarkos_consensus_check_next_block_latency_secs";
        /// Never recorded. Superseded by `snarkos_consensus_prepare_advance_secs`.
        Histogram PREPARE_ADVANCE_TO_NEXT_QUORUM_BLOCK_LATENCY =
            "snarkos_consensus_prepare_advance_to_next_quorum_block_latency_secs";
        /// Seconds spent committing the subdag of a block, from the start of the commit to the
        /// block being added to the ledger.
        Histogram CERTIFICATE_COMMIT_LATENCY = "snarkos_consensus_certificate_commit_latency_secs";
        /// The number of certificates committed in the subdag of the latest block.
        Gauge COMMITTED_CERTIFICATES = "snarkos_consensus_committed_certificates_total";
        /// Seconds between the timestamp of the latest block and that of the block before it.
        Histogram BLOCK_LATENCY = "snarkos_consensus_block_latency_secs";
        /// Milliseconds between the timestamp of the latest block and this node's clock when it
        /// added that block.
        Histogram BLOCK_LAG = "snarkos_consensus_block_lag_ms";
        /// Seconds spent in prepare_advance_to_next_quorum_block, i.e. constructing the block.
        Histogram PREPARE_ADVANCE_SECS = "snarkos_consensus_prepare_advance_secs";
        /// Seconds spent in check_next_block, i.e. verifying the constructed block.
        Histogram CHECK_NEXT_BLOCK_SECS = "snarkos_consensus_check_next_block_secs";
        /// Seconds spent in advance_to_next_block, i.e. writing the block to the ledger.
        Histogram ADVANCE_TO_NEXT_BLOCK_SECS = "snarkos_consensus_advance_to_next_block_secs";
        /// The number of unconfirmed transactions this node has accepted into its mempool since
        /// it started.
        Gauge UNCONFIRMED_TRANSACTIONS = "snarkos_consensus_unconfirmed_transactions_total";
        /// The number of unconfirmed solutions this node has accepted into its mempool since it
        /// started.
        Gauge UNCONFIRMED_SOLUTIONS = "snarkos_consensus_unconfirmed_solutions_total";
        /// Seconds from a transmission first being seen to the block confirming it, labelled by
        /// `transmission_type`.
        LabeledHistogram TRANSMISSION_LATENCY = "snarkos_consensus_transmission_latency";
        /// The number of tracked transactions discarded for going unconfirmed past the staleness
        /// threshold.
        Counter STALE_UNCONFIRMED_TRANSACTIONS = "snarkos_consensus_stale_unconfirmed_transactions";
        /// The number of tracked solutions discarded for going unconfirmed past the staleness
        /// threshold.
        Counter STALE_UNCONFIRMED_SOLUTIONS = "snarkos_consensus_stale_unconfirmed_solutions";
        /// The percentage of recent rounds in which a validator had a certificate, labelled by
        /// `validator_address`.
        LabeledGauge VALIDATOR_CERTIFICATE_PARTICIPATION = "snarkos_consensus_validator_certificate_participation";
        /// The percentage of recent certificates a validator signed, labelled by
        /// `validator_address`.
        LabeledGauge VALIDATOR_SIGNATURE_PARTICIPATION = "snarkos_consensus_validator_signature_participation";
        /// The garbage collection round the published participation scores were computed at.
        Gauge VALIDATOR_PARTICIPATION_GC_ROUND = "snarkos_consensus_validator_participation_gc_round";
        /// The number of telemetry updates dropped because the worker queue was full.
        Gauge VALIDATOR_PARTICIPATION_DROPPED = "snarkos_consensus_validator_participation_dropped_total";
    }

    pub mod router {
        /// The number of peers the router is connected to.
        Gauge CONNECTED = "snarkos_router_connected_total";
        /// The number of peers the router may attempt to connect to.
        Gauge CANDIDATE = "snarkos_router_candidate_total";
        /// Never recorded. The router no longer tracks restricted peers.
        Gauge RESTRICTED = "snarkos_router_restricted_total";
    }

    pub mod tcp {
        /// The number of inbound messages that have been read off a socket and are waiting to be
        /// processed, across all connections.
        Gauge QUEUED_INBOUND_MESSAGES = "snarkos_tcp_queued_inbound_messages";
    }

    pub mod build {
        /// Always 1. Carries this node's build details as labels: `version`, `git_commit`,
        /// `git_branch` and `features`.
        LabeledGauge BUILD_INFO = "snarkos_build_info";
    }
}
