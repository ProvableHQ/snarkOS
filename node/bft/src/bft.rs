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

use crate::{
    MAX_LEADER_CERTIFICATE_DELAY_IN_SECS,
    helpers::{ConsensusSender, DAG, PrimaryReceiver, PrimarySender, Storage, fmt_id, now},
    primary::{Primary, PrimaryCallback},
    sync::SyncCallback,
};
use snarkos_account::Account;
use snarkos_node_bft_ledger_service::LedgerService;
use snarkos_node_sync::{BlockSync, Ping};
use snarkvm::{
    console::account::Address,
    ledger::{
        block::Transaction,
        committee::Committee,
        narwhal::{BatchCertificate, Data, Subdag, Transmission, TransmissionID},
        puzzle::{Solution, SolutionID},
    },
    prelude::{Field, Network, Result, bail, ensure},
    utilities::LoggableError,
};

use aleo_std::StorageMode;
use anyhow::Context;
use colored::Colorize;
use indexmap::{IndexMap, IndexSet};
#[cfg(feature = "locktick")]
use locktick::{parking_lot::RwLock, tokio::Mutex as TMutex};
#[cfg(not(feature = "locktick"))]
use parking_lot::RwLock;
use std::{
    collections::{BTreeMap, HashSet},
    net::SocketAddr,
    sync::{
        Arc,
        atomic::{AtomicI64, Ordering},
    },
};
#[cfg(not(feature = "locktick"))]
use tokio::sync::Mutex as TMutex;
use tokio::sync::{OnceCell, oneshot};

#[derive(Clone)]
pub struct BFT<N: Network> {
    /// The primary for this node.
    primary: Primary<N>,
    /// The DAG of batches from which we build the blockchain.
    dag: Arc<RwLock<DAG<N>>>,
    /// The batch certificate of the leader from the current even round, if one was present.
    leader_certificate: Arc<RwLock<Option<BatchCertificate<N>>>>,
    /// The timer for the leader certificate to be received.
    leader_certificate_timer: Arc<AtomicI64>,
    /// The consensus sender.
    consensus_sender: Arc<OnceCell<ConsensusSender<N>>>,
    /// The BFT lock.
    lock: Arc<TMutex<()>>,
}

impl<N: Network> BFT<N> {
    /// Initializes a new instance of the BFT.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account: Account<N>,
        storage: Storage<N>,
        ledger: Arc<dyn LedgerService<N>>,
        block_sync: Arc<BlockSync<N>>,
        ip: Option<SocketAddr>,
        trusted_validators: &[SocketAddr],
        storage_mode: StorageMode,
        dev: Option<u16>,
    ) -> Result<Self> {
        Ok(Self {
            primary: Primary::new(account, storage, ledger, block_sync, ip, trusted_validators, storage_mode, dev)?,
            dag: Default::default(),
            leader_certificate: Default::default(),
            leader_certificate_timer: Default::default(),
            consensus_sender: Default::default(),
            lock: Default::default(),
        })
    }

    /// Run the BFT instance.
    ///
    /// This will return as soon as all required tasks are spawned.
    /// The function must not be called more than once per instance.
    pub async fn run(
        &mut self,
        ping: Option<Arc<Ping<N>>>,
        consensus_sender: Option<ConsensusSender<N>>,
        primary_sender: PrimarySender<N>,
        primary_receiver: PrimaryReceiver<N>,
    ) -> Result<()> {
        info!("Starting the BFT instance...");
        // Set up callbacks.
        let primary_callback = Some(Arc::new(self.clone()) as Arc<dyn PrimaryCallback<N>>);

        let sync_callback = Some(Arc::new(self.clone()) as Arc<dyn SyncCallback<N>>);

        // Next, run the primary instance.
        self.primary.run(ping, primary_callback, sync_callback, primary_sender, primary_receiver).await?;

        // Lastly, set the consensus sender.
        // Note: This ensures that, during initial syncing, that the BFT does not advance the ledger.
        if let Some(consensus_sender) = consensus_sender {
            self.consensus_sender.set(consensus_sender).expect("Consensus sender already set");
        }
        Ok(())
    }

    /// Returns `true` if the primary is synced.
    pub fn is_synced(&self) -> bool {
        self.primary.is_synced()
    }

    /// Returns the primary.
    pub const fn primary(&self) -> &Primary<N> {
        &self.primary
    }

    /// Returns the storage.
    pub const fn storage(&self) -> &Storage<N> {
        self.primary.storage()
    }

    /// Returns the ledger.
    pub fn ledger(&self) -> &Arc<dyn LedgerService<N>> {
        self.primary.ledger()
    }

    /// Returns the leader of the current even round, if one was present.
    pub fn leader(&self) -> Option<Address<N>> {
        self.leader_certificate.read().as_ref().map(|certificate| certificate.author())
    }

    /// Returns the certificate of the leader from the current even round, if one was present.
    pub const fn leader_certificate(&self) -> &Arc<RwLock<Option<BatchCertificate<N>>>> {
        &self.leader_certificate
    }
}

