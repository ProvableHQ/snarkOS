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
    Gateway,
    MAX_FETCH_TIMEOUT_IN_MS,
    PRIMARY_PING_IN_MS,
    Transport,
    events::DataBlocks,
    helpers::{BFTSender, Pending, Storage, SyncReceiver, fmt_id, max_redundant_requests},
    spawn_blocking,
};
use snarkos_node_bft_events::{CertificateRequest, CertificateResponse, Event};
use snarkos_node_bft_ledger_service::LedgerService;
use snarkos_node_sync::{BLOCK_REQUEST_BATCH_DELAY, BlockSync, Ping, PrepareSyncRequest, locators::BlockLocators};
use snarkvm::{
    console::{network::Network, types::Field},
    ledger::{authority::Authority, block::Block, narwhal::BatchCertificate},
    prelude::{cfg_into_iter, cfg_iter},
};

use anyhow::{Result, anyhow, bail};
use indexmap::IndexMap;
#[cfg(feature = "locktick")]
use locktick::{parking_lot::Mutex, tokio::Mutex as TMutex};
#[cfg(not(feature = "locktick"))]
use parking_lot::Mutex;
#[cfg(not(feature = "serial"))]
use rayon::prelude::*;
use std::{
    collections::{BTreeMap, HashMap},
    future::Future,
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
#[cfg(not(feature = "locktick"))]
use tokio::sync::Mutex as TMutex;
use tokio::{
    sync::{OnceCell, oneshot},
    task::JoinHandle,
};

/// Block synchronization logic for validators.
///
/// Synchronization works differently for nodes that act as validators in AleoBFT;
/// In the common case, validators generate blocks after receiving an anchor block that has been accepted
/// by a supermajority of the committee instead of fetching entire blocks from other nodes.
/// However, if a validator does not have an up-to-date DAG, it might still fetch entire blocks from other nodes.
///
/// This struct also manages fetching certificates from other validators during normal operation,
/// and blocks when falling behind.
///
/// Finally, `Sync` handles synchronization of blocks with the validator's local storage:
/// it loads blocks from the storage on startup and writes new blocks to the storage after discovering them.
#[derive(Clone)]
pub struct Sync<N: Network> {
    /// The gateway enables communication with other validators.
    gateway: Gateway<N>,
    /// The storage.
    storage: Storage<N>,
    /// The ledger service.
    ledger: Arc<dyn LedgerService<N>>,
    /// The block synchronization logic.
    block_sync: Arc<BlockSync<N>>,
    /// The pending certificates queue.
    pending: Arc<Pending<Field<N>, BatchCertificate<N>>>,
    /// The BFT sender.
    bft_sender: Arc<OnceCell<BFTSender<N>>>,
    /// Handles to the spawned background tasks.
    handles: Arc<Mutex<Vec<JoinHandle<()>>>>,
    /// The response lock.
    response_lock: Arc<TMutex<()>>,
    /// The sync lock. Ensures that only one task syncs the ledger at a time.
    sync_lock: Arc<TMutex<()>>,
    /// The latest block responses.
    ///
    /// This is used in [`Sync::sync_storage_with_block()`] to accumulate blocks  whose addition to the ledger is
    /// deferred until certain checks pass.
    /// Blocks need to be processed in order, hence a BTree map.
    ///
    /// Whenever a new block is added to this map, BlockSync::set_sync_height needs to be called.
    latest_block_responses: Arc<TMutex<BTreeMap<u32, Block<N>>>>,
}

impl<N: Network> Sync<N> {
    /// Initializes a new sync instance.
    pub fn new(
        gateway: Gateway<N>,
        storage: Storage<N>,
        ledger: Arc<dyn LedgerService<N>>,
        block_sync: Arc<BlockSync<N>>,
    ) -> Self {
        // Return the sync instance.
        Self {
            gateway,
            storage,
            ledger,
            block_sync,
            pending: Default::default(),
            bft_sender: Default::default(),
            handles: Default::default(),
            response_lock: Default::default(),
            sync_lock: Default::default(),
            latest_block_responses: Default::default(),
        }
    }

    /// Initializes the sync module and sync the storage with the ledger at bootup.
    pub async fn initialize(&self, bft_sender: Option<BFTSender<N>>) -> Result<()> {
        // If a BFT sender was provided, set it.
        if let Some(bft_sender) = bft_sender {
            self.bft_sender.set(bft_sender).expect("BFT sender already set in gateway");
        }

        info!("Syncing storage with the ledger...");

        // Sync the storage with the ledger.
        self.sync_storage_with_ledger_at_bootup().await?;

        debug!("Finished initial block synchronization at startup");
        Ok(())
    }

    /// Sends the given batch of block requests to peers.
    ///
    /// Responses to block requests will eventually be processed by `Self::try_advancing_block_synchronization`.
    #[inline]
    async fn send_block_requests(
        &self,

        block_requests: Vec<(u32, PrepareSyncRequest<N>)>,
        sync_peers: IndexMap<SocketAddr, BlockLocators<N>>,
    ) {
        trace!("Prepared {num_requests} block requests", num_requests = block_requests.len());

        // Sends the block requests to the sync peers.
        for requests in block_requests.chunks(DataBlocks::<N>::MAXIMUM_NUMBER_OF_BLOCKS as usize) {
            if !self.block_sync.send_block_requests(&self.gateway, &sync_peers, requests).await {
                // Stop if we fail to process a batch of requests.
                break;
            }

            // Sleep to avoid triggering spam detection.
            tokio::time::sleep(BLOCK_REQUEST_BATCH_DELAY).await;
        }
    }

    /// Starts the sync module.
    ///
    /// When this function returns successfully, the sync module will have spawned background tasks
    /// that fetch blocks from other validators.
    pub async fn run(&self, ping: Option<Arc<Ping<N>>>, sync_receiver: SyncReceiver<N>) -> Result<()> {
        info!("Starting the sync module...");

        // Start the block sync loop.
        let self_ = self.clone();
        self.spawn(async move {
            // Sleep briefly to allow an initial primary ping to come in prior to entering the loop.
            // Ideally, a node does not consider itself synced when it has not received
            // any block locators from peers. However, in the initial bootup of validators,
            // this needs to happen, so we use this additional sleep as a grace period.
            tokio::time::sleep(Duration::from_millis(PRIMARY_PING_IN_MS)).await;
            loop {
                // Sleep briefly to avoid triggering spam detection.
                tokio::time::sleep(Duration::from_millis(PRIMARY_PING_IN_MS)).await;

                let new_blocks = self_.try_block_sync().await;
                if new_blocks {
                    if let Some(ping) = &ping {
                        match self_.get_block_locators() {
                            Ok(locators) => ping.update_block_locators(locators),
                            Err(err) => error!("Failed to update block locators: {err}"),
                        }
                    }
                }
            }
        });

        // Start the pending queue expiration loop.
        let self_ = self.clone();
        self.spawn(async move {
            loop {
                // Sleep briefly.
                tokio::time::sleep(Duration::from_millis(MAX_FETCH_TIMEOUT_IN_MS)).await;

                // Remove the expired pending transmission requests.
                let self__ = self_.clone();
                let _ = spawn_blocking!({
                    self__.pending.clear_expired_callbacks();
                    Ok(())
                });
            }
        });

        /* Set up callbacks for events from the Gateway */

        // Retrieve the sync receiver.
        let SyncReceiver {
            mut rx_block_sync_advance_with_sync_blocks,
            mut rx_block_sync_remove_peer,
            mut rx_block_sync_update_peer_locators,
            mut rx_certificate_request,
            mut rx_certificate_response,
        } = sync_receiver;

        // Process the block sync request to advance with sync blocks.
        // Each iteration of this loop is triggered by an incoming [`BlockResponse`],
        // which is initially handled by [`Gateway::inbound()`],
        // which calls [`SyncSender::advance_with_sync_blocks()`],
        // which calls [`tx_block_sync_advance_with_sync_blocks.send()`],
        // which causes the `rx_block_sync_advance_with_sync_blocks.recv()` call below to return.
        let self_ = self.clone();
        self.spawn(async move {
            while let Some((peer_ip, blocks, callback)) = rx_block_sync_advance_with_sync_blocks.recv().await {
                callback.send(self_.advance_with_sync_blocks(peer_ip, blocks).await).ok();
            }
        });

        // Process the block sync request to remove the peer.
        let self_ = self.clone();
        self.spawn(async move {
            while let Some(peer_ip) = rx_block_sync_remove_peer.recv().await {
                self_.remove_peer(peer_ip);
            }
        });

        // Process each block sync request to update peer locators.
        // Each iteration of this loop is triggered by an incoming [`PrimaryPing`],
        // which is initially handled by [`Gateway::inbound()`],
        // which calls [`SyncSender::update_peer_locators()`],
        // which calls [`tx_block_sync_update_peer_locators.send()`],
        // which causes the `rx_block_sync_update_peer_locators.recv()` call below to return.
        let self_ = self.clone();
        self.spawn(async move {
            while let Some((peer_ip, locators, callback)) = rx_block_sync_update_peer_locators.recv().await {
                let self_clone = self_.clone();
                tokio::spawn(async move {
                    callback.send(self_clone.update_peer_locators(peer_ip, locators)).ok();
                });
            }
        });

        // Process each certificate request.
        // Each iteration of this loop is triggered by an incoming [`CertificateRequest`],
        // which is initially handled by [`Gateway::inbound()`],
        // which calls [`tx_certificate_request.send()`],
        // which causes the `rx_certificate_request.recv()` call below to return.
        let self_ = self.clone();
        self.spawn(async move {
            while let Some((peer_ip, certificate_request)) = rx_certificate_request.recv().await {
                self_.send_certificate_response(peer_ip, certificate_request);
            }
        });

        // Process each certificate response.
        // Each iteration of this loop is triggered by an incoming [`CertificateResponse`],
        // which is initially handled by [`Gateway::inbound()`],
        // which calls [`tx_certificate_response.send()`],
        // which causes the `rx_certificate_response.recv()` call below to return.
        let self_ = self.clone();
        self.spawn(async move {
            while let Some((peer_ip, certificate_response)) = rx_certificate_response.recv().await {
                self_.finish_certificate_request(peer_ip, certificate_response);
            }
        });

        Ok(())
    }

    /// Execute one iteration of block synchronization.
    ///
    /// Returns true if we made progress. Returning true does *not* imply being fully synced.
    ///
    /// This is called periodically by a tokio background task spawned in `Self::run`.
    /// Some unit tests also call this function directly to manually trigger block synchronization.
    pub(crate) async fn try_block_sync(&self) -> bool {
        // Check if any existing requests can be removed.
        // We should do this even if we cannot block sync, to ensure
        // there are no dangling block requests.
        let new_requests = self.block_sync.handle_block_request_timeouts(&self.gateway);
        if let Some((sync_peers, requests)) = new_requests {
            self.send_block_requests(sync_peers, requests).await;
        }

        // Do not attempt to sync if there are no blocks to sync.
        // This prevents redundant log messages and performing unnecessary computation.
        if !self.block_sync.can_block_sync() {
            trace!("No blocks to sync");
            return false;
        }

        // Prepare the block requests, if any.
        // In the process, we update the state of `is_block_synced` for the sync module.
        let (sync_peers, requests) = self.block_sync.prepare_block_requests();
        self.send_block_requests(sync_peers, requests).await;

        // Sync the storage with the blocks
        match self.try_advancing_block_synchronization().await {
            Ok(new_blocks) => new_blocks,
            Err(err) => {
                error!("Block synchronization failed - {err}");
                false
            }
        }
    }
}

// Callbacks used when receiving messages from the Gateway
impl<N: Network> Sync<N> {
    /// We received a block response and can (possibly) advance synchronization.
    async fn advance_with_sync_blocks(&self, peer_ip: SocketAddr, blocks: Vec<Block<N>>) -> Result<()> {
        // Verify that the response is valid and add it to block sync.
        self.block_sync.insert_block_responses(peer_ip, blocks)?;

        // Try to process responses stored in BlockSync.
        // Note: Do not call `self.block_sync.try_advancing_block_synchronziation` here as it will process
        // and remove any completed requests, which means the call to `sync_storage_with_blocks` will not process
        // them as expected.
        self.try_advancing_block_synchronization().await?;

        Ok(())
    }

    /// We received new peer locators during a Ping.
    fn update_peer_locators(&self, peer_ip: SocketAddr, locators: BlockLocators<N>) -> Result<()> {
        self.block_sync.update_peer_locators(peer_ip, locators)
    }

    /// A peer disconnected.
    fn remove_peer(&self, peer_ip: SocketAddr) {
        self.block_sync.remove_peer(&peer_ip);
    }

    #[cfg(test)]
    pub fn test_update_peer_locators(&self, peer_ip: SocketAddr, locators: BlockLocators<N>) -> Result<()> {
        self.update_peer_locators(peer_ip, locators)
    }
}

// Methods to manage storage.
impl<N: Network> Sync<N> {
    /// Syncs the storage with the ledger at bootup.
    ///
    /// This is called when starting the validator and after finishing a sync without BFT.
    async fn sync_storage_with_ledger_at_bootup(&self) -> Result<()> {
        // Retrieve the latest block in the ledger.
        let latest_block = self.ledger.latest_block();

        // Retrieve the block height.
        let block_height = latest_block.height();
        // Determine the maximum number of blocks corresponding to rounds
        // that would not have been garbage collected, i.e. that would be kept in storage.
        // Since at most one block is created every two rounds,
        // this is half of the maximum number of rounds kept in storage.
        let max_gc_blocks = u32::try_from(self.storage.max_gc_rounds())?.saturating_div(2);
        // Determine the earliest height of blocks corresponding to rounds kept in storage,
        // conservatively set to the block height minus the maximum number of blocks calculated above.
        // By virtue of the BFT protocol, we can guarantee that all GC range blocks will be loaded.
        let gc_height = block_height.saturating_sub(max_gc_blocks);
        // Retrieve the blocks.
        let blocks = self.ledger.get_blocks(gc_height..block_height.saturating_add(1))?;

        // Acquire the sync lock.
        let _lock = self.sync_lock.lock().await;

        debug!("Syncing storage with the ledger from block {} to {}...", gc_height, block_height.saturating_add(1));

        /* Sync storage */

        // Sync the height with the block.
        self.storage.sync_height_with_block(latest_block.height());
        // Sync the round with the block.
        self.storage.sync_round_with_block(latest_block.round());
        // Perform GC on the latest block round.
        self.storage.garbage_collect_certificates(latest_block.round());
        // Iterate over the blocks.
        for block in &blocks {
            // If the block authority is a sub-DAG, then sync the batch certificates with the block.
            // Note that the block authority is always a sub-DAG in production;
            // beacon signatures are only used for testing,
            // and as placeholder (irrelevant) block authority in the genesis block.
            if let Authority::Quorum(subdag) = block.authority() {
                // Reconstruct the unconfirmed transactions.
                let unconfirmed_transactions = cfg_iter!(block.transactions())
                    .filter_map(|tx| {
                        tx.to_unconfirmed_transaction().map(|unconfirmed| (unconfirmed.id(), unconfirmed)).ok()
                    })
                    .collect::<HashMap<_, _>>();

                // Iterate over the certificates.
                for certificates in subdag.values().cloned() {
                    cfg_into_iter!(certificates).for_each(|certificate| {
                        self.storage.sync_certificate_with_block(block, certificate, &unconfirmed_transactions);
                    });
                }

                // Update the validator telemetry.
                #[cfg(feature = "telemetry")]
                self.gateway.validator_telemetry().insert_subdag(subdag);
            }
        }

        /* Sync the BFT DAG */

        // Construct a list of the certificates.
        let certificates = blocks
            .iter()
            .flat_map(|block| {
                match block.authority() {
                    // If the block authority is a beacon, then skip the block.
                    Authority::Beacon(_) => None,
                    // If the block authority is a subdag, then retrieve the certificates.
                    Authority::Quorum(subdag) => Some(subdag.values().flatten().cloned().collect::<Vec<_>>()),
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        // If a BFT sender was provided, send the certificates to the BFT.
        if let Some(bft_sender) = self.bft_sender.get() {
            // Await the callback to continue.
            if let Err(e) = bft_sender.tx_sync_bft_dag_at_bootup.send(certificates).await {
                bail!("Failed to update the BFT DAG from sync: {e}");
            }
        }

        self.block_sync.set_sync_height(block_height);

        Ok(())
    }

    /// Returns which height we are synchronized to.
    /// If there are queued block responses, this might be higher than the latest block in the ledger.
    async fn compute_sync_height(&self) -> u32 {
        let ledger_height = self.ledger.latest_block_height();
        let mut responses = self.latest_block_responses.lock().await;

        // Remove any old responses.
        responses.retain(|height, _| *height > ledger_height);

        // Ensure the returned value is always greater or equal than ledger height.
        responses.last_key_value().map(|(height, _)| *height).unwrap_or(0).max(ledger_height)
    }

    /// Aims to advance synchronization using any recent block responses received from peers.
    ///
    /// This is the validator's version of `BlockSync::try_advancing_block_synchronization`
    /// and is called periodically at runtime.
    ///
    /// This returns Ok(true) if we successfully advanced the ledger by at least one new block.
    ///
    /// A key difference to `BlockSync`'s versions is that it will only add blocks to the ledger once they have been confirmed by the network.
    /// If blocks are not confirmed yet, they will be kept in [`Self::latest_block_responses`].
    /// It will also pass certificates from synced blocks to the BFT module so that consensus can progress as expected
    /// (see [`Self::sync_storage_with_block`] for more details).
    ///
    /// If the node falls behind more than GC rounds, this function calls [`Self::sync_storage_without_bft`] instead,
    /// which syncs without updating the BFT state.
    async fn try_advancing_block_synchronization(&self) -> Result<bool> {
        // Acquire the response lock.
        let _lock = self.response_lock.lock().await;

        // For sanity, set the sync height again.
        // (if the sync height is already larger or equal, this is a noop)
        let ledger_height = self.ledger.latest_block_height();
        self.block_sync.set_sync_height(ledger_height);

        // Retrieve the maximum block height of the peers.
        let tip = self.block_sync.find_sync_peers().map(|(x, _)| x.into_values().max().unwrap_or(0)).unwrap_or(0);

        // Determine the maximum number of blocks corresponding to rounds
        // that would not have been garbage collected, i.e. that would be kept in storage.
        // Since at most one block is created every two rounds,
        // this is half of the maximum number of rounds kept in storage.
        let max_gc_blocks = u32::try_from(self.storage.max_gc_rounds())?.saturating_div(2);

        // Updates sync state and returns the error (if any).
        let cleanup = |start_height, current_height, error| {
            let new_blocks = current_height > start_height;

            // Make the underlying `BlockSync` instance aware of the new sync height.
            if new_blocks {
                self.block_sync.set_sync_height(current_height);
            }

            if let Some(err) = error { Err(err) } else { Ok(new_blocks) }
        };

        // Determine the earliest height of blocks corresponding to rounds kept in storage,
        // conservatively set to the block height minus the maximum number of blocks calculated above.
        // By virtue of the BFT protocol, we can guarantee that all GC range blocks will be loaded.
        let max_gc_height = tip.saturating_sub(max_gc_blocks);
        let within_gc = (ledger_height + 1) > max_gc_height;

        if within_gc {
            // Retrieve the current height, based on the ledger height and the
            // (unconfirmed) blocks that are already queued up.
            let start_height = self.compute_sync_height().await;

            // For sanity, update the sync height before starting.
            // (if this is lower or equal to the current sync height, this is a noop)
            self.block_sync.set_sync_height(start_height);

            // The height is incremented as blocks are added.
            let mut current_height = start_height;
            trace!("Try advancing with block responses (at block {current_height})");

            // If we already were within GC or successfully caught up with GC, try to advance BFT normally again.
            loop {
                let next_height = current_height + 1;
                let Some(block) = self.block_sync.peek_next_block(next_height) else {
                    break;
                };
                info!("Syncing the BFT to block {}...", block.height());
                // Sync the storage with the block.
                match self.sync_storage_with_block(block).await {
                    Ok(_) => {
                        // Update the current height if sync succeeds.
                        current_height = next_height;
                    }
                    Err(err) => {
                        // Mark the current height as processed in block_sync.
                        self.block_sync.remove_block_response(next_height);
                        return cleanup(start_height, current_height, Some(err));
                    }
                }
            }

            cleanup(start_height, current_height, None)
        } else {
            info!("Block sync is too far behind other validators. Syncing without BFT.");

            // For non-BFT sync we need to start at the current height of the ledger,as blocks are immediately
            // added to it and not queue up in `latest_block_responses`.
            let start_height = ledger_height;
            let mut current_height = start_height;

            // For sanity, update the sync height before starting.
            // (if this is lower or equal to the current sync height, this is a noop)
            self.block_sync.set_sync_height(start_height);

            // Try to advance the ledger *to tip* without updating the BFT.
            // TODO(kaimast): why to tip and not to tip-GC?
            loop {
                let next_height = current_height + 1;

                let Some(block) = self.block_sync.peek_next_block(next_height) else {
                    break;
                };
                info!("Syncing the ledger to block {}...", block.height());

                // Sync the ledger with the block without BFT.
                match self.sync_ledger_with_block_without_bft(block).await {
                    Ok(_) => {
                        // Update the current height if sync succeeds.
                        current_height = next_height;
                    }
                    Err(err) => {
                        // Mark the current height as processed in block_sync.
                        self.block_sync.remove_block_response(next_height);
                        return cleanup(start_height, current_height, Some(err));
                    }
                }
            }

            // Sync the storage with the ledger if we should transition to the BFT sync.
            let within_gc = (current_height + 1) > max_gc_height;
            if within_gc {
                info!("Finished catching up with the network. Switching back to BFT sync.");
                if let Err(err) = self.sync_storage_with_ledger_at_bootup().await {
                    error!("BFT sync (with bootup routine) failed - {err}");
                }
            }

            cleanup(start_height, current_height, None)
        }
    }

    /// Syncs the ledger with the given block without updating the BFT.
    ///
    /// This is only used by `[Self::try_advancing_block_synchronization`].
    async fn sync_ledger_with_block_without_bft(&self, block: Block<N>) -> Result<()> {
        // Acquire the sync lock.
        let _lock = self.sync_lock.lock().await;

        let self_ = self.clone();
        tokio::task::spawn_blocking(move || {
            // Check the next block.
            self_.ledger.check_next_block(&block)?;
            // Attempt to advance to the next block.
            self_.ledger.advance_to_next_block(&block)?;

            // Sync the height with the block.
            self_.storage.sync_height_with_block(block.height());
            // Sync the round with the block.
            self_.storage.sync_round_with_block(block.round());
            // Mark the block height as processed in block_sync.
            self_.block_sync.remove_block_response(block.height());

            Ok(())
        })
        .await?
    }

    /// Advances the ledger by the given block and updates the storage accordingly.
    ///
    /// This also updates the DAG, and uses the DAG to ensure that the block's leader certificate
    /// meets the voter availability threshold (i.e. > f voting stake)
    /// or is reachable via a DAG path from a later leader certificate that does.
    /// Since performing this check requires DAG certificates from later blocks,
    /// the block is stored in `Sync::latest_block_responses`,
    /// and its addition to the ledger is deferred until the check passes.
    /// Several blocks may be stored in `Sync::latest_block_responses`
    /// before they can be all checked and added to the ledger.
    async fn sync_storage_with_block(&self, block: Block<N>) -> Result<()> {
        // Acquire the sync lock.
        let _lock = self.sync_lock.lock().await;

        // If this block has already been processed, return early.
        // TODO(kaimast): Should we remove the response here?
        if self.ledger.contains_block_height(block.height()) {
            debug!("Ledger is already synced with block at height {}. Will not sync.", block.height());
            return Ok(());
        }

        // Acquire the latest block responses lock.
        let mut latest_block_responses = self.latest_block_responses.lock().await;

        if latest_block_responses.contains_key(&block.height()) {
            debug!("An unconfirmed block is queued already for height {}. Will not sync.", block.height());
            return Ok(());
        }

        // If the block authority is a sub-DAG, then sync the batch certificates with the block.
        // Note that the block authority is always a sub-DAG in production;
        // beacon signatures are only used for testing,
        // and as placeholder (irrelevant) block authority in the genesis block.
        if let Authority::Quorum(subdag) = block.authority() {
            // Reconstruct the unconfirmed transactions.
            let unconfirmed_transactions = cfg_iter!(block.transactions())
                .filter_map(|tx| {
                    tx.to_unconfirmed_transaction().map(|unconfirmed| (unconfirmed.id(), unconfirmed)).ok()
                })
                .collect::<HashMap<_, _>>();

            // Iterate over the certificates.
            for certificates in subdag.values().cloned() {
                cfg_into_iter!(certificates.clone()).for_each(|certificate| {
                    // Sync the batch certificate with the block.
                    self.storage.sync_certificate_with_block(&block, certificate.clone(), &unconfirmed_transactions);
                });

                // Sync the BFT DAG with the certificates.
                for certificate in certificates {
                    // If a BFT sender was provided, send the certificate to the BFT.
                    // For validators, BFT spawns a receiver task in `BFT::start_handlers`.
                    if let Some(bft_sender) = self.bft_sender.get() {
                        // Await the callback to continue.
                        if let Err(err) = bft_sender.send_sync_bft(certificate).await {
                            bail!("Failed to sync certificate - {err}");
                        };
                    }
                }
            }
        }

        // Fetch the latest block height.
        let ledger_block_height = self.ledger.latest_block_height();

        // Insert the latest block response.
        latest_block_responses.insert(block.height(), block);
        // Clear the latest block responses of older blocks.
        latest_block_responses.retain(|height, _| *height > ledger_block_height);

        // Get a list of contiguous blocks from the latest block responses.
        let contiguous_blocks: Vec<Block<N>> = (ledger_block_height.saturating_add(1)..)
            .take_while(|&k| latest_block_responses.contains_key(&k))
            .filter_map(|k| latest_block_responses.get(&k).cloned())
            .collect();

        // Check if each block response, from the contiguous sequence just constructed,
        // is ready to be added to the ledger.
        // Ensure that the block's leader certificate meets the availability threshold
        // based on the certificates in the DAG just after the block's round.
        // If the availability threshold is not met,
        // process the next block and check if it is linked to the current block,
        // in the sense that there is a path in the DAG
        // from the next block's leader certificate
        // to the current block's leader certificate.
        // Note: We do not advance to the most recent block response because we would be unable to
        // validate if the leader certificate in the block has been certified properly.
        for next_block in contiguous_blocks.into_iter() {
            // Retrieve the height of the next block.
            let next_block_height = next_block.height();

            // Fetch the leader certificate and the relevant rounds.
            let leader_certificate = match next_block.authority() {
                Authority::Quorum(subdag) => subdag.leader_certificate().clone(),
                _ => bail!("Received a block with an unexpected authority type."),
            };
            let commit_round = leader_certificate.round();
            let certificate_round =
                commit_round.checked_add(1).ok_or_else(|| anyhow!("Integer overflow on round number"))?;

            // Get the committee lookback for the round just after the leader.
            let certificate_committee_lookback = self.ledger.get_committee_lookback_for_round(certificate_round)?;
            // Retrieve all of the certificates for the round just after the leader.
            let certificates = self.storage.get_certificates_for_round(certificate_round);
            // Construct a set over the authors, at the round just after the leader,
            // who included the leader's certificate in their previous certificate IDs.
            let authors = certificates
                .iter()
                .filter_map(|c| match c.previous_certificate_ids().contains(&leader_certificate.id()) {
                    true => Some(c.author()),
                    false => None,
                })
                .collect();

            debug!("Validating sync block {next_block_height} at round {commit_round}...");
            // Check if the leader is ready to be committed.
            if certificate_committee_lookback.is_availability_threshold_reached(&authors) {
                // Initialize the current certificate.
                let mut current_certificate = leader_certificate;
                // Check if there are any linked blocks that need to be added.
                let mut blocks_to_add = vec![next_block];

                // Check if there are other blocks to process based on `is_linked`.
                for height in (self.ledger.latest_block_height().saturating_add(1)..next_block_height).rev() {
                    // Retrieve the previous block.
                    let Some(previous_block) = latest_block_responses.get(&height) else {
                        bail!("Block {height} is missing from the latest block responses.");
                    };
                    // Retrieve the previous block's leader certificate.
                    let previous_certificate = match previous_block.authority() {
                        Authority::Quorum(subdag) => subdag.leader_certificate().clone(),
                        _ => bail!("Received a block with an unexpected authority type."),
                    };
                    // Determine if there is a path between the previous certificate and the current certificate.
                    if self.is_linked(previous_certificate.clone(), current_certificate.clone())? {
                        debug!("Previous sync block {height} is linked to the current block {next_block_height}");
                        // Add the previous leader certificate to the list of certificates to commit.
                        blocks_to_add.insert(0, previous_block.clone());
                        // Update the current certificate to the previous leader certificate.
                        current_certificate = previous_certificate;
                    }
                }

                // Add the blocks to the ledger.
                for block in blocks_to_add {
                    // Check that the blocks are sequential and can be added to the ledger.
                    let block_height = block.height();
                    if block_height != self.ledger.latest_block_height().saturating_add(1) {
                        warn!("Skipping block {block_height} from the latest block responses - not sequential.");
                        continue;
                    }
                    #[cfg(feature = "telemetry")]
                    let block_authority = block.authority().clone();

                    let self_ = self.clone();
                    tokio::task::spawn_blocking(move || {
                        // Check the next block.
                        self_.ledger.check_next_block(&block)?;
                        // Attempt to advance to the next block.
                        self_.ledger.advance_to_next_block(&block)?;

                        // Sync the height with the block.
                        self_.storage.sync_height_with_block(block.height());
                        // Sync the round with the block.
                        self_.storage.sync_round_with_block(block.round());

                        Ok::<(), anyhow::Error>(())
                    })
                    .await??;
                    // Remove the block height from the latest block responses.
                    latest_block_responses.remove(&block_height);

                    // Update the validator telemetry.
                    #[cfg(feature = "telemetry")]
                    if let Authority::Quorum(subdag) = block_authority {
                        self_.gateway.validator_telemetry().insert_subdag(&subdag);
                    }
                }
            } else {
                debug!(
                    "Availability threshold was not reached for block {next_block_height} at round {commit_round}. Checking next block..."
                );
            }

            // Don't remove the response from BlockSync yet as we might fall back to non-BFT sync.
        }

        Ok(())
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
            let certificates = self.storage.get_certificates_for_round(round);
            // Filter the certificates to only include those that are in the traversal.
            traversal = certificates
                .into_iter()
                .filter(|p| traversal.iter().any(|c| c.previous_certificate_ids().contains(&p.id())))
                .collect();
        }
        Ok(traversal.contains(&previous_certificate))
    }
}

// Methods to assist with the block sync module.
impl<N: Network> Sync<N> {
    /// Returns `true` if the node is synced and has connected peers.
    pub fn is_synced(&self) -> bool {
        // Ensure the validator is connected to other validators,
        // not just clients.
        if self.gateway.number_of_connected_peers() == 0 {
            return false;
        }

        self.block_sync.is_block_synced()
    }

    /// Returns the number of blocks the node is behind the greatest peer height.
    pub fn num_blocks_behind(&self) -> Option<u32> {
        self.block_sync.num_blocks_behind()
    }

    /// Returns the current block locators of the node.
    pub fn get_block_locators(&self) -> Result<BlockLocators<N>> {
        self.block_sync.get_block_locators()
    }
}

// Methods to assist with fetching batch certificates from peers.
impl<N: Network> Sync<N> {
    /// Sends a certificate request to the specified peer.
    pub async fn send_certificate_request(
        &self,
        peer_ip: SocketAddr,
        certificate_id: Field<N>,
    ) -> Result<BatchCertificate<N>> {
        // Initialize a oneshot channel.
        let (callback_sender, callback_receiver) = oneshot::channel();
        // Determine how many sent requests are pending.
        let num_sent_requests = self.pending.num_sent_requests(certificate_id);
        // Determine if we've already sent a request to the peer.
        let contains_peer_with_sent_request = self.pending.contains_peer_with_sent_request(certificate_id, peer_ip);
        // Determine the maximum number of redundant requests.
        let num_redundant_requests = max_redundant_requests(self.ledger.clone(), self.storage.current_round())?;
        // Determine if we should send a certificate request to the peer.
        // We send at most `num_redundant_requests` requests and each peer can only receive one request at a time.
        let should_send_request = num_sent_requests < num_redundant_requests && !contains_peer_with_sent_request;

        // Insert the certificate ID into the pending queue.
        self.pending.insert(certificate_id, peer_ip, Some((callback_sender, should_send_request)));

        // If the number of requests is less than or equal to the redundancy factor, send the certificate request to the peer.
        if should_send_request {
            // Send the certificate request to the peer.
            if self.gateway.send(peer_ip, Event::CertificateRequest(certificate_id.into())).await.is_none() {
                bail!("Unable to fetch batch certificate {certificate_id} - failed to send request")
            }
        } else {
            debug!(
                "Skipped sending request for certificate {} to '{peer_ip}' ({num_sent_requests} redundant requests)",
                fmt_id(certificate_id)
            );
        }
        // Wait for the certificate to be fetched.
        // TODO (raychu86): Consider making the timeout dynamic based on network traffic and/or the number of validators.
        match tokio::time::timeout(Duration::from_millis(MAX_FETCH_TIMEOUT_IN_MS), callback_receiver).await {
            // If the certificate was fetched, return it.
            Ok(result) => Ok(result?),
            // If the certificate was not fetched, return an error.
            Err(e) => bail!("Unable to fetch certificate {} - (timeout) {e}", fmt_id(certificate_id)),
        }
    }

    /// Handles the incoming certificate request.
    fn send_certificate_response(&self, peer_ip: SocketAddr, request: CertificateRequest<N>) {
        // Attempt to retrieve the certificate.
        if let Some(certificate) = self.storage.get_certificate(request.certificate_id) {
            // Send the certificate response to the peer.
            let self_ = self.clone();
            tokio::spawn(async move {
                let _ = self_.gateway.send(peer_ip, Event::CertificateResponse(certificate.into())).await;
            });
        }
    }

    /// Handles the incoming certificate response.
    /// This method ensures the certificate response is well-formed and matches the certificate ID.
    fn finish_certificate_request(&self, peer_ip: SocketAddr, response: CertificateResponse<N>) {
        let certificate = response.certificate;
        // Check if the peer IP exists in the pending queue for the given certificate ID.
        let exists = self.pending.get_peers(certificate.id()).unwrap_or_default().contains(&peer_ip);
        // If the peer IP exists, finish the pending request.
        if exists {
            // TODO: Validate the certificate.
            // Remove the certificate ID from the pending queue.
            self.pending.remove(certificate.id(), Some(certificate));
        }
    }
}

impl<N: Network> Sync<N> {
    /// Spawns a task with the given future; it should only be used for long-running tasks.
    fn spawn<T: Future<Output = ()> + Send + 'static>(&self, future: T) {
        self.handles.lock().push(tokio::spawn(future));
    }

    /// Shuts down the primary.
    pub async fn shut_down(&self) {
        info!("Shutting down the sync module...");
        // Acquire the response lock.
        let _lock = self.response_lock.lock().await;
        // Acquire the sync lock.
        let _lock = self.sync_lock.lock().await;
        // Abort the tasks.
        self.handles.lock().iter().for_each(|handle| handle.abort());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{helpers::now, ledger_service::CoreLedgerService, storage_service::BFTMemoryService};

    use snarkos_account::Account;
    use snarkos_node_sync::BlockSync;
    use snarkvm::{
        console::{
            account::{Address, PrivateKey},
            network::MainnetV0,
        },
        ledger::{
            narwhal::{BatchCertificate, BatchHeader, Subdag},
            store::{ConsensusStore, helpers::memory::ConsensusMemory},
        },
        prelude::{Ledger, VM},
        utilities::TestRng,
    };

    use aleo_std::StorageMode;
    use indexmap::IndexSet;
    use rand::Rng;
    use std::collections::BTreeMap;

    type CurrentNetwork = MainnetV0;
    type CurrentLedger = Ledger<CurrentNetwork, ConsensusMemory<CurrentNetwork>>;
    type CurrentConsensusStore = ConsensusStore<CurrentNetwork, ConsensusMemory<CurrentNetwork>>;

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_commit_via_is_linked() -> anyhow::Result<()> {
        let rng = &mut TestRng::default();
        // Initialize the round parameters.
        let max_gc_rounds = BatchHeader::<CurrentNetwork>::MAX_GC_ROUNDS as u64;
        let commit_round = 2;

        // Initialize the store.
        let store = CurrentConsensusStore::open(StorageMode::new_test(None)).unwrap();
        let account: Account<CurrentNetwork> = Account::new(rng)?;

        // Create a genesis block with a seeded RNG to reproduce the same genesis private keys.
        let seed: u64 = rng.r#gen();
        let genesis_rng = &mut TestRng::from_seed(seed);
        let genesis = VM::from(store).unwrap().genesis_beacon(account.private_key(), genesis_rng).unwrap();

        // Extract the private keys from the genesis committee by using the same RNG to sample private keys.
        let genesis_rng = &mut TestRng::from_seed(seed);
        let private_keys = [
            *account.private_key(),
            PrivateKey::new(genesis_rng)?,
            PrivateKey::new(genesis_rng)?,
            PrivateKey::new(genesis_rng)?,
        ];

        // Initialize the ledger with the genesis block.
        let ledger = CurrentLedger::load(genesis.clone(), StorageMode::new_test(None)).unwrap();
        // Initialize the ledger.
        let core_ledger = Arc::new(CoreLedgerService::new(ledger.clone(), Default::default()));

        // Sample 5 rounds of batch certificates starting at the genesis round from a static set of 4 authors.
        let (round_to_certificates_map, committee) = {
            let addresses = vec![
                Address::try_from(private_keys[0])?,
                Address::try_from(private_keys[1])?,
                Address::try_from(private_keys[2])?,
                Address::try_from(private_keys[3])?,
            ];

            let committee = ledger.latest_committee().unwrap();

            // Initialize a mapping from the round number to the set of batch certificates in the round.
            let mut round_to_certificates_map: HashMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> =
                HashMap::new();
            let mut previous_certificates: IndexSet<BatchCertificate<CurrentNetwork>> = IndexSet::with_capacity(4);

            for round in 0..=commit_round + 8 {
                let mut current_certificates = IndexSet::new();
                let previous_certificate_ids: IndexSet<_> = if round == 0 || round == 1 {
                    IndexSet::new()
                } else {
                    previous_certificates.iter().map(|c| c.id()).collect()
                };
                let committee_id = committee.id();

                // Create a certificate for the leader.
                if round <= 5 {
                    let leader = committee.get_leader(round).unwrap();
                    let leader_index = addresses.iter().position(|&address| address == leader).unwrap();
                    let non_leader_index = addresses.iter().position(|&address| address != leader).unwrap();
                    for i in [leader_index, non_leader_index].into_iter() {
                        let batch_header = BatchHeader::new(
                            &private_keys[i],
                            round,
                            now(),
                            committee_id,
                            Default::default(),
                            previous_certificate_ids.clone(),
                            rng,
                        )
                        .unwrap();
                        // Sign the batch header.
                        let mut signatures = IndexSet::with_capacity(4);
                        for (j, private_key_2) in private_keys.iter().enumerate() {
                            if i != j {
                                signatures.insert(private_key_2.sign(&[batch_header.batch_id()], rng).unwrap());
                            }
                        }
                        current_certificates.insert(BatchCertificate::from(batch_header, signatures).unwrap());
                    }
                }

                // Create a certificate for each validator.
                if round > 5 {
                    for (i, private_key_1) in private_keys.iter().enumerate() {
                        let batch_header = BatchHeader::new(
                            private_key_1,
                            round,
                            now(),
                            committee_id,
                            Default::default(),
                            previous_certificate_ids.clone(),
                            rng,
                        )
                        .unwrap();
                        // Sign the batch header.
                        let mut signatures = IndexSet::with_capacity(4);
                        for (j, private_key_2) in private_keys.iter().enumerate() {
                            if i != j {
                                signatures.insert(private_key_2.sign(&[batch_header.batch_id()], rng).unwrap());
                            }
                        }
                        current_certificates.insert(BatchCertificate::from(batch_header, signatures).unwrap());
                    }
                }
                // Update the map of certificates.
                round_to_certificates_map.insert(round, current_certificates.clone());
                previous_certificates = current_certificates.clone();
            }
            (round_to_certificates_map, committee)
        };

        // Initialize the storage.
        let storage = Storage::new(core_ledger.clone(), Arc::new(BFTMemoryService::new()), max_gc_rounds);
        // Insert certificates into storage.
        let mut certificates: Vec<BatchCertificate<CurrentNetwork>> = Vec::new();
        for i in 1..=commit_round + 8 {
            let c = (*round_to_certificates_map.get(&i).unwrap()).clone();
            certificates.extend(c);
        }
        for certificate in certificates.clone().iter() {
            storage.testing_only_insert_certificate_testing_only(certificate.clone());
        }

        // Create block 1.
        let leader_round_1 = commit_round;
        let leader_1 = committee.get_leader(leader_round_1).unwrap();
        let leader_certificate = storage.get_certificate_for_round_with_author(commit_round, leader_1).unwrap();
        let block_1 = {
            let mut subdag_map: BTreeMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> = BTreeMap::new();
            let mut leader_cert_map = IndexSet::new();
            leader_cert_map.insert(leader_certificate.clone());
            let mut previous_cert_map = IndexSet::new();
            for cert in storage.get_certificates_for_round(commit_round - 1) {
                previous_cert_map.insert(cert);
            }
            subdag_map.insert(commit_round, leader_cert_map.clone());
            subdag_map.insert(commit_round - 1, previous_cert_map.clone());
            let subdag = Subdag::from(subdag_map.clone())?;
            core_ledger.prepare_advance_to_next_quorum_block(subdag, Default::default())?
        };
        // Insert block 1.
        core_ledger.advance_to_next_block(&block_1)?;

        // Create block 2.
        let leader_round_2 = commit_round + 2;
        let leader_2 = committee.get_leader(leader_round_2).unwrap();
        let leader_certificate_2 = storage.get_certificate_for_round_with_author(leader_round_2, leader_2).unwrap();
        let block_2 = {
            let mut subdag_map_2: BTreeMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> = BTreeMap::new();
            let mut leader_cert_map_2 = IndexSet::new();
            leader_cert_map_2.insert(leader_certificate_2.clone());
            let mut previous_cert_map_2 = IndexSet::new();
            for cert in storage.get_certificates_for_round(leader_round_2 - 1) {
                previous_cert_map_2.insert(cert);
            }
            let mut prev_commit_cert_map_2 = IndexSet::new();
            for cert in storage.get_certificates_for_round(leader_round_2 - 2) {
                if cert != leader_certificate {
                    prev_commit_cert_map_2.insert(cert);
                }
            }
            subdag_map_2.insert(leader_round_2, leader_cert_map_2.clone());
            subdag_map_2.insert(leader_round_2 - 1, previous_cert_map_2.clone());
            subdag_map_2.insert(leader_round_2 - 2, prev_commit_cert_map_2.clone());
            let subdag_2 = Subdag::from(subdag_map_2.clone())?;
            core_ledger.prepare_advance_to_next_quorum_block(subdag_2, Default::default())?
        };
        // Insert block 2.
        core_ledger.advance_to_next_block(&block_2)?;

        // Create block 3
        let leader_round_3 = commit_round + 4;
        let leader_3 = committee.get_leader(leader_round_3).unwrap();
        let leader_certificate_3 = storage.get_certificate_for_round_with_author(leader_round_3, leader_3).unwrap();
        let block_3 = {
            let mut subdag_map_3: BTreeMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> = BTreeMap::new();
            let mut leader_cert_map_3 = IndexSet::new();
            leader_cert_map_3.insert(leader_certificate_3.clone());
            let mut previous_cert_map_3 = IndexSet::new();
            for cert in storage.get_certificates_for_round(leader_round_3 - 1) {
                previous_cert_map_3.insert(cert);
            }
            let mut prev_commit_cert_map_3 = IndexSet::new();
            for cert in storage.get_certificates_for_round(leader_round_3 - 2) {
                if cert != leader_certificate_2 {
                    prev_commit_cert_map_3.insert(cert);
                }
            }
            subdag_map_3.insert(leader_round_3, leader_cert_map_3.clone());
            subdag_map_3.insert(leader_round_3 - 1, previous_cert_map_3.clone());
            subdag_map_3.insert(leader_round_3 - 2, prev_commit_cert_map_3.clone());
            let subdag_3 = Subdag::from(subdag_map_3.clone())?;
            core_ledger.prepare_advance_to_next_quorum_block(subdag_3, Default::default())?
        };
        // Insert block 3.
        core_ledger.advance_to_next_block(&block_3)?;

        // Initialize the syncing ledger.
        let syncing_ledger = Arc::new(CoreLedgerService::new(
            CurrentLedger::load(genesis, StorageMode::new_test(None)).unwrap(),
            Default::default(),
        ));
        // Initialize the gateway.
        let gateway = Gateway::new(account.clone(), storage.clone(), syncing_ledger.clone(), None, &[], None)?;
        // Initialize the block synchronization logic.
        let block_sync = Arc::new(BlockSync::new(syncing_ledger.clone()));
        // Initialize the sync module.
        let sync = Sync::new(gateway.clone(), storage.clone(), syncing_ledger.clone(), block_sync);
        // Try to sync block 1.
        sync.sync_storage_with_block(block_1).await?;
        assert_eq!(syncing_ledger.latest_block_height(), 1);
        // Try to sync block 2.
        sync.sync_storage_with_block(block_2).await?;
        assert_eq!(syncing_ledger.latest_block_height(), 2);
        // Try to sync block 3.
        sync.sync_storage_with_block(block_3).await?;
        assert_eq!(syncing_ledger.latest_block_height(), 3);
        // Ensure blocks 1 and 2 were added to the ledger.
        assert!(syncing_ledger.contains_block_height(1));
        assert!(syncing_ledger.contains_block_height(2));

        Ok(())
    }

    #[tokio::test]
    #[tracing_test::traced_test]
    async fn test_pending_certificates() -> anyhow::Result<()> {
        let rng = &mut TestRng::default();
        // Initialize the round parameters.
        let max_gc_rounds = BatchHeader::<CurrentNetwork>::MAX_GC_ROUNDS as u64;
        let commit_round = 2;

        // Initialize the store.
        let store = CurrentConsensusStore::open(StorageMode::new_test(None)).unwrap();
        let account: Account<CurrentNetwork> = Account::new(rng)?;

        // Create a genesis block with a seeded RNG to reproduce the same genesis private keys.
        let seed: u64 = rng.r#gen();
        let genesis_rng = &mut TestRng::from_seed(seed);
        let genesis = VM::from(store).unwrap().genesis_beacon(account.private_key(), genesis_rng).unwrap();

        // Extract the private keys from the genesis committee by using the same RNG to sample private keys.
        let genesis_rng = &mut TestRng::from_seed(seed);
        let private_keys = [
            *account.private_key(),
            PrivateKey::new(genesis_rng)?,
            PrivateKey::new(genesis_rng)?,
            PrivateKey::new(genesis_rng)?,
        ];
        // Initialize the ledger with the genesis block.
        let ledger = CurrentLedger::load(genesis.clone(), StorageMode::new_test(None)).unwrap();
        // Initialize the ledger.
        let core_ledger = Arc::new(CoreLedgerService::new(ledger.clone(), Default::default()));
        // Sample rounds of batch certificates starting at the genesis round from a static set of 4 authors.
        let (round_to_certificates_map, committee) = {
            // Initialize the committee.
            let committee = ledger.latest_committee().unwrap();
            // Initialize a mapping from the round number to the set of batch certificates in the round.
            let mut round_to_certificates_map: HashMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> =
                HashMap::new();
            let mut previous_certificates: IndexSet<BatchCertificate<CurrentNetwork>> = IndexSet::with_capacity(4);

            for round in 0..=commit_round + 8 {
                let mut current_certificates = IndexSet::new();
                let previous_certificate_ids: IndexSet<_> = if round == 0 || round == 1 {
                    IndexSet::new()
                } else {
                    previous_certificates.iter().map(|c| c.id()).collect()
                };
                let committee_id = committee.id();
                // Create a certificate for each validator.
                for (i, private_key_1) in private_keys.iter().enumerate() {
                    let batch_header = BatchHeader::new(
                        private_key_1,
                        round,
                        now(),
                        committee_id,
                        Default::default(),
                        previous_certificate_ids.clone(),
                        rng,
                    )
                    .unwrap();
                    // Sign the batch header.
                    let mut signatures = IndexSet::with_capacity(4);
                    for (j, private_key_2) in private_keys.iter().enumerate() {
                        if i != j {
                            signatures.insert(private_key_2.sign(&[batch_header.batch_id()], rng).unwrap());
                        }
                    }
                    current_certificates.insert(BatchCertificate::from(batch_header, signatures).unwrap());
                }

                // Update the map of certificates.
                round_to_certificates_map.insert(round, current_certificates.clone());
                previous_certificates = current_certificates.clone();
            }
            (round_to_certificates_map, committee)
        };

        // Initialize the storage.
        let storage = Storage::new(core_ledger.clone(), Arc::new(BFTMemoryService::new()), max_gc_rounds);
        // Insert certificates into storage.
        let mut certificates: Vec<BatchCertificate<CurrentNetwork>> = Vec::new();
        for i in 1..=commit_round + 8 {
            let c = (*round_to_certificates_map.get(&i).unwrap()).clone();
            certificates.extend(c);
        }
        for certificate in certificates.clone().iter() {
            storage.testing_only_insert_certificate_testing_only(certificate.clone());
        }
        // Create block 1.
        let leader_round_1 = commit_round;
        let leader_1 = committee.get_leader(leader_round_1).unwrap();
        let leader_certificate = storage.get_certificate_for_round_with_author(commit_round, leader_1).unwrap();
        let mut subdag_map: BTreeMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> = BTreeMap::new();
        let block_1 = {
            let mut leader_cert_map = IndexSet::new();
            leader_cert_map.insert(leader_certificate.clone());
            let mut previous_cert_map = IndexSet::new();
            for cert in storage.get_certificates_for_round(commit_round - 1) {
                previous_cert_map.insert(cert);
            }
            subdag_map.insert(commit_round, leader_cert_map.clone());
            subdag_map.insert(commit_round - 1, previous_cert_map.clone());
            let subdag = Subdag::from(subdag_map.clone())?;
            core_ledger.prepare_advance_to_next_quorum_block(subdag, Default::default())?
        };
        // Insert block 1.
        core_ledger.advance_to_next_block(&block_1)?;

        // Create block 2.
        let leader_round_2 = commit_round + 2;
        let leader_2 = committee.get_leader(leader_round_2).unwrap();
        let leader_certificate_2 = storage.get_certificate_for_round_with_author(leader_round_2, leader_2).unwrap();
        let mut subdag_map_2: BTreeMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> = BTreeMap::new();
        let block_2 = {
            let mut leader_cert_map_2 = IndexSet::new();
            leader_cert_map_2.insert(leader_certificate_2.clone());
            let mut previous_cert_map_2 = IndexSet::new();
            for cert in storage.get_certificates_for_round(leader_round_2 - 1) {
                previous_cert_map_2.insert(cert);
            }
            subdag_map_2.insert(leader_round_2, leader_cert_map_2.clone());
            subdag_map_2.insert(leader_round_2 - 1, previous_cert_map_2.clone());
            let subdag_2 = Subdag::from(subdag_map_2.clone())?;
            core_ledger.prepare_advance_to_next_quorum_block(subdag_2, Default::default())?
        };
        // Insert block 2.
        core_ledger.advance_to_next_block(&block_2)?;

        // Create block 3
        let leader_round_3 = commit_round + 4;
        let leader_3 = committee.get_leader(leader_round_3).unwrap();
        let leader_certificate_3 = storage.get_certificate_for_round_with_author(leader_round_3, leader_3).unwrap();
        let mut subdag_map_3: BTreeMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> = BTreeMap::new();
        let block_3 = {
            let mut leader_cert_map_3 = IndexSet::new();
            leader_cert_map_3.insert(leader_certificate_3.clone());
            let mut previous_cert_map_3 = IndexSet::new();
            for cert in storage.get_certificates_for_round(leader_round_3 - 1) {
                previous_cert_map_3.insert(cert);
            }
            subdag_map_3.insert(leader_round_3, leader_cert_map_3.clone());
            subdag_map_3.insert(leader_round_3 - 1, previous_cert_map_3.clone());
            let subdag_3 = Subdag::from(subdag_map_3.clone())?;
            core_ledger.prepare_advance_to_next_quorum_block(subdag_3, Default::default())?
        };
        // Insert block 3.
        core_ledger.advance_to_next_block(&block_3)?;

        /*
            Check that the pending certificates are computed correctly.
        */

        // Retrieve the pending certificates.
        let pending_certificates = storage.get_pending_certificates();
        // Check that all of the pending certificates are not contained in the ledger.
        for certificate in pending_certificates.clone() {
            assert!(!core_ledger.contains_certificate(&certificate.id()).unwrap_or(false));
        }
        // Initialize an empty set to be populated with the committed certificates in the block subdags.
        let mut committed_certificates: IndexSet<BatchCertificate<CurrentNetwork>> = IndexSet::new();
        {
            let subdag_maps = [&subdag_map, &subdag_map_2, &subdag_map_3];
            for subdag in subdag_maps.iter() {
                for subdag_certificates in subdag.values() {
                    committed_certificates.extend(subdag_certificates.iter().cloned());
                }
            }
        };
        // Create the set of candidate pending certificates as the set of all certificates minus the set of the committed certificates.
        let mut candidate_pending_certificates: IndexSet<BatchCertificate<CurrentNetwork>> = IndexSet::new();
        for certificate in certificates.clone() {
            if !committed_certificates.contains(&certificate) {
                candidate_pending_certificates.insert(certificate);
            }
        }
        // Check that the set of pending certificates is equal to the set of candidate pending certificates.
        assert_eq!(pending_certificates, candidate_pending_certificates);
        Ok(())
    }
}
