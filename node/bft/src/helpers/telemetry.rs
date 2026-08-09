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

//! Validator participation telemetry.
//!
//! Telemetry is observational: the participation scores computed here feed the
//! gateway's heartbeat log, the consensus metrics gauges, and a REST endpoint.
//! Nothing in consensus reads them. This module is structured around that fact.
//!
//! The state is owned by a single background task ([`TelemetryWorker`]) rather
//! than shared behind locks, which gives three properties:
//!
//! - The BFT path can never block on telemetry. Enqueueing an update is
//!   wait-free; if the worker were to fall behind, updates are dropped and a
//!   warning is logged, degrading a metric rather than a consensus decision.
//! - Telemetry can never deadlock against a `rayon` parallel iterator. The
//!   previous lock-based implementation carried a doc comment forbidding its
//!   locks from being touched inside (or across) a parallel iterator, enforced
//!   only by review. There is now no lock for a parallel iterator to block on.
//! - Recomputing the participation scores, which is `O(validators * rounds)`,
//!   is coalesced: the worker drains its whole queue before recomputing once.
//!
//! Readers observe the most recently published snapshot through a
//! [`tokio::sync::watch`] channel. That channel does have an internal lock, but
//! it is held only long enough to clone an `Arc`, there is exactly one writer,
//! and no other lock is ever acquired underneath it, so no cycle can form.

use snarkvm::{
    ledger::{
        committee::Committee,
        narwhal::{BatchCertificate, BatchHeader, Subdag},
    },
    prelude::{Address, Field, Network, cfg_iter},
};

use indexmap::{IndexMap, IndexSet};
#[cfg(not(feature = "serial"))]
use rayon::prelude::*;
use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::sync::{
    mpsc::{self, error::TrySendError},
    oneshot,
    watch,
};

// TODO: Consider other metrics to track:
//  - Response time
//  - Sync rate
//  - Latest height of each validator
//  - Percentage of proposals that are converted into certificates
//  - Fullness of proposals
//  - Connectivity (how many other validators are they connected to)
//  - Various stake weight considerations
//  - The latest seen IP address of each validator (useful for debugging purposes)

/// The capacity of the telemetry update queue.
///
/// Each entry holds one subdag's worth of certificate metadata. The worker's only
/// job is to fold these into a few maps, so it drains far faster than the BFT path
/// can produce. This bound exists to cap memory in the pathological case, such as a
/// long catch-up sync, not because a backlog is expected in steady state.
const TELEMETRY_QUEUE_CAPACITY: usize = 1024;

/// The participation scores for each validator.
///     Certificate Score: The % of rounds the validator has a valid certificate
///     Signature Score: The % of certificates the validator has a valid signature for
///     Combined Score: The weighted score using the certificate and signature scores
type ParticipationScores = (f64, f64, f64);

/// A published snapshot of the participation scores for every tracked validator.
type ScoreSnapshot<N> = Arc<IndexMap<Address<N>, ParticipationScores>>;

/// The metadata of a certificate that the telemetry tracker keeps track of.
///
/// This is derived on the caller's thread, before the update is enqueued, so that the
/// (relatively expensive) recovery of the signer addresses can be done in parallel.
#[derive(Clone, Debug)]
pub struct CertificateMetadata<N: Network> {
    /// The round of the certificate.
    round: u64,
    /// The ID of the certificate.
    id: Field<N>,
    /// The author of the certificate.
    author: Address<N>,
    /// The author of the certificate, followed by the address of each of its signers.
    signers: Vec<Address<N>>,
}

impl<N: Network> CertificateMetadata<N> {
    /// Derives the telemetry metadata for the given certificate.
    fn new(certificate: &BatchCertificate<N>) -> Self {
        let author = certificate.author();
        let signers = [author]
            .into_iter()
            .chain(certificate.signatures().map(|signature| signature.to_address()))
            .collect::<Vec<_>>();

        Self { round: certificate.round(), id: certificate.id(), author, signers }
    }
}