impl<N: Network> BFT<N> {
    /// Returns the number of unconfirmed transmissions.
    pub fn num_unconfirmed_transmissions(&self) -> usize {
        self.primary.num_unconfirmed_transmissions()
    }

    /// Returns the number of unconfirmed ratifications.
    pub fn num_unconfirmed_ratifications(&self) -> usize {
        self.primary.num_unconfirmed_ratifications()
    }

    /// Returns the number of solutions.
    pub fn num_unconfirmed_solutions(&self) -> usize {
        self.primary.num_unconfirmed_solutions()
    }

    /// Returns the number of unconfirmed transactions.
    pub fn num_unconfirmed_transactions(&self) -> usize {
        self.primary.num_unconfirmed_transactions()
    }
}

impl<N: Network> BFT<N> {
    /// Returns the worker transmission IDs.
    pub fn worker_transmission_ids(&self) -> impl '_ + Iterator<Item = TransmissionID<N>> {
        self.primary.worker_transmission_ids()
    }

    /// Returns the worker transmissions.
    pub fn worker_transmissions(&self) -> impl '_ + Iterator<Item = (TransmissionID<N>, Transmission<N>)> {
        self.primary.worker_transmissions()
    }

    /// Returns the worker solutions.
    pub fn worker_solutions(&self) -> impl '_ + Iterator<Item = (SolutionID<N>, Data<Solution<N>>)> {
        self.primary.worker_solutions()
    }

    /// Returns the worker transactions.
    pub fn worker_transactions(&self) -> impl '_ + Iterator<Item = (N::TransactionID, Data<Transaction<N>>)> {
        self.primary.worker_transactions()
    }
}

#[async_trait::async_trait]
impl<N: Network> PrimaryCallback<N> for BFT<N> {
    /// Notification that a new round has started.
    fn update_to_next_round(&self, current_round: u64) -> bool {
        // Ensure the current round is at least the storage round (this is a sanity check).
        let storage_round = self.storage().current_round();
        if current_round < storage_round {
            debug!(
                "BFT is safely skipping an update for round {current_round}, as storage is at round {storage_round}"
            );
            return false;
        }

        // Determine if the BFT is ready to update to the next round.
        let is_ready = match current_round % 2 == 0 {
            true => self.update_leader_certificate_to_even_round(current_round),
            false => self.is_leader_quorum_or_nonleaders_available(current_round),
        };

        #[cfg(feature = "metrics")]
        {
            let start = self.leader_certificate_timer.load(Ordering::SeqCst);
            // Only log if the timer was set, otherwise we get a time difference since the EPOCH.
            if start > 0 {
                let end = now();
                let elapsed = std::time::Duration::from_secs((end - start) as u64);
                metrics::histogram(metrics::bft::COMMIT_ROUNDS_LATENCY, elapsed.as_secs_f64());
            }
        }

        // Log whether the round is going to update.
        if current_round % 2 == 0 {
            // Determine if there is a leader certificate.
            if let Some(leader_certificate) = self.leader_certificate.read().as_ref() {
                // Ensure the state of the leader certificate is consistent with the BFT being ready.
                if !is_ready {
                    trace!(is_ready, "BFT - A leader certificate was found, but 'is_ready' is false");
                }
                // Log the leader election.
                let leader_round = leader_certificate.round();
                match leader_round == current_round {
                    true => {
                        info!("\n\nRound {current_round} elected a leader - {}\n", leader_certificate.author());
                        #[cfg(feature = "metrics")]
                        metrics::increment_counter(metrics::bft::LEADERS_ELECTED);
                    }
                    false => warn!("BFT failed to elect a leader for round {current_round} (!= {leader_round})"),
                }
            } else {
                match is_ready {
                    true => info!("\n\nRound {current_round} reached quorum without a leader\n"),
                    false => info!("{}", format!("\n\nRound {current_round} did not elect a leader (yet)\n").dimmed()),
                }
            }
        }

        // If the BFT is ready, then update to the next round.
        if is_ready {
            // Update to the next round in storage.
            if let Err(err) = self.storage().increment_to_next_round(current_round) {
                err.log_warning(format!("BFT failed to increment to the next round from round {current_round}"));
                return false;
            }
            // Update the timer for the leader certificate.
            self.leader_certificate_timer.store(now(), Ordering::SeqCst);
        }

        is_ready
    }

