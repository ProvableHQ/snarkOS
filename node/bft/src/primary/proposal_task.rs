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
use futures::{FutureExt, future::BoxFuture};
#[cfg(feature = "locktick")]
use locktick::parking_lot::RwLock;
#[cfg(not(feature = "locktick"))]
use parking_lot::RwLock;
use snarkvm::{prelude::Network, utilities::flatten_error};
use std::{marker::PhantomData, sync::Arc};
use tokio::{
    sync::Notify,
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

    /// Attempts to propose a batch.
    ///
    /// Returns `Ok(true)` when a batch was successfully proposed, `Ok(false)` to retry, and
    /// `Err` on an unexpected error.
    async fn propose_batch(&self) -> Result<bool>;
}

/// Manages batch proposal readiness and drives the batch proposal loop.
///
/// Holds the condition-variable state (`is_proposal_ready` + `is_ready_notify`) and the
/// logic for the proposal task.  The actual task is started by calling [`Self::run`] inside a
/// spawned future (see [`Primary::start_handlers`]).
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
    /// Whether the primary is ready to propose a new batch.
    ///
    /// Initialized to `true` so round 1 can be proposed immediately without an explicit signal.
    /// Set to `true` by [`ProposalTask::signal`] when a new round starts,
    /// and reset to `false` after a batch is successfully proposed.
    is_proposal_ready: RwLock<bool>,

    /// Notifies the proposal loop when `is_proposal_ready` transitions to `true`.
    is_ready_notify: Notify,
}

impl<N: Network> Default for ProposalTask<N> {
    fn default() -> Self {
        Self {
            inner: Arc::new(ProposalTaskInner { is_proposal_ready: RwLock::new(true), is_ready_notify: Notify::new() }),
            _phantom: PhantomData,
        }
    }
}

impl<N: Network> ProposalTask<N> {
    /// Signals that the primary is ready to propose a new batch for the current round.
    ///
    /// Should be called from [`Primary::try_increment_to_the_next_round`] whenever the primary
    /// successfully advances to a new round.
    pub fn signal(&self) {
        *self.inner.is_proposal_ready.write() = true;
        self.inner.is_ready_notify.notify_one();
    }

    /// Waits until the primary is ready to propose. Returns immediately if already ready.
    async fn wait(&self) {
        loop {
            let mut fut = std::pin::pin!(self.inner.is_ready_notify.notified());
            {
                if *self.inner.is_proposal_ready.read() {
                    return;
                }
                fut.as_mut().enable();
            }
            fut.await;
        }
    }