/// An update sent from the BFT path to the telemetry worker.
#[derive(Debug)]
enum TelemetryUpdate<N: Network> {
    /// Garbage collect below `gc_round`, then insert the metadata of a subdag's certificates.
    ///
    /// The GC round is computed by the sender, so that the worker does not need to know
    /// anything about the subdag itself.
    Subdag { gc_round: u64, metadata: Vec<CertificateMetadata<N>> },
    /// Insert the metadata of a single certificate.
    Certificate(Box<CertificateMetadata<N>>),
    /// Acknowledge once every previously enqueued update has been applied and published.
    Flush(oneshot::Sender<()>),
}

/// A handle to the validator telemetry tracker.
///
/// This is a cheap `Clone`: it holds a queue sender and a snapshot receiver. All
/// telemetry state lives in the [`TelemetryWorker`] task, so no method on this type
/// can block the caller behind another telemetry operation.
#[derive(Clone, Debug)]
pub struct Telemetry<N: Network> {
    /// Sends updates to the telemetry worker.
    sender: mpsc::Sender<TelemetryUpdate<N>>,
    /// Receives the latest published participation scores.
    scores: watch::Receiver<ScoreSnapshot<N>>,
    /// The running total of updates dropped because the queue was full.
    num_dropped: Arc<AtomicU64>,
}

impl<N: Network> Telemetry<N> {
    /// Initializes the telemetry tracker, returning the shared handle and its worker.
    ///
    /// The caller is responsible for spawning [`TelemetryWorker::run`]. Note that this
    /// deliberately does not spawn the task itself, so that constructing a telemetry
    /// tracker does not require an active Tokio runtime.
    pub fn new() -> (Self, TelemetryWorker<N>) {
        let (sender, receiver) = mpsc::channel(TELEMETRY_QUEUE_CAPACITY);
        let (score_sender, score_receiver) = watch::channel::<ScoreSnapshot<N>>(Default::default());

        let telemetry = Self { sender, scores: score_receiver, num_dropped: Default::default() };
        let worker = TelemetryWorker { receiver, scores: score_sender, state: TelemetryState::new() };

        (telemetry, worker)
    }

    /// Insert a subdag into the telemetry tracker.
    /// Note: This currently assumes the subdag is fully formed and included in the block.
    ///
    /// This never blocks. If the worker is behind, the update is dropped and a warning is
    /// logged; see the module documentation for why that is acceptable here.
    pub fn insert_subdag(&self, subdag: &Subdag<N>) {
        // Determine the round to garbage collect below.
        let gc_round = subdag.anchor_round().saturating_sub(BatchHeader::<N>::MAX_GC_ROUNDS as u64);

        // Derive the metadata of each certificate in parallel, before enqueueing the update.
        // Recovering the address of a signer is expensive, so this is the bulk of the work here.
        let certificates: Vec<_> = subdag.values().flatten().collect();
        let metadata: Vec<_> =
            cfg_iter!(certificates).map(|certificate| CertificateMetadata::new(certificate)).collect();

        self.enqueue(TelemetryUpdate::Subdag { gc_round, metadata });
    }

    /// Insert a certificate into the telemetry tracker.
    pub fn insert_certificate(&self, certificate: &BatchCertificate<N>) {
        self.enqueue(TelemetryUpdate::Certificate(Box::new(CertificateMetadata::new(certificate))));
    }

    /// Fetch the certificate and signature participation scores for each validator in the committee set.
    /// Returns a map of `address` to `(certificate_score, signature_score)`.
    ///
    /// This reads the most recently published snapshot, and never blocks a writer.
    pub fn get_participation_scores(&self, committee: &Committee<N>) -> IndexMap<Address<N>, (f64, f64)> {
        // Clone the Arc out of the watch channel, then drop the borrow immediately, so that
        // the projection below happens outside of it.
        let snapshot: ScoreSnapshot<N> = self.scores.borrow().clone();

        scores_for_committee(&snapshot, committee)
    }

    /// Returns the number of telemetry updates dropped so far because the queue was full.
    ///
    /// A nonzero and growing value means the published scores are missing rounds, which is
    /// the one way this design can be visibly wrong.
    pub fn num_dropped(&self) -> u64 {
        self.num_dropped.load(Ordering::Relaxed)
    }