    /// Notification about a new certificate.
    async fn add_new_certificate(&self, certificate: BatchCertificate<N>) -> Result<()> {
        // Update the DAG with the certificate.
        self.update_dag::<true, false>(certificate).await
    }
}

#[async_trait::async_trait]
impl<N: Network> SyncCallback<N> for BFT<N> {
    /// Syncs the BFT DAG with the given batch certificates. These batch certificates **must**
    /// already exist in the ledger.
    ///
    /// This method commits all the certificates into the DAG.
    /// Note that there is no need to insert the certificates into the DAG, because these certificates
    /// already exist in the ledger and therefore do not need to be re-ordered into future committed subdags.
    async fn sync_dag_at_bootup(&self, certificates: Vec<BatchCertificate<N>>) -> Result<()> {
        // Acquire the BFT write lock.
        let mut dag = self.dag.write();

        // Commit all the certificates.
        for certificate in certificates {
            dag.commit(&certificate, self.storage().max_gc_rounds());
        }

        Ok(())
    }

    /// Sends a new certificate.
    async fn add_new_certificate(&self, certificate: BatchCertificate<N>) -> Result<()> {
        // Update the DAG with the certificate.
        self.update_dag::<true, false>(certificate).await
    }
}

impl<N: Network> BFT<N> {
    /// Updates the leader certificate to the current even round,
    /// returning `true` if the BFT is ready to update to the next round.
    ///
    /// This method runs on every even round, by determining the leader of the current even round,
    /// and setting the leader certificate to their certificate in the round, if they were present.
    fn update_leader_certificate_to_even_round(&self, even_round: u64) -> bool {
        // Retrieve the current round.
        let current_round = self.storage().current_round();
        // Ensure the current round matches the given round.
        if current_round != even_round {
            warn!("BFT storage (at round {current_round}) is out of sync with the current even round {even_round}");
            return false;
        }

        // If the current round is odd, return false.
        if current_round % 2 != 0 || current_round < 2 {
            error!("BFT cannot update the leader certificate in an odd round");
            return false;
        }

        // Retrieve the certificates for the current round.
        let current_certificates = self.storage().get_certificates_for_round(current_round);
        // If there are no current certificates, set the leader certificate to 'None', and return early.
        if current_certificates.is_empty() {
            // Set the leader certificate to 'None'.
            *self.leader_certificate.write() = None;
            return false;
        }

        // Retrieve the committee lookback of the current round.
        let committee_lookback = match self.ledger().get_committee_lookback_for_round(current_round) {
            Ok(committee) => committee,
            Err(err) => {
                err.log_error(format!(
                    "BFT failed to retrieve the committee lookback for the even round {current_round}"
                ));
                return false;
            }
        };
        // Determine the leader of the current round.
        let leader = match self.ledger().latest_leader() {
            Some((cached_round, cached_leader)) if cached_round == current_round => cached_leader,
            _ => {
                // Compute the leader for the current round.
                let computed_leader = match committee_lookback.get_leader(current_round) {
                    Ok(leader) => leader,
                    Err(err) => {
                        err.log_error(format!("BFT failed to compute the leader for the even round {current_round}"));
                        return false;
                    }
                };

                // Cache the computed leader.
                self.ledger().update_latest_leader(current_round, computed_leader);

                computed_leader
            }
        };
        // Find and set the leader certificate, if the leader was present in the current even round.
        let leader_certificate = current_certificates.iter().find(|certificate| certificate.author() == leader);
        *self.leader_certificate.write() = leader_certificate.cloned();

        self.is_even_round_ready_for_next_round(current_certificates, committee_lookback, current_round)
    }