    /// Runs the batch proposal loop. This is intended to be spawned as a long-running task.
    ///
    /// Each outer loop iteration covers one proposed batch.  The inner loop retries within the
    /// same round until the round advances or a batch is successfully certified.
    pub(super) async fn run<P: BatchPropose + 'static>(self, primary: P) {
        loop {
            let round_start = Instant::now();
            let round = primary.current_round();
            let mut leader_timeout_reached = false;
            let mut attempt = 1;

            // The inner loop represents multiple attempts to propose a batch within the same round.
            // It exits when the round advances or a batch is successfully proposed.
            while primary.current_round() == round {
                // A node cannot propose while it is syncing. So, block here first.
                if let Some(fut) = primary.wait_for_synced_if_syncing() {
                    fut.await;
                    // Restart the loop, as the current round may have changed.
                    continue;
                }

                // If the minimum batch delay has not been reached yet, wait for it first,
                // as we cannot propose without it having elapsed in any case.e
                // TODO(kaimast): the sleep time should be based on the timestamp of the previous batch
                sleep_until(round_start + MIN_BATCH_DELAY).await;

                // Wait for either the leader to time-out or for the proposal to be ready.
                // Additionally, we add a smaller timeout here out of caution, to ensure node does not get stuck on this select! call.
                tokio::select! {
                                    _ = sleep_until(round_start + MAX_BATCH_DELAY) => {
                                        if !leader_timeout_reached {
                                            info!("Leader for round {round} timed out");
                                        }
                                        leader_timeout_reached = true;
                                    },
                                    _ = self.wait().boxed() => {},
                                    _ = sleep(CREATE_BATCH_INTERVAL) => {
                // Retry if the timeout triggered (but do not count it as a proper attempt).
                                    debug!("Skipping batch proposal for round {round} {}", "(not ready yet)".dimmed());
                                    continue;
                                    }
                                };

                if attempt > 1 {
                    // Sleep to avoid busy waiting, if all conditions were met
                    // and we still need to retry.
                    sleep(CREATE_BATCH_INTERVAL).await;

                    // Print a log message to see how often these retries happen (or if ever).
                    debug!("Retrying batch proposal for round {round} (attempt #{attempt})");
                }

                // If there is no proposed batch, attempt to propose a batch.
                // Note: Do NOT spawn a task around this function call. Proposing a batch is a
                // critical path, and only one batch needs to be proposed at a time.
                match primary.propose_batch().await {
                    Ok(true) => {
                        // Reset readiness so the next round waits for an explicit signal.
                        *self.inner.is_proposal_ready.write() = false;
                        break;
                    }
                    Ok(false) => (), // retry
                    Err(err) => {
                        // Log warning and retry
                        warn!("{}", flatten_error(err.context("Cannot propose a batch")));
                    }
                }

                attempt += 1;
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
            atomic::{AtomicU32, Ordering},
        },
        time::Duration,
    };

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
        let task = ProposalTask::<MainnetV0> {
            inner: Arc::new(ProposalTaskInner {
                is_proposal_ready: RwLock::new(false),
                is_ready_notify: Notify::new(),
            }),
            _phantom: PhantomData,
        };

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
        let task = ProposalTask::<MainnetV0> {
            inner: Arc::new(ProposalTaskInner {
                is_proposal_ready: RwLock::new(false),
                is_ready_notify: Notify::new(),
            }),
            _phantom: PhantomData,
        };

        tokio::spawn(task.run(proposer));

        // Yield once: the task runs through round 1 (inner loop exits immediately because
        // current_round() already returns 2) and then parks in round 2's join_all.
        tokio::task::yield_now().await;

        assert_eq!(propose_count.load(Ordering::SeqCst), 0, "propose_batch called despite round advancement");
    }

    /// Tests the following scenario
    ///
    ///   1. A batch was already certified for the current round, so `is_proposal_ready` is `false`.
    ///   2. `signal()` is **never** called externally — the BFT cannot advance the round until
    ///      `propose_batch()` is called (which internally checks the leader-certificate timer).
    #[test_log::test(tokio::test)]
    async fn test_proposal_task_advances_without_leader_cert() {
        // Start NOT ready: simulates a batch that was already certified for the round but the
        // round has not yet advanced (the even-round leader cert was missing — e.g. the elected
        // leader was one of the freshly-reset minority validators).
        let task = ProposalTask::<MainnetV0> {
            inner: Arc::new(ProposalTaskInner {
                is_proposal_ready: RwLock::new(false),
                is_ready_notify: Notify::new(),
            }),
            _phantom: PhantomData,
        };

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

    /// When `propose_batch` returns `Ok(false)`, the task retries within the same round until
    /// it succeeds.
    ///
    /// The first attempt is gated behind `MAX_BATCH_DELAY`; subsequent retries are nearly
    /// instant because `reached_min_batch_delay` is true and the futures list is empty.
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

        // The task internally waits MAX_BATCH_DELAY before the first attempt; allow up to 10s.
        tokio::time::timeout(std::time::Duration::from_secs(10), proposed_notify.notified())
            .await
            .expect("propose_batch did not succeed within 10 seconds");

        assert_eq!(propose_count.load(Ordering::SeqCst), RETRIES + 1, "expected {} total attempts", RETRIES + 1);
    }
}