    /// Waits until every previously enqueued update has been applied and published.
    ///
    /// Returns an error if the worker is not running. Intended for tests and for
    /// deterministic shutdown; the production paths are all fire-and-forget.
    pub async fn flush(&self) -> Result<(), ()> {
        let (sender, receiver) = oneshot::channel();
        // Use `send` rather than `try_send` here: a flush must not be dropped on a full queue.
        self.sender.send(TelemetryUpdate::Flush(sender)).await.map_err(|_| ())?;
        receiver.await.map_err(|_| ())
    }

    /// Enqueues an update for the worker, dropping it if the queue is full.
    fn enqueue(&self, update: TelemetryUpdate<N>) {
        match self.sender.try_send(update) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                let num_dropped = self.num_dropped.fetch_add(1, Ordering::Relaxed).saturating_add(1);
                warn!("Telemetry queue is full - dropping an update ({num_dropped} dropped in total)");
            }
            Err(TrySendError::Closed(_)) => {
                // The worker has stopped; this is expected during shutdown.
                trace!("Telemetry worker is not running - dropping an update");
            }
        }
    }
}

/// The background task that owns all telemetry state.
///
/// There is exactly one of these, it is the only thing that touches [`TelemetryState`],
/// and it holds no locks, so it can neither deadlock against BFT nor block it.
#[derive(Debug)]
pub struct TelemetryWorker<N: Network> {
    /// Receives updates from the BFT path.
    receiver: mpsc::Receiver<TelemetryUpdate<N>>,
    /// Publishes participation score snapshots to readers.
    scores: watch::Sender<ScoreSnapshot<N>>,
    /// The telemetry state machine.
    state: TelemetryState<N>,
}

impl<N: Network> TelemetryWorker<N> {
    /// Runs the telemetry worker until every [`Telemetry`] handle has been dropped.
    pub async fn run(mut self) {
        debug!("Starting the validator telemetry worker...");

        while let Some(update) = self.receiver.recv().await {
            // Apply this update, then coalesce everything else that is already queued.
            //
            // Recomputing the participation scores is `O(validators * rounds)`, and during
            // catch-up sync many subdags can arrive back to back. Folding the whole backlog
            // in before recomputing once is cheaper and no less correct, since only the
            // final scores are ever published.
            let mut recompute = false;
            let mut acks = Vec::new();
            let mut next = Some(update);

            while let Some(update) = next {
                match update {
                    TelemetryUpdate::Subdag { gc_round, metadata } => {
                        self.state.garbage_collect_certificates(gc_round);
                        self.state.insert_certificate_metadata(&metadata);
                        recompute = true;
                    }
                    TelemetryUpdate::Certificate(metadata) => {
                        self.state.insert_certificate_metadata(std::slice::from_ref(&*metadata));
                    }
                    // Acknowledged below, after any pending recomputation has been published.
                    TelemetryUpdate::Flush(ack) => acks.push(ack),
                }

                next = self.receiver.try_recv().ok();
            }

            if recompute {
                self.state.update_participation_scores();
                // Note that `send_replace` succeeds even when there are no receivers.
                self.scores.send_replace(Arc::new(self.state.participation_scores.clone()));
            }

            for ack in acks {
                // The waiter may have gone away; that is not an error.
                let _ = ack.send(());
            }
        }

        debug!("The validator telemetry worker has stopped");
    }
}

/// Tracker for the participation metrics of validators.
///
/// This is plain owned data with `&mut self` methods: no locks, no interior mutability,
/// no async. It is only ever reached through [`TelemetryWorker`], which guarantees
/// single-threaded access.
#[derive(Clone, Debug)]
pub struct TelemetryState<N: Network> {
    /// The certificates seen for each round
    /// A mapping of `round` to set of certificate IDs.
    /// Note that this map is sorted to allow grouped iteration over rounds.
    tracked_certificates: BTreeMap<u64, IndexSet<Field<N>>>,

    /// The total number of signatures seen for a validator, including for their own certificates.
    /// A mapping of `address` to a mapping of `round` to `count`.
    validator_signatures: IndexMap<Address<N>, IndexMap<u64, u32>>,