    /// Returns 'true' if the quorum threshold `(N - f)` is reached for this round under one of the following conditions:
    ///  - If the leader certificate is set for the current even round.
    ///  - The timer for the leader certificate has expired.
    fn is_even_round_ready_for_next_round(
        &self,
        certificates: IndexSet<BatchCertificate<N>>,
        committee: Committee<N>,
        current_round: u64,
    ) -> bool {
        // Retrieve the authors for the current round.
        let authors = certificates.into_iter().map(|c| c.author()).collect();
        // Check if quorum threshold is reached.
        if !committee.is_quorum_threshold_reached(&authors) {
            trace!("BFT failed to reach quorum threshold in even round {current_round}");
            return false;
        }
        // If the leader certificate is set for the current even round, return 'true'.
        if let Some(leader_certificate) = self.leader_certificate.read().as_ref() {
            if leader_certificate.round() == current_round {
                return true;
            }
        }
        // If the timer has expired, and we can achieve quorum threshold (N - f) without the leader, return 'true'.
        if self.is_timer_expired() {
            debug!("BFT (timer expired) - Advancing from round {current_round} to the next round (without the leader)");
            return true;
        }
        // Otherwise, return 'false'.
        false
    }

    /// Returns `true` if the timer for the leader certificate has expired.
    ///
    /// This is always true for a new BFT instance.
    fn is_timer_expired(&self) -> bool {
        self.leader_certificate_timer.load(Ordering::SeqCst) + MAX_LEADER_CERTIFICATE_DELAY_IN_SECS <= now()
    }

    /// Returns 'true' if the quorum threshold `(N - f)` is reached for this round under one of the following conditions:
    ///  - The leader certificate is `None`.
    ///  - The leader certificate is not included up to availability threshold `(f + 1)` (in the previous certificates of the current round).
    ///  - The leader certificate timer has expired.
    fn is_leader_quorum_or_nonleaders_available(&self, odd_round: u64) -> bool {
        // Retrieve the current round.
        let current_round = self.storage().current_round();
        // Ensure the current round matches the given round.
        if current_round != odd_round {
            warn!("BFT storage (at round {current_round}) is out of sync with the current odd round {odd_round}");
            return false;
        }
        // If the current round is even, return false.
        if current_round % 2 != 1 {
            error!("BFT does not compute stakes for the leader certificate in an even round");
            return false;
        }
        // Retrieve the certificates for the current round.
        let current_certificates = self.storage().get_certificates_for_round(current_round);
        // Retrieve the committee lookback for the current round.
        let committee_lookback = match self.ledger().get_committee_lookback_for_round(current_round) {
            Ok(committee) => committee,
            Err(err) => {
                err.log_error(format!(
                    "BFT failed to retrieve the committee lookback for the odd round {current_round}"
                ));
                return false;
            }
        };
        // Retrieve the authors of the current certificates.
        let authors = current_certificates.clone().into_iter().map(|c| c.author()).collect();
        // Check if quorum threshold is reached.
        if !committee_lookback.is_quorum_threshold_reached(&authors) {
            trace!("BFT failed reach quorum threshold in odd round {current_round}. ");
            return false;
        }
        // Retrieve the leader certificate.
        let Some(leader_certificate) = self.leader_certificate.read().clone() else {
            // If there is no leader certificate for the previous round, return 'true'.
            return true;
        };
        // Compute the stake for the leader certificate.
        let (stake_with_leader, stake_without_leader) = self.compute_stake_for_leader_certificate(
            leader_certificate.id(),
            current_certificates,
            &committee_lookback,
        );
        // Return 'true' if any of the following conditions hold:
        stake_with_leader >= committee_lookback.availability_threshold()
            || stake_without_leader >= committee_lookback.quorum_threshold()
            || self.is_timer_expired()
    }

