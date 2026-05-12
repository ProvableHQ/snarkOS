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

use crate::{CREATE_BATCH_INTERVAL, MAX_BATCH_DELAY, MIN_BATCH_DELAY};

use anyhow::Result;
use colored::Colorize;
use futures::future::BoxFuture;
use snarkvm::{prelude::Network, utilities::flatten_error};
use std::{marker::PhantomData, sync::Arc};
use tokio::{
    sync::watch,
    time::{Instant, sleep, sleep_until},
};
use tracing::{debug, warn};

/// Abstracts over batch-proposal operations, allowing the proposal loop to be tested without a
/// real primary.
#[async_trait::async_trait]
pub(super) trait BatchPropose: Send + Sync {
    /// Returns the current consensus round.
    fn current_round(&self) -> u64;

    /// Returns `None` if the node is already synced; otherwise returns a future that resolves
    /// once sync completes.
    fn wait_for_synced_if_syncing(&self) -> Option<BoxFuture<'_, ()>>;

    /// Returns `true` if the node is currently synced with the network.
    fn is_synced(&self) -> bool;

    /// Attempts to propose a batch.
    ///
    /// Returns `Ok(true)` when a batch was successfully proposed, `Ok(false)` to retry, and
    /// `Err` on an unexpected error.
    async fn propose_batch(&self) -> Result<bool>;
}

/// Manages batch proposal readiness and drives the batch proposal loop.
///
/// Holds the readiness state and the logic for the proposal task. The actual task is started by
/// calling [`Self::run`] inside a spawned future (see [`Primary::start_handlers`]).
pub struct ProposalTask<N: Network> {
    inner: Arc<ProposalTaskInner>,
    _phantom: PhantomData<N>,
}

/// Manual `Clone` impl so that `N: Clone` is not required.
impl<N: Network> Clone for ProposalTask<N> {
    fn clone(&self) -> Self {
        Self { inner: Arc::clone(&self.inner), _phantom: PhantomData }
    }
}

/// The inner state of a [`ProposalTask`], shared via `Arc`.
struct ProposalTaskInner {
    /// Tracks whether the primary is ready to propose a new batch.
    ///
    /// Initialized to `true` so round 1 can be proposed immediately without an explicit signal.
    /// Set to `true` by [`ProposalTask::signal`] when a new round starts,
    /// and reset to `false` after a batch is successfully proposed.
    ready: watch::Sender<bool>,
}

impl<N: Network> Default for ProposalTask<N> {
    fn default() -> Self {
        let (ready, _) = watch::channel(true);
        Self { inner: Arc::new(ProposalTaskInner { ready }), _phantom: PhantomData }
    }
}

impl<N: Network> ProposalTask<N> {
    /// Signals that the primary is ready to propose a new batch for the current round.
    ///
    /// Should be called from [`Primary::try_increment_to_the_next_round`] whenever the primary
    /// successfully advances to a new round.
    pub fn signal(&self) {
        self.inner.ready.send_replace(true);
    }

    /// Runs the batch proposal loop. This is intended to be spawned as a long-running task.
    ///
    /// Each iteration covers one full round (wait → propose → wait for signatures).
    /// The three stages are implemented as separate methods; see their doc-comments for details.
    pub(super) async fn run<P: BatchPropose + 'static>(self, primary: P) {
        let mut ready_rx = self.inner.ready.subscribe();