    /// The total number of certificates seen for a validator.
    /// A mapping of `address` to a list of rounds.
    validator_certificates: IndexMap<Address<N>, IndexSet<u64>>,

    /// The certificate, signature, and participation scores for each validator.
    participation_scores: IndexMap<Address<N>, ParticipationScores>,
}

impl<N: Network> Default for TelemetryState<N> {
    /// Initializes a new instance of the telemetry state machine.
    fn default() -> Self {
        Self::new()
    }
}

impl<N: Network> TelemetryState<N> {
    /// Initializes a new instance of the telemetry state machine.
    pub fn new() -> Self {
        Self {
            tracked_certificates: Default::default(),
            validator_signatures: Default::default(),
            validator_certificates: Default::default(),
            participation_scores: Default::default(),
        }
    }

    /// Returns the participation scores as of the last call to [`Self::update_participation_scores`].
    pub fn participation_scores(&self) -> &IndexMap<Address<N>, ParticipationScores> {
        &self.participation_scores
    }

    /// Insert a certificate into the tracker.
    pub fn insert_certificate(&mut self, certificate: &BatchCertificate<N>) {
        self.insert_certificate_metadata(&[CertificateMetadata::new(certificate)]);
    }

    /// Insert the metadata of the given certificates into the tracker.
    pub fn insert_certificate_metadata(&mut self, metadata: &[CertificateMetadata<N>]) {
        for metadata in metadata {
            // If the certificate already exists in the tracker, then skip it.
            if !self.tracked_certificates.entry(metadata.round).or_default().insert(metadata.id) {
                continue;
            }

            // Insert the certificate author and signers.
            for address in &metadata.signers {
                self.validator_signatures
                    .entry(*address)
                    .or_default()
                    .entry(metadata.round)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }

            // Insert the certificate
            self.validator_certificates.entry(metadata.author).or_default().insert(metadata.round);
        }
    }

    /// Calculate and update the participation scores for each validator.
    pub fn update_participation_scores(&mut self) {
        // Calculate the combined score with custom weights:
        // - 90% certificate participation score
        // - 10% signature participation score
        fn weighted_score(certificate_score: f64, signature_score: f64) -> f64 {
            let score = (0.9 * certificate_score) + (0.1 * signature_score);

            // Truncate to the last 2 decimal places.
            (score * 100.0).round() / 100.0
        }

        // Fetch the total number of certificates.
        let total_certificates = self.validator_certificates.values().map(|rounds| rounds.len()).sum::<usize>();

        // Calculate the signature participation scores for each validator.
        let signature_participation_scores: IndexMap<_, _> = self
            .validator_signatures
            .iter()
            .map(|(address, signatures)| {
                let total_signatures = signatures.values().sum::<u32>() as f64;
                let score = total_signatures / total_certificates as f64 * 100.0;
                (*address, score as u16)
            })
            .collect();

        // Calculate the certificate participation scores for each validator.
        // This score is based on how many certificates the validator has included in every two rounds.
        let tracked_rounds: Vec<_> = self.tracked_certificates.keys().skip_while(|r| *r % 2 == 0).copied().collect();
        let certificate_participation_scores: IndexMap<_, _> = self
            .validator_certificates
            .iter()
            .map(|(address, certificate_rounds)| {
                // Count the number of round pairs that are included in the certificate rounds.
                let num_included_round_pairs = tracked_rounds
                    .chunks(2)
                    .filter(|chunk| chunk.iter().any(|r| certificate_rounds.contains(r)))
                    .count();
                // Calculate the number of round pairs.
                let num_round_pairs = (tracked_rounds.len().saturating_add(1)).saturating_div(2);
                // Calculate the score based on the number of certificate rounds the validator is a part of.
                let score = num_included_round_pairs as f64 / num_round_pairs.max(1) as f64 * 100.0;
                (*address, score as u16)
            })
            .collect();

        // Calculate the final participation scores for each validator.
        let validator_addresses: IndexSet<_> =
            signature_participation_scores.keys().chain(certificate_participation_scores.keys()).copied().collect();
        let mut new_participation_scores = IndexMap::new();
        for address in validator_addresses {
            let signature_score = *signature_participation_scores.get(&address).unwrap_or(&0) as f64;
            let certificate_score = *certificate_participation_scores.get(&address).unwrap_or(&0) as f64;
            let combined_score = weighted_score(certificate_score, signature_score);
            new_participation_scores.insert(address, (certificate_score, signature_score, combined_score));
        }

        // Update the participation scores.
        self.participation_scores = new_participation_scores;
    }