    /// Computes the amount of stake that has & has not signed for the leader certificate.
    fn compute_stake_for_leader_certificate(
        &self,
        leader_certificate_id: Field<N>,
        current_certificates: IndexSet<BatchCertificate<N>>,
        current_committee: &Committee<N>,
    ) -> (u64, u64) {
        // If there are no current certificates, return early.
        if current_certificates.is_empty() {
            return (0, 0);
        }

        // Initialize a tracker for the stake with the leader.
        let mut stake_with_leader = 0u64;
        // Initialize a tracker for the stake without the leader.
        let mut stake_without_leader = 0u64;
        // Iterate over the current certificates.
        for certificate in current_certificates {
            // Retrieve the stake for the author of the certificate.
            let stake = current_committee.get_stake(certificate.author());
            // Determine if the certificate includes the leader.
            match certificate.previous_certificate_ids().iter().any(|id| *id == leader_certificate_id) {
                // If the certificate includes the leader, add the stake to the stake with the leader.
                true => stake_with_leader = stake_with_leader.saturating_add(stake),
                // If the certificate does not include the leader, add the stake to the stake without the leader.
                false => stake_without_leader = stake_without_leader.saturating_add(stake),
            }
        }
        // Return the stake with the leader, and the stake without the leader.
        (stake_with_leader, stake_without_leader)
    }
}

impl<N: Network> BFT<N> {
    /// Stores the certificate in the DAG, and attempts to commit one or more anchors.
    async fn update_dag<const ALLOW_LEDGER_ACCESS: bool, const IS_SYNCING: bool>(
        &self,
        certificate: BatchCertificate<N>,
    ) -> Result<()> {
        // Acquire the BFT lock.
        let _lock = self.lock.lock().await;

        // Retrieve the round of the new certificate to add to the DAG.
        let certificate_round = certificate.round();

        // Insert the certificate into the DAG.
        self.dag.write().insert(certificate);

        // Get the previous round number.
        let commit_round = certificate_round.saturating_sub(1);

        // Leaders are elected in even rounds.
        // If the previous round is odd, the current round cannot commit any leader certs.
        if commit_round % 2 != 0 || commit_round < 2 {
            return Ok(());
        }
        // If the commit round is at or below the last committed round, return early.
        if commit_round <= self.dag.read().last_committed_round() {
            return Ok(());
        }

        /* Proceeding to check if the leader is ready to be committed. */
        trace!("Checking if the leader is ready to be committed for round {commit_round}...");

        // Retrieve the committee lookback for the commit round.
        let Ok(committee_lookback) = self.ledger().get_committee_lookback_for_round(commit_round) else {
            bail!("BFT failed to retrieve the committee lookback for commit round {commit_round}");
        };

        // Either retrieve the cached leader or compute it.
        let leader = match self.ledger().latest_leader() {
            Some((cached_round, cached_leader)) if cached_round == commit_round => cached_leader,
            _ => {
                // Compute the leader for the commit round.
                let Ok(computed_leader) = committee_lookback.get_leader(commit_round) else {
                    bail!("BFT failed to compute the leader for commit round {commit_round}");
                };

                // Cache the computed leader.
                self.ledger().update_latest_leader(commit_round, computed_leader);

                computed_leader
            }
        };

        // Retrieve the leader certificate for the commit round.
        let Some(leader_certificate) = self.dag.read().get_certificate_for_round_with_author(commit_round, leader)
        else {
            trace!("BFT did not find the leader certificate for commit round {commit_round} yet");
            return Ok(());
        };
        // Retrieve all of the certificates for the **certificate** round.
        let Some(certificates) = self.dag.read().get_certificates_for_round(certificate_round) else {
            // TODO (howardwu): Investigate how many certificates we should have at this point.
            bail!("BFT failed to retrieve the certificates for certificate round {certificate_round}");
        };
        // Retrieve the committee lookback for the certificate round (i.e. the round just after the commit round).
        let Ok(certificate_committee_lookback) = self.ledger().get_committee_lookback_for_round(certificate_round)
        else {
            bail!("BFT failed to retrieve the committee lookback for certificate round {certificate_round}");
        };
        // Construct a set over the authors who included the leader's certificate in the certificate round.
        let authors = certificates
            .values()
            .filter_map(|c| match c.previous_certificate_ids().contains(&leader_certificate.id()) {
                true => Some(c.author()),
                false => None,
            })
            .collect();
        // Check if the leader is ready to be committed.
        if !certificate_committee_lookback.is_availability_threshold_reached(&authors) {
            // If the leader is not ready to be committed, return early.
            trace!("BFT is not ready to commit {commit_round}");
            return Ok(());
        }

        /* Proceeding to commit the leader. */
        info!("Proceeding to commit round {commit_round} with leader '{}'", fmt_id(leader));

        // Commit the leader certificate, and all previous leader certificates since the last committed round.
        self.commit_leader_certificate::<ALLOW_LEDGER_ACCESS, IS_SYNCING>(leader_certificate).await
    }