        loop {
            let round = primary.current_round();
            // TODO(kaimast): the round_start time should be based on the timestamp of the
            // previous batch, not the current wall-clock time.
            let round_start = Instant::now();

            if !Self::wait_until_proposal_ready(&primary, &mut ready_rx, round, round_start).await {
                continue; // round changed; restart
            }

            if !Self::propose(&primary, round).await {
                continue; // round changed; restart
            }

            // Reset readiness so the next round waits for an explicit signal.
            self.inner.ready.send_replace(false);

            Self::wait_for_signatures(&primary, &mut ready_rx, round).await;
        }
    }

    /// Stage 1: Wait until conditions are met to propose a batch.
    ///
    /// Blocks until sync is complete, MIN_BATCH_DELAY has elapsed since `round_start`, and either
    /// `signal()` fires (leader cert arrived) or MAX_BATCH_DELAY expires without one.
    ///
    /// Returns `true` if ready to propose, `false` if the round changed (caller should restart).
    async fn wait_until_proposal_ready<P: BatchPropose>(
        primary: &P,
        ready_rx: &mut watch::Receiver<bool>,
        round: u64,
        round_start: Instant,
    ) -> bool {
        loop {
            if primary.current_round() != round {
                return false;
            }

            // A node cannot propose while it is syncing.
            if let Some(fut) = primary.wait_for_synced_if_syncing() {
                fut.await;
                // Re-check round after sync completes.
                continue;
            }

            // Enforce the minimum inter-proposal delay.
            // This is a no-op once the deadline has already passed.
            sleep_until(round_start + MIN_BATCH_DELAY).await;

            // Wait for a readiness signal, the MAX_BATCH_DELAY deadline, or a short heartbeat
            // that lets the round-change check at the top of the loop fire regularly.
            tokio::select! {
                _ = sleep_until(round_start + MAX_BATCH_DELAY) => {
                    debug!("Did not receive leader certificate within MAX_BATCH_DELAY");
                    return true;
                },
                _ = Self::wait_until_ready(ready_rx) => {
                    return true;
                },
                _ = sleep(CREATE_BATCH_INTERVAL) => {
                    debug!("Skipping batch proposal for round {round} {}", "(not ready yet)".dimmed());
                }
            };
        }
    }

    /// Stage 2: Propose a batch.
    ///
    /// Calls `propose_batch()` with CREATE_BATCH_INTERVAL retries until it returns `Ok(true)`
    /// (batch submitted to the network).
    ///
    /// Returns `true` if the batch was submitted, `false` if the round changed or the node started
    /// syncing (caller should restart; Stage 1 will then await sync completion).
    async fn propose<P: BatchPropose>(primary: &P, round: u64) -> bool {
        let mut attempt = 1u32;
        loop {
            if primary.current_round() != round {
                return false;
            }

            // Bail out if sync started mid-Stage-2; otherwise propose_batch may spin at the
            // CREATE_BATCH_INTERVAL cadence on Ok(false) paths (e.g. previous round has not
            // reached quorum, not enough connected validators, cached batch rebroadcast).
            if !primary.is_synced() {
                return false;
            }

            if attempt > 1 {
                sleep(CREATE_BATCH_INTERVAL).await;
                debug!("Retrying batch proposal for round {round} (attempt #{attempt})");
            }

            // Note: Do NOT spawn a task around this function call.  Proposing a batch is a
            // critical path, and only one batch needs to be proposed at a time.
            match primary.propose_batch().await {
                Ok(true) => return true, // batch submitted; proceed to Stage 3
                Ok(false) => {}          // not ready yet; retry
                Err(err) => {
                    warn!("{}", flatten_error(err.context("Cannot propose a batch")));
                }
            }

            attempt += 1;
        }
    }

    /// Stage 3: Wait for the proposed batch to collect enough signatures.
    ///
    /// Periodically rebroadcasts the batch to non-signers (via `propose_batch`) at most once per
    /// MAX_BATCH_DELAY until the round advances. Returns when the round changes or when the node
    /// starts syncing — in the latter case the outer loop restarts and Stage 1's sync gate takes
    /// over.
    async fn wait_for_signatures<P: BatchPropose>(primary: &P, ready_rx: &mut watch::Receiver<bool>, round: u64) {
        loop {
            if primary.current_round() != round {
                return;
            }

            // Wait for the rebroadcast interval or an explicit round-advance signal,
            // whichever comes first.
            tokio::select! {
                _ = Self::wait_until_ready(ready_rx) => return, // round advanced
                _ = sleep(MAX_BATCH_DELAY) => {}
            }

            if primary.current_round() != round {
                return;
            }

            // A node cannot rebroadcast its proposed batch while it is syncing — its previous
            // certificates may be stale and peers won't sign it anyway. Bail out so the outer
            // loop falls back through Stage 1, which awaits sync completion before proposing.
            if !primary.is_synced() {
                return;
            }

            // Rebroadcast to non-signers (`propose_batch` handles this internally).
            match primary.propose_batch().await {
                Ok(_) => {}
                Err(err) => {
                    warn!("{}", flatten_error(err.context("Cannot rebroadcast a batch")));
                }
            }
        }
    }

    /// Waits until the readiness watch channel holds `true`. Returns immediately if it already does.
    ///
    /// Spurious wakeups (e.g. from a reset to `false`) are handled by re-checking the value in a loop.
    async fn wait_until_ready(receiver: &mut watch::Receiver<bool>) {
        loop {
            // Fetch the `is_ready` value and return if it is true.
            if *receiver.borrow_and_update() {
                return;
            }

            // Block until the `is_value` changed, or the channel is closed.
            if receiver.changed().await.is_err() {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use snarkvm::prelude::MainnetV0;
    use std::{
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicU32, Ordering},
        },
        time::Duration,
    };
    use tokio::sync::Notify;

    /// A minimal [`BatchPropose`] implementation for testing.
    ///
    /// Always reports round 1 and synced. Records how many times [`propose_batch`] is called and
    /// fires a [`Notify`] on each call.
    struct DummyProposer {
        propose_count: Arc<AtomicU32>,
        proposed_notify: Arc<Notify>,
    }

    #[async_trait::async_trait]
    impl BatchPropose for DummyProposer {
        fn current_round(&self) -> u64 {
            1
        }

        fn wait_for_synced_if_syncing(&self) -> Option<BoxFuture<'_, ()>> {
            None
        }

        fn is_synced(&self) -> bool {
            true
        }

        async fn propose_batch(&self) -> Result<bool> {
            self.propose_count.fetch_add(1, Ordering::SeqCst);
            self.proposed_notify.notify_one();

            Ok(true)
        }
    }

    /// A [`BatchPropose`] implementation that returns round 1 on the very first call to
    /// `current_round`, then round 2 for all subsequent calls.
    ///
    /// This simulates the round advancing between the outer-loop capture and the inner-loop
    /// condition check, without any real-time waiting or time mocking.
    struct RoundAdvancingProposer {
        current_round_calls: Arc<AtomicU32>,
        propose_count: Arc<AtomicU32>,
    }

    #[async_trait::async_trait]
    impl BatchPropose for RoundAdvancingProposer {
        fn current_round(&self) -> u64 {
            let n = self.current_round_calls.fetch_add(1, Ordering::SeqCst);
            if n == 0 { 1 } else { 2 }
        }

        fn wait_for_synced_if_syncing(&self) -> Option<BoxFuture<'_, ()>> {
            None
        }

        fn is_synced(&self) -> bool {
            true
        }

        async fn propose_batch(&self) -> Result<bool> {
            self.propose_count.fetch_add(1, Ordering::SeqCst);
            Ok(true)
        }
    }

    /// A [`BatchPropose`] implementation that returns `Ok(false)` a fixed number of times before
    /// succeeding.
    struct RetryProposer {
        retries_before_success: u32,
        propose_count: Arc<AtomicU32>,
        proposed_notify: Arc<Notify>,
    }

    #[async_trait::async_trait]
    impl BatchPropose for RetryProposer {
        fn current_round(&self) -> u64 {
            1
        }

        fn wait_for_synced_if_syncing(&self) -> Option<BoxFuture<'_, ()>> {
            None
        }

        fn is_synced(&self) -> bool {
            true
        }

        async fn propose_batch(&self) -> Result<bool> {
            let count = self.propose_count.fetch_add(1, Ordering::SeqCst) + 1;
            if count <= self.retries_before_success {
                Ok(false)
            } else {
                self.proposed_notify.notify_one();
                Ok(true)
            }
        }
    }

    /// Signals the proposal task and verifies that `propose_batch` is called on the dummy.
    #[tokio::test]
    async fn test_proposal_task_calls_propose_batch_on_signal() {
        // Start with the task not ready so the initial signal is the trigger.
        let (ready, _) = watch::channel(false);
        let task = ProposalTask::<MainnetV0> { inner: Arc::new(ProposalTaskInner { ready }), _phantom: PhantomData };

        let proposed_notify = Arc::new(Notify::new());
        let propose_count = Arc::new(AtomicU32::new(0));

        let proposer = DummyProposer { propose_count: propose_count.clone(), proposed_notify: proposed_notify.clone() };

        let task_for_spawn = task.clone();
        tokio::spawn(task_for_spawn.run(proposer));

        // Before signalling, propose_batch should not have been called.
        sleep(Duration::from_millis(50)).await;
        assert_eq!(propose_count.load(Ordering::SeqCst), 0, "propose_batch called before signal");

        // Signal readiness — the proposal loop should wake up and call propose_batch.
        task.signal();

        tokio::time::timeout(std::time::Duration::from_secs(5), proposed_notify.notified())
            .await
            .expect("propose_batch was not called within 5 seconds after signal");

        assert!(propose_count.load(Ordering::SeqCst) >= 1, "propose_batch was not called");
    }

    /// When the round advances between iterations, `propose_batch` is not called for the old round.
    ///
    /// `RoundAdvancingProposer` returns round 1 on the first `current_round()` call (outer-loop
    /// capture) and round 2 on every subsequent call. The inner-loop condition therefore fails
    /// immediately — no time mocking needed.
    #[tokio::test]
    async fn test_proposal_task_exits_on_round_advancement() {
        let propose_count = Arc::new(AtomicU32::new(0));
        let proposer = RoundAdvancingProposer {
            current_round_calls: Arc::new(AtomicU32::new(0)),
            propose_count: propose_count.clone(),
        };

        // Start not-ready so the task parks in round 2's inner loop without proposing round 1.
        let (ready, _) = watch::channel(false);
        let task = ProposalTask::<MainnetV0> { inner: Arc::new(ProposalTaskInner { ready }), _phantom: PhantomData };

        tokio::spawn(task.run(proposer));

        // Yield once: the task runs through round 1 (inner loop exits immediately because
        // current_round() already returns 2) and then parks in round 2's inner loop.
        tokio::task::yield_now().await;

        assert_eq!(propose_count.load(Ordering::SeqCst), 0, "propose_batch called despite round advancement");
    }

    /// Tests the following scenario
    ///
    ///   1. A batch was already certified for the current round, so readiness is `false`.
    ///   2. `signal()` is **never** called externally — the BFT cannot advance the round until
    ///      `propose_batch()` is called (which internally checks the leader-certificate timer).
    #[test_log::test(tokio::test)]
    async fn test_proposal_task_advances_without_leader_cert() {
        // Start NOT ready: simulates a batch that was already certified for the round but the
        // round has not yet advanced (the even-round leader cert was missing — e.g. the elected
        // leader was one of the freshly-reset minority validators).
        let (ready, _) = watch::channel(false);
        let task = ProposalTask::<MainnetV0> { inner: Arc::new(ProposalTaskInner { ready }), _phantom: PhantomData };

        let proposed_notify = Arc::new(Notify::new());
        let propose_count = Arc::new(AtomicU32::new(0));

        // A proposer that stays on round 1 and returns Ok(true) on every call to
        // propose_batch(), simulating try_advance_to_next_round finding the leader-certificate
        // timer expired and advancing the round without an external signal().
        struct NoSignalProposer {
            propose_count: Arc<AtomicU32>,
            proposed_notify: Arc<Notify>,
        }

        #[async_trait::async_trait]
        impl BatchPropose for NoSignalProposer {
            fn current_round(&self) -> u64 {
                1
            }

            fn wait_for_synced_if_syncing(&self) -> Option<BoxFuture<'_, ()>> {
                None
            }

            fn is_synced(&self) -> bool {
                true
            }

            async fn propose_batch(&self) -> Result<bool> {
                self.propose_count.fetch_add(1, Ordering::SeqCst);
                self.proposed_notify.notify_one();
                Ok(true)
            }
        }

        let proposer =
            NoSignalProposer { propose_count: propose_count.clone(), proposed_notify: proposed_notify.clone() };

        // signal() is intentionally never called — the task must retry on its own.
        tokio::spawn(task.run(proposer));

        // Allow enough time for MAX_BATCH_DELAY (2.5 s) to elapse plus the CREATE_BATCH_INTERVAL
        // (250 ms) retry window. Use 10 s to give generous headroom on slow CI machines.
        tokio::time::timeout(std::time::Duration::from_secs(10), proposed_notify.notified())
            .await
            .expect("propose_batch was not called");

        assert!(propose_count.load(Ordering::SeqCst) >= 1, "propose_batch should have been called at least once");
    }

    /// After the leader-certificate timer fires (MAX_BATCH_DELAY elapses without an explicit
    /// `signal()`), the task should still retry `propose_batch` when it returns `Ok(false)` and
    /// eventually succeed once it returns `Ok(true)`.
    ///
    /// This models the real primary: when a round is already certified but the round has not yet
    /// advanced (e.g. the elected leader was a freshly-reset minority validator), `propose_batch`
    /// returns `Ok(false)` until `try_advance_to_next_round` can make progress.
    #[test_log::test(tokio::test)]
    async fn test_proposal_task_retries_after_leader_timeout() {
        const RETRIES: u32 = 2;

        // Start NOT ready — no external signal will be sent. The task must wait for
        // MAX_BATCH_DELAY to fire, then retry until propose_batch succeeds.
        let (ready, _) = watch::channel(false);
        let task = ProposalTask::<MainnetV0> { inner: Arc::new(ProposalTaskInner { ready }), _phantom: PhantomData };

        let proposed_notify = Arc::new(Notify::new());
        let propose_count = Arc::new(AtomicU32::new(0));
        let proposer = RetryProposer {
            retries_before_success: RETRIES,
            propose_count: propose_count.clone(),
            proposed_notify: proposed_notify.clone(),
        };

        // signal() is intentionally never called — the leader timeout arm must trigger.
        tokio::spawn(task.run(proposer));

        // Allow enough time for MAX_BATCH_DELAY (2.5 s) plus RETRIES × CREATE_BATCH_INTERVAL (250 ms each).
        // Use 10 s to give generous headroom on slow CI machines.
        tokio::time::timeout(std::time::Duration::from_secs(10), proposed_notify.notified())
            .await
            .expect("propose_batch did not succeed within 10 seconds after leader timeout");

        // Stage 3 may make additional rebroadcast calls after success, so use >.
        assert!(propose_count.load(Ordering::SeqCst) > RETRIES, "expected at least {} total attempts", RETRIES + 1);
    }

    /// When `propose_batch` returns `Ok(false)`, the task retries within the same round until
    /// it succeeds.
    #[tokio::test]
    async fn test_proposal_task_retries_on_false() {
        const RETRIES: u32 = 2;

        // Default starts ready, so no signal needed.
        let task = ProposalTask::<MainnetV0>::default();

        let proposed_notify = Arc::new(Notify::new());
        let propose_count = Arc::new(AtomicU32::new(0));
        let proposer = RetryProposer {
            retries_before_success: RETRIES,
            propose_count: propose_count.clone(),
            proposed_notify: proposed_notify.clone(),
        };

        tokio::spawn(task.run(proposer));

        // The task internally waits MIN_BATCH_DELAY before the first attempt; allow up to 10s.
        tokio::time::timeout(std::time::Duration::from_secs(10), proposed_notify.notified())
            .await
            .expect("propose_batch did not succeed within 10 seconds");

        // Stage 3 may make additional rebroadcast calls after success, so use >.
        assert!(propose_count.load(Ordering::SeqCst) > RETRIES, "expected at least {} total attempts", RETRIES + 1);
    }

    /// While the node is syncing, Stage 3 must not rebroadcast the proposed batch — its previous
    /// certificates may be stale and peers will not sign it. Once sync completes, rebroadcast
    /// should resume.
    #[test_log::test(tokio::test)]
    async fn test_proposal_task_pauses_rebroadcast_while_syncing() {
        /// Synced for Stage 1/2. After the first successful `propose_batch`, flips to "syncing"
        /// so Stage 3's rebroadcast loop must pause. The held `sync_release` `Notify` lets the
        /// test resume sync on demand to assert that rebroadcast comes back.
        struct SyncTogglingProposer {
            propose_count: Arc<AtomicU32>,
            proposed_notify: Arc<Notify>,
            is_syncing: Arc<AtomicBool>,
            sync_release: Arc<Notify>,
        }

        #[async_trait::async_trait]
        impl BatchPropose for SyncTogglingProposer {
            fn current_round(&self) -> u64 {
                1
            }

            fn wait_for_synced_if_syncing(&self) -> Option<BoxFuture<'_, ()>> {
                if self.is_syncing.load(Ordering::SeqCst) {
                    let release = self.sync_release.clone();
                    Some(Box::pin(async move { release.notified().await }))
                } else {
                    None
                }
            }

            fn is_synced(&self) -> bool {
                !self.is_syncing.load(Ordering::SeqCst)
            }

            async fn propose_batch(&self) -> Result<bool> {
                self.propose_count.fetch_add(1, Ordering::SeqCst);
                self.proposed_notify.notify_one();
                // Transition to syncing once the Stage 2 proposal has gone out.
                self.is_syncing.store(true, Ordering::SeqCst);
                Ok(true)
            }
        }

        // Default starts ready — Stage 1 completes after MIN_BATCH_DELAY without a signal.
        let task = ProposalTask::<MainnetV0>::default();

        let proposed_notify = Arc::new(Notify::new());
        let propose_count = Arc::new(AtomicU32::new(0));
        let is_syncing = Arc::new(AtomicBool::new(false));
        let sync_release = Arc::new(Notify::new());

        let proposer = SyncTogglingProposer {
            propose_count: propose_count.clone(),
            proposed_notify: proposed_notify.clone(),
            is_syncing: is_syncing.clone(),
            sync_release: sync_release.clone(),
        };

        tokio::spawn(task.run(proposer));

        // Wait for Stage 2 to make its single propose call.
        tokio::time::timeout(Duration::from_secs(10), proposed_notify.notified())
            .await
            .expect("Stage 2 did not call propose_batch within 10 seconds");
        assert_eq!(propose_count.load(Ordering::SeqCst), 1, "expected exactly one Stage 2 call");

        // Stage 3 sleeps MAX_BATCH_DELAY before each rebroadcast attempt; wait past that to give
        // the sync gate a chance to fire. Without the gate, propose_count would increment here.
        tokio::time::sleep(MAX_BATCH_DELAY + Duration::from_secs(1)).await;
        assert_eq!(propose_count.load(Ordering::SeqCst), 1, "Stage 3 rebroadcast fired while the node was syncing",);

        // Release sync. Stage 3 should resume rebroadcasting after the next MAX_BATCH_DELAY tick.
        is_syncing.store(false, Ordering::SeqCst);
        sync_release.notify_waiters();

        tokio::time::timeout(MAX_BATCH_DELAY + Duration::from_secs(5), proposed_notify.notified())
            .await
            .expect("Stage 3 did not resume rebroadcast after sync completed");
        assert!(propose_count.load(Ordering::SeqCst) >= 2, "expected rebroadcast after sync completed");
    }

    /// Stage 2 retries `propose_batch` every CREATE_BATCH_INTERVAL (250ms) while it returns
    /// `Ok(false)`. If sync starts mid-retry, the loop must bail out so the outer loop can fall
    /// back through Stage 1's sync gate — otherwise the node spins, calling `propose_batch` four
    /// times per second (which on a real primary triggers the cached-batch rebroadcast).
    #[test_log::test(tokio::test)]
    async fn test_proposal_task_bails_stage_2_when_syncing_starts() {
        /// Always returns `Ok(false)` from `propose_batch`, so Stage 2 enters its retry loop.
        /// The test flips `syncing` to true after a few calls; subsequent calls should stop.
        struct AlwaysFalseProposer {
            propose_count: Arc<AtomicU32>,
            syncing: Arc<AtomicBool>,
            sync_release: Arc<Notify>,
        }

        #[async_trait::async_trait]
        impl BatchPropose for AlwaysFalseProposer {
            fn current_round(&self) -> u64 {
                1
            }

            fn wait_for_synced_if_syncing(&self) -> Option<BoxFuture<'_, ()>> {
                if self.syncing.load(Ordering::SeqCst) {
                    let release = self.sync_release.clone();
                    Some(Box::pin(async move { release.notified().await }))
                } else {
                    None
                }
            }

            fn is_synced(&self) -> bool {
                !self.syncing.load(Ordering::SeqCst)
            }

            async fn propose_batch(&self) -> Result<bool> {
                self.propose_count.fetch_add(1, Ordering::SeqCst);
                // Never succeed — Stage 2 will keep retrying every CREATE_BATCH_INTERVAL.
                Ok(false)
            }
        }

        let task = ProposalTask::<MainnetV0>::default();
        let propose_count = Arc::new(AtomicU32::new(0));
        let syncing = Arc::new(AtomicBool::new(false));
        let sync_release = Arc::new(Notify::new());

        let proposer = AlwaysFalseProposer {
            propose_count: propose_count.clone(),
            syncing: syncing.clone(),
            sync_release: sync_release.clone(),
        };

        tokio::spawn(task.run(proposer));

        // Let Stage 2 spin for a few CREATE_BATCH_INTERVAL ticks (250ms each). After MIN_BATCH_DELAY
        // (1s) for Stage 1 to release plus a couple of retry cycles, we should see >= 2 calls.
        tokio::time::sleep(Duration::from_millis(2000)).await;
        let pre_sync_calls = propose_count.load(Ordering::SeqCst);
        assert!(pre_sync_calls >= 2, "Stage 2 should have retried at least twice while synced (got {pre_sync_calls})");

        // Start syncing. Stage 2 must bail out, and Stage 1 must then block on the sync gate.
        syncing.store(true, Ordering::SeqCst);

        // Allow Stage 2 to notice and bail, and Stage 1 to install its sync wait.
        tokio::time::sleep(Duration::from_millis(500)).await;
        let after_bail_calls = propose_count.load(Ordering::SeqCst);

        // From this point, no further propose_batch calls should happen until sync releases.
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert_eq!(
            propose_count.load(Ordering::SeqCst),
            after_bail_calls,
            "propose_batch called while syncing — Stage 2 did not bail (or Stage 1 missed the gate)",
        );

        // Release sync; the outer loop should drive Stage 2 again.
        syncing.store(false, Ordering::SeqCst);
        sync_release.notify_waiters();
        tokio::time::sleep(Duration::from_secs(2)).await;
        assert!(propose_count.load(Ordering::SeqCst) > after_bail_calls, "Stage 2 did not resume after sync completed",);
    }
}