    /// Remove the certificates from the telemetry tracker that are no longer relevant based on gc.
    pub fn garbage_collect_certificates(&mut self, gc_round: u64) {
        // Remove certificates that are not longer relevant
        self.tracked_certificates.retain(|&round, _| round > gc_round);

        // Remove signatures that are no longer relevant.
        self.validator_signatures.retain(|_, rounds| {
            rounds.retain(|&round, _| round > gc_round);
            // Remove the address if there are no more tracked signatures.
            !rounds.is_empty()
        });

        // Remove certificates that are no longer relevant.
        self.validator_certificates.retain(|_, rounds| {
            rounds.retain(|&round| round > gc_round);
            // Remove the address if there are no more tracked certificates.
            !rounds.is_empty()
        });
    }
}

/// Projects a score snapshot onto the members of the given committee.
///
/// Validators that are not present in the snapshot are reported as `(0.0, 0.0)`.
fn scores_for_committee<N: Network>(
    snapshot: &IndexMap<Address<N>, ParticipationScores>,
    committee: &Committee<N>,
) -> IndexMap<Address<N>, (f64, f64)> {
    committee
        .members()
        .iter()
        .map(|(address, _)| {
            let scores =
                snapshot.get(address).map(|(cert_score, sig_score, _)| (*cert_score, *sig_score)).unwrap_or((0.0, 0.0));
            (*address, scores)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::{
        ledger::{
            committee::test_helpers::sample_committee_for_round_and_members,
            narwhal::batch_certificate::test_helpers::sample_batch_certificate_for_round,
        },
        prelude::MainnetV0,
        utilities::TestRng,
    };

    use rand::RngExt;

    type CurrentNetwork = MainnetV0;

    #[test]
    fn test_insert_certificates() {
        let rng = &mut TestRng::default();

        // Initialize the telemetry state.
        let mut state = TelemetryState::<CurrentNetwork>::new();

        // Set the current round.
        let current_round = 2;

        // Sample the certificates.
        let mut certificates = IndexSet::new();
        for _ in 0..10 {
            certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        }

        // Insert the certificates.
        assert!(state.tracked_certificates.is_empty());
        for certificate in &certificates {
            state.insert_certificate(certificate);
        }
        assert_eq!(state.tracked_certificates.get(&current_round).unwrap().len(), certificates.len());
    }

    #[test]
    fn test_insert_duplicate_certificate() {
        let rng = &mut TestRng::default();

        // Initialize the telemetry state.
        let mut state = TelemetryState::<CurrentNetwork>::new();

        // Set the current round.
        let current_round = 2;

        // Sample a certificate.
        let certificate = sample_batch_certificate_for_round(current_round, rng);

        // Insert the certificate, and snapshot the tracked signatures.
        state.insert_certificate(&certificate);
        let validator_signatures = state.validator_signatures.clone();

        // Insert the same certificate again.
        state.insert_certificate(&certificate);

        // Ensure the certificate and its signatures were only counted once.
        assert_eq!(state.tracked_certificates.get(&current_round).unwrap().len(), 1);
        assert_eq!(state.validator_signatures, validator_signatures);
        assert_eq!(state.validator_certificates.get(&certificate.author()).unwrap().len(), 1);
    }

    #[test]
    fn test_participation_scores() {
        let rng = &mut TestRng::default();

        // Initialize the telemetry state.
        let mut state = TelemetryState::<CurrentNetwork>::new();

        // Set the current round.
        let current_round = 2;

        // Sample the certificates.
        let mut certificates = IndexSet::new();
        certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        certificates.insert(sample_batch_certificate_for_round(current_round, rng));

        // Initialize the committee.
        let committee = sample_committee_for_round_and_members(
            current_round,
            vec![
                certificates[0].author(),
                certificates[1].author(),
                certificates[2].author(),
                certificates[3].author(),
            ],
            rng,
        );

        // Insert the certificates.
        assert!(state.tracked_certificates.is_empty());
        for certificate in &certificates {
            state.insert_certificate(certificate);
        }

        // Fetch the participation scores, which have not been computed yet.
        let participation_scores = scores_for_committee(state.participation_scores(), &committee);
        assert_eq!(participation_scores.len(), committee.members().len());
        for (address, _) in committee.members() {
            assert_eq!(*participation_scores.get(address).unwrap(), (0.0, 0.0));
        }

        // Calculate the participation scores.
        state.update_participation_scores();

        // Ensure that the participation scores are updated.
        let participation_scores = scores_for_committee(state.participation_scores(), &committee);
        for (address, _) in committee.members() {
            let (cert_score, sig_score) = *participation_scores.get(address).unwrap();
            assert!(cert_score > 0.0 || sig_score > 0.0);
        }

        println!("{participation_scores:?}");
    }

    #[test]
    fn test_garbage_collection() {
        let rng = &mut TestRng::default();

        // Initialize the telemetry state.
        let mut state = TelemetryState::<CurrentNetwork>::new();

        // Set the current round.
        let current_round = 2;
        let next_round = current_round + 1;

        // Sample the certificates for round `current_round`
        let mut certificates = IndexSet::new();
        let num_initial_certificates = rng.random_range(1..10);
        for _ in 0..num_initial_certificates {
            certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        }

        // Sample the certificates for round `next_round`
        let num_new_certificates = rng.random_range(1..10);
        for _ in 0..num_new_certificates {
            certificates.insert(sample_batch_certificate_for_round(next_round, rng));
        }

        // Insert the certificates.
        for certificate in &certificates {
            state.insert_certificate(certificate);
        }
        assert_eq!(state.tracked_certificates.get(&current_round).unwrap().len(), num_initial_certificates);
        assert_eq!(state.tracked_certificates.get(&next_round).unwrap().len(), num_new_certificates);

        // Garbage collect the certificates
        state.garbage_collect_certificates(current_round);
        assert!(!state.tracked_certificates.contains_key(&current_round));
        assert_eq!(state.tracked_certificates.get(&next_round).unwrap().len(), num_new_certificates);
    }

    /// Exercises the handle -> queue -> worker -> watch plumbing.
    ///
    /// The score calculation itself is covered by the [`TelemetryState`] tests above;
    /// this test is about the actor wiring: that updates reach the worker, that `flush`
    /// observes them, that reads project the published snapshot, and that dropping the
    /// last handle stops the worker.
    #[tokio::test]
    async fn test_worker_applies_updates() {
        let rng = &mut TestRng::default();

        // Initialize the telemetry tracker and spawn its worker.
        let (telemetry, worker) = Telemetry::<CurrentNetwork>::new();
        let handle = tokio::spawn(worker.run());

        // Set the current round.
        let current_round = 2;

        // Sample the certificates.
        let certificates: Vec<_> = (0..4).map(|_| sample_batch_certificate_for_round(current_round, rng)).collect();

        // Initialize the committee.
        let committee = sample_committee_for_round_and_members(
            current_round,
            certificates.iter().map(|certificate| certificate.author()).collect(),
            rng,
        );

        // Before anything is published, every committee member reads as zero.
        let participation_scores = telemetry.get_participation_scores(&committee);
        assert_eq!(participation_scores.len(), committee.members().len());
        for (address, _) in committee.members() {
            assert_eq!(*participation_scores.get(address).unwrap(), (0.0, 0.0));
        }

        // Insert the certificates, and wait for the worker to apply them.
        for certificate in &certificates {
            telemetry.insert_certificate(certificate);
        }
        telemetry.flush().await.unwrap();

        // Nothing was dropped, and the reads still succeed for every committee member.
        assert_eq!(telemetry.num_dropped(), 0);
        assert_eq!(telemetry.get_participation_scores(&committee).len(), committee.members().len());

        // Dropping the last handle stops the worker.
        drop(telemetry);
        handle.await.unwrap();
    }
}