    /// Commits the leader certificate, and all previous leader certificates since the last committed round.
    async fn commit_leader_certificate<const ALLOW_LEDGER_ACCESS: bool, const IS_SYNCING: bool>(
        &self,
        leader_certificate: BatchCertificate<N>,
    ) -> Result<()> {
        // Fetch the leader round.
        let latest_leader_round = leader_certificate.round();
        // Determine the list of all previous leader certificates since the last committed round.
        // The order of the leader certificates is from **newest** to **oldest**.
        let mut leader_certificates = vec![leader_certificate.clone()];
        {
            // Retrieve the leader round.
            let leader_round = leader_certificate.round();

            let mut current_certificate = leader_certificate;
            for round in (self.dag.read().last_committed_round() + 2..=leader_round.saturating_sub(2)).rev().step_by(2)
            {
                // Retrieve the previous committee for the leader round.
                let previous_committee_lookback =
                    self.ledger().get_committee_lookback_for_round(round).with_context(|| {
                        format!("BFT failed to retrieve a previous committee lookback for the even round {round}")
                    })?;

                // Either retrieve the cached leader or compute it.
                let leader = match self.ledger().latest_leader() {
                    Some((cached_round, cached_leader)) if cached_round == round => cached_leader,
                    _ => {
                        // Compute the leader for the commit round.
                        let computed_leader = previous_committee_lookback
                            .get_leader(round)
                            .with_context(|| format!("BFT failed to compute the leader for the even round {round}"))?;

                        // Cache the computed leader.
                        self.ledger().update_latest_leader(round, computed_leader);

                        computed_leader
                    }
                };
                // Retrieve the previous leader certificate.
                let Some(previous_certificate) = self.dag.read().get_certificate_for_round_with_author(round, leader)
                else {
                    continue;
                };
                // Determine if there is a path between the previous certificate and the current certificate.
                if self.is_linked(previous_certificate.clone(), current_certificate.clone())? {
                    // Add the previous leader certificate to the list of certificates to commit.
                    leader_certificates.push(previous_certificate.clone());
                    // Update the current certificate to the previous leader certificate.
                    current_certificate = previous_certificate;
                }
            }
        }

        // Iterate over the leader certificates to commit.
        for leader_certificate in leader_certificates.into_iter().rev() {
            // Retrieve the leader certificate round.
            let leader_round = leader_certificate.round();
            // Compute the commit subdag.
            let commit_subdag = match self.order_dag_with_dfs::<ALLOW_LEDGER_ACCESS>(leader_certificate) {
                Ok(subdag) => subdag,
                Err(e) => bail!("BFT failed to order the DAG with DFS - {e}"),
            };
            // If the node is not syncing, trigger consensus, as this will build a new block for the ledger.
            if !IS_SYNCING {
                // Initialize a map for the deduped transmissions.
                let mut transmissions = IndexMap::new();
                // Initialize a map for the deduped transaction ids.
                let mut seen_transaction_ids = IndexSet::new();
                // Initialize a map for the deduped solution ids.
                let mut seen_solution_ids = IndexSet::new();
                // Start from the oldest leader certificate.
                for certificate in commit_subdag.values().flatten() {
                    // Retrieve the transmissions.
                    for transmission_id in certificate.transmission_ids() {
                        // If the transaction ID or solution ID already exists in the map, skip it.
                        // Note: This additional check is done to ensure that we do not include duplicate
                        // transaction IDs or solution IDs that may have a different transmission ID.
                        match transmission_id {
                            TransmissionID::Solution(solution_id, _) => {
                                // If the solution already exists, skip it.
                                if seen_solution_ids.contains(&solution_id) {
                                    continue;
                                }
                            }
                            TransmissionID::Transaction(transaction_id, _) => {
                                // If the transaction already exists, skip it.
                                if seen_transaction_ids.contains(transaction_id) {
                                    continue;
                                }
                            }
                            TransmissionID::Ratification => {
                                bail!("Ratifications are currently not supported in the BFT.")
                            }
                        }
                        // If the transmission already exists in the map, skip it.
                        if transmissions.contains_key(transmission_id) {
                            continue;
                        }
                        // If the transmission already exists in the ledger, skip it.
                        // Note: On failure to read from the ledger, we skip including this transmission, out of safety.
                        if self.ledger().contains_transmission(transmission_id).unwrap_or(true) {
                            continue;
                        }
                        // Retrieve the transmission.
                        let Some(transmission) = self.storage().get_transmission(*transmission_id) else {
                            bail!(
                                "BFT failed to retrieve transmission '{}.{}' from round {}",
                                fmt_id(transmission_id),
                                fmt_id(transmission_id.checksum().unwrap_or_default()).dimmed(),
                                certificate.round()
                            );
                        };
                        // Insert the transaction ID or solution ID into the map.
                        match transmission_id {
                            TransmissionID::Solution(id, _) => {
                                seen_solution_ids.insert(id);
                            }
                            TransmissionID::Transaction(id, _) => {
                                seen_transaction_ids.insert(id);
                            }
                            TransmissionID::Ratification => {}
                        }
                        // Add the transmission to the set.
                        transmissions.insert(*transmission_id, transmission);
                    }
                }
                // Trigger consensus, as this will build a new block for the ledger.
                // Construct the subdag.
                let subdag = Subdag::from(commit_subdag.clone())?;
                // Retrieve the anchor round.
                let anchor_round = subdag.anchor_round();
                // Retrieve the number of transmissions.
                let num_transmissions = transmissions.len();
                // Retrieve metadata about the subdag.
                let subdag_metadata = subdag.iter().map(|(round, c)| (*round, c.len())).collect::<Vec<_>>();

                // Ensure the subdag anchor round matches the leader round.
                ensure!(
                    anchor_round == leader_round,
                    "BFT failed to commit - the subdag anchor round {anchor_round} does not match the leader round {leader_round}",
                );

                // Trigger consensus.
                if let Some(consensus_sender) = self.consensus_sender.get() {
                    // Initialize a callback sender and receiver.
                    let (callback_sender, callback_receiver) = oneshot::channel();
                    // Send the subdag and transmissions to consensus.
                    consensus_sender.tx_consensus_subdag.send((subdag, transmissions, callback_sender)).await?;
                    // Await the callback to continue.
                    match callback_receiver.await {
                        Ok(Ok(())) => (), // continue
                        Ok(Err(err)) => {
                            err.log_error(format!("BFT failed to advance the subdag for round {anchor_round}"));
                            return Ok(());
                        }
                        Err(err) => {
                            err.log_error(format!(
                                "BFT failed to receive consensus the callback for round {anchor_round}"
                            ));
                            return Ok(());
                        }
                    }
                }

                info!(
                    "\n\nCommitting a subdag from round {anchor_round} with {num_transmissions} transmissions: {subdag_metadata:?}\n"
                );
            }

            // Update the DAG, as the subdag was successfully included into a block.
            let mut dag_write = self.dag.write();
            for certificate in commit_subdag.values().flatten() {
                dag_write.commit(certificate, self.storage().max_gc_rounds());
            }

            // Update the validator telemetry.
            #[cfg(feature = "telemetry")]
            self.primary().gateway().validator_telemetry().insert_subdag(&Subdag::from(commit_subdag)?);
        }

        // Perform garbage collection based on the latest committed leader round.
        // The protocol guarantees that validators commit the same anchors in the same order,
        // but they may do so in different chunks of anchors,
        // where 'chunk' refers to the vector of certificates that the loop just above iterates over.
        // Doing garbage collection at the end of each chunk (as we do here),
        // as opposed to after each certificate in the chunk (if we moved this call inside the loop, at the end),
        // may give raise to a discrepancy between the DAGs of different validators who commit different chunks:
        // one validator may have more certificates than the other, not yet garbage collected.
        // However, when `order_dag_with_dfs()` collects the sub-DAG to commit from an anchor,
        // it excludes certificates that are below the GC round,
        // so the possible aforementioned discrepancy between DAGs should not affect the consensus.
        // That exclusion in `order_dag_with_dfs()` is critical to prevent forking,
        // so long as garbage collection is done after each chunk.
        // If garbage collection were done after each committed certificate,
        // that exclusion in `order_dag_with_dfs()` should be unnecessary.
        self.storage().garbage_collect_certificates(latest_leader_round);

        Ok(())
    }

    /// Returns the subdag of batch certificates to commit.
    fn order_dag_with_dfs<const ALLOW_LEDGER_ACCESS: bool>(
        &self,
        leader_certificate: BatchCertificate<N>,
    ) -> Result<BTreeMap<u64, IndexSet<BatchCertificate<N>>>> {
        // Initialize a map for the certificates to commit.
        let mut commit = BTreeMap::<u64, IndexSet<_>>::new();
        // Initialize a set for the already ordered certificates.
        let mut already_ordered = HashSet::new();
        // Initialize a buffer for the certificates to order.
        let mut buffer = vec![leader_certificate];
        // Iterate over the certificates to order.
        while let Some(certificate) = buffer.pop() {
            // Insert the certificate into the map.
            commit.entry(certificate.round()).or_default().insert(certificate.clone());

            // Check if the previous certificate is below the GC round.
            // This is currently a critical check to prevent forking,
            // as explained in the comment at the end of `commit_leader_certificate()`,
            // just before the call to garbage collection.
            let previous_round = certificate.round().saturating_sub(1);
            if previous_round + self.storage().max_gc_rounds() <= self.dag.read().last_committed_round() {
                continue;
            }
            // Iterate over the previous certificate IDs.
            // Note: Using '.rev()' ensures we remain order-preserving (i.e. "left-to-right" on each level),
            // because this 'while' loop uses 'pop()' to retrieve the next certificate to order.
            for previous_certificate_id in certificate.previous_certificate_ids().iter().rev() {
                // If the previous certificate is already ordered, continue.
                if already_ordered.contains(previous_certificate_id) {
                    continue;
                }
                // If the previous certificate was recently committed, continue.
                if self.dag.read().is_recently_committed(previous_round, *previous_certificate_id) {
                    continue;
                }
                // If the previous certificate already exists in the ledger, continue.
                if ALLOW_LEDGER_ACCESS && self.ledger().contains_certificate(previous_certificate_id).unwrap_or(false) {
                    continue;
                }

                // Retrieve the previous certificate.
                let previous_certificate = {
                    // Start by retrieving the previous certificate from the DAG.
                    match self.dag.read().get_certificate_for_round_with_id(previous_round, *previous_certificate_id) {
                        // If the previous certificate is found, return it.
                        Some(previous_certificate) => previous_certificate,
                        // If the previous certificate is not found, retrieve it from the storage.
                        None => match self.storage().get_certificate(*previous_certificate_id) {
                            // If the previous certificate is found, return it.
                            Some(previous_certificate) => previous_certificate,
                            // Otherwise, the previous certificate is missing, and throw an error.
                            None => bail!(
                                "Missing previous certificate {} for round {previous_round}",
                                fmt_id(previous_certificate_id)
                            ),
                        },
                    }
                };
                // Insert the previous certificate into the set of already ordered certificates.
                already_ordered.insert(previous_certificate.id());
                // Insert the previous certificate into the buffer.
                buffer.push(previous_certificate);
            }
        }
        // Ensure we only retain certificates that are above the GC round.
        commit.retain(|round, _| round + self.storage().max_gc_rounds() > self.dag.read().last_committed_round());
        // Return the certificates to commit.
        Ok(commit)
    }

    /// Returns `true` if there is a path from the previous certificate to the current certificate.
    fn is_linked(
        &self,
        previous_certificate: BatchCertificate<N>,
        current_certificate: BatchCertificate<N>,
    ) -> Result<bool> {
        // Initialize the list containing the traversal.
        let mut traversal = vec![current_certificate.clone()];
        // Iterate over the rounds from the current certificate to the previous certificate.
        for round in (previous_certificate.round()..current_certificate.round()).rev() {
            // Retrieve all of the certificates for this past round.
            let Some(certificates) = self.dag.read().get_certificates_for_round(round) else {
                // This is a critical error, as the traversal should have these certificates.
                // If this error is hit, it is likely that the maximum GC rounds should be increased.
                bail!("BFT failed to retrieve the certificates for past round {round}");
            };
            // Filter the certificates to only include those that are in the traversal.
            traversal = certificates
                .into_values()
                .filter(|p| traversal.iter().any(|c| c.previous_certificate_ids().contains(&p.id())))
                .collect();
        }
        Ok(traversal.contains(&previous_certificate))
    }
}

impl<N: Network> BFT<N> {
    /// Shuts down the BFT.
    pub async fn shut_down(&self) {
        info!("Shutting down the BFT...");
        // Acquire the lock.
        let _lock = self.lock.lock().await;
        // Shut down the primary.
        self.primary.shut_down().await;
    }
}

#[cfg(test)]
mod tests;
