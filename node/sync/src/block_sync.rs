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
    helpers::{BlockId, BlockRequestId, BlockSyncRequest},
    locators::BlockLocators,
    pending_chain::PendingChain,
};
use snarkos_node_bft_ledger_service::LedgerService;
use snarkos_node_router::messages::DataBlocks;
use snarkos_node_sync_communication_service::CommunicationService;
use snarkos_node_sync_locators::{CHECKPOINT_INTERVAL, NUM_RECENT_BLOCKS};

use snarkvm::{
    console::network::Network,
    ledger::{
        PendingBlock,
        authority::Authority,
        block::Block,
        narwhal::{BatchHeader, Subdag},
    },
};

use anyhow::{Result, bail};
use indexmap::IndexMap;
use itertools::Itertools;
#[cfg(feature = "locktick")]
use locktick::parking_lot::{Mutex, RwLock};
#[cfg(not(feature = "locktick"))]
use parking_lot::{Mutex, RwLock};
use rand::seq::SliceRandom;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};

/// The time nodes wait between issuing batches of block requests to avoid triggering spam detection.
// TODO (kaimast): Document why 10ms (not 1 or 100)
pub const BLOCK_REQUEST_BATCH_DELAY: Duration = Duration::from_millis(10);

/// The maximum number of peers we attempt to sync from.
const NUM_SYNC_CANDIDATE_PEERS: usize = 5;

const BLOCK_REQUEST_TIMEOUT_IN_SECS: u64 = 30; // 30 seconds
const MAX_BLOCK_REQUESTS: usize = 50; // 50 requests

/// The maximum number of blocks tolerated before the primary is considered behind its peers.
/// This is set to two because the most recent block will not be confirmed until the next one.
pub const MAX_BLOCKS_BEHIND: u32 = 2; // blocks

/// This is a dummy IP address that is used to represent the local node.
/// Note: This here does not need to be a real IP address, but it must be unique/distinct from all other connections.
pub const DUMMY_SELF_IP: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

/// Struct that tracks oustanding requests.
///
/// This is wrapped in a single lock in BlockSync, so that it
/// can be updated atomically.
#[derive(Clone)]
struct BlockSyncState<N: Network> {
    requests: BTreeMap<BlockRequestId, BlockSyncRequest<N>>,

    /// Allows tracking if specific block has already been requested.
    block_to_request: HashMap<N::BlockHash, BlockRequestId>,

    /// Removing an entry from this map must remove the corresponding entry from the requests map.
    responses: HashMap<BlockRequestId, Vec<Block<N>>>,
}

impl<N: Network> Default for BlockSyncState<N> {
    fn default() -> Self {
        Self { requests: Default::default(), block_to_request: Default::default(), responses: Default::default() }
    }
}

/// A struct that keeps track of synchronizing blocks with other nodes.
///
/// It generates requests to send to other peers and processes responses to those requests.
/// The struct also keeps track of block locators, which indicate which peers it can fetch blocks from.
///
/// # Notes
/// - The actual network communication happens in `snarkos_node::Client` (for clients and provers) and in `snarkos_node_bft::Sync` (for validators).
///
/// - Validators only sync from other nodes using this struct if they fall behind, e.g.,
///   because they experience a network partition.
///   In the common case, validators will generate blocks from the DAG after an anchor certificate has been approved
///   by a supermajority of the committee.
///
/// # State
/// - When a request is inserted, the `requests` map and `request_timestamps` map insert an entry for the request height.
/// - When a response is inserted, the `responses` map inserts the entry for the request height.
/// - When a request is completed, the `requests` map still has the entry, but its `sync_ips` is empty;
///   the `request_timestamps` map remains unchanged.
/// - When a response is removed/completed, the `requests` map and `request_timestamps` map also remove the entry for the request height.
/// - When a request is timed out, the `requests`, `request_timestamps`, and `responses` map remove the entry for the request height.
pub struct BlockSync<N: Network> {
    /// The ledger.
    ledger: Arc<dyn LedgerService<N>>,
    /// The map of peer IP to their block locators.
    /// The block locators are consistent with the ledger and every other peer's block locators.
    locators: RwLock<HashMap<SocketAddr, BlockLocators<N>>>,
    /// Tracks the pending block requests and their responses.
    state: RwLock<BlockSyncState<N>>,
    /// The boolean indicator of whether the node is synced up to the latest block (within the given tolerance).
    is_block_synced: AtomicBool,
    /// The number of blocks the peer is behind the greatest peer height.
    num_blocks_behind: AtomicU32,
    /// The lock to guarantee advance_with_sync_blocks() is called only once at a time.
    advance_with_sync_blocks_lock: Mutex<()>,
    /// The set of pending chains (blocks that have not received sufficient votes yet).
    pending_chains: RwLock<Vec<PendingChain<N>>>,
    /// The last time a new block was synced.
    last_update: Mutex<Instant>,
}

impl<N: Network> BlockSync<N> {
    /// Initializes a new block sync module.
    pub fn new(ledger: Arc<dyn LedgerService<N>>) -> Self {
        Self {
            ledger,
            last_update: Mutex::new(Instant::now()),
            locators: Default::default(),
            pending_chains: Default::default(),
            state: Default::default(),
            is_block_synced: Default::default(),
            num_blocks_behind: Default::default(),
            advance_with_sync_blocks_lock: Default::default(),
        }
    }

    /// Returns `true` if the node is synced up to the latest block (within the given tolerance).
    #[inline]
    pub fn is_block_synced(&self) -> bool {
        self.is_block_synced.load(Ordering::SeqCst)
    }

    /// Returns the number of blocks the node is behind the greatest peer height.
    #[inline]
    pub fn num_blocks_behind(&self) -> u32 {
        self.num_blocks_behind.load(Ordering::SeqCst)
    }
}

// Helper functions needed for testing
#[cfg(test)]
impl<N: Network> BlockSync<N> {
    /// Returns the latest block height of the given peer IP.
    fn get_peer_height(&self, peer_ip: &SocketAddr) -> Option<u32> {
        self.locators.read().get(peer_ip).map(|locators| locators.latest_locator_height())
    }

    /// Returns the block request for the given height, if it exists.
    fn get_block_request(&self, request_id: &BlockRequestId) -> Option<BlockSyncRequest<N>> {
        self.state.read().requests.get(request_id).cloned()
    }
}

impl<N: Network> BlockSync<N> {
    /// Returns the block locators.
    #[inline]
    pub fn get_block_locators(&self) -> Result<BlockLocators<N>> {
        // Retrieve the latest block height.
        let latest_height = self.ledger.latest_block_height();

        // Initialize the recents map.
        // TODO: generalize this for RECENT_INTERVAL > 1, or remove this comment if we hardwire that to 1
        let mut recents = IndexMap::with_capacity(NUM_RECENT_BLOCKS);
        // Retrieve the recent block hashes.
        for height in latest_height.saturating_sub((NUM_RECENT_BLOCKS - 1) as u32)..=latest_height {
            recents.insert(height, self.ledger.get_block_hash(height)?);
        }

        // Initialize the checkpoints map.
        let mut checkpoints = IndexMap::with_capacity((latest_height / CHECKPOINT_INTERVAL + 1).try_into()?);
        // Retrieve the checkpoint block hashes.
        for height in (0..=latest_height).step_by(CHECKPOINT_INTERVAL as usize) {
            checkpoints.insert(height, self.ledger.get_block_hash(height)?);
        }

        // Construct the block locators.
        BlockLocators::new(recents, checkpoints)
    }

    /// Returns true if there are pending responses to block requests that need to be processed.
    pub fn has_pending_responses(&self) -> bool {
        !self.state.read().responses.is_empty()
    }

    /// Send a batch of block requests.
    #[must_use]
    pub async fn send_block_request<C: CommunicationService>(
        &self,
        communication: &C,
        request: BlockSyncRequest<N>,
    ) -> bool {
        let request_id = (request.sync_peer, request.start_height());

        // Construct the message.
        let message = C::prepare_block_request(request.start_height(), request.end_height());

        let sync_peer = request.sync_peer;
        if let Err(err) = self.insert_block_request(request) {
            error!("{err}");
            return false;
        }

        // Send the message to the peers.
        let sender = communication.send(sync_peer, message).await;

        // If sending fails, remove the block request from the sync pool.
        if sender.is_none() {
            warn!("Failed to send block request to peer '{}'", sync_peer);
            // Remove the entire block request from the sync pool.
            self.remove_block_request(&request_id);
            return false;
        }

        true
    }

    /// Inserts a new block response from the given peer IP.
    ///
    /// Returns an error if the block was malformed, or we already received a different block for this height.
    /// Note, that this only queues the response. After this, you most likely want to call `Self::try_advancing_block_synchronization`.
    #[inline]
    pub fn insert_block_responses(&self, peer_ip: SocketAddr, blocks: Vec<Block<N>>) -> Result<()> {
        // Insert the candidate blocks into the sync pool.
        if let Err(error) = self.insert_block_response(peer_ip, blocks) {
            bail!("{error}");
        }
        Ok(())
    }

    /// Returns the next block for the given `next_height` if the request is complete,
    /// or `None` otherwise. This does not remove the block from the `responses` map.
    #[inline]
    fn peek_next_blocks(&self) -> Vec<(BlockRequestId, Vec<Block<N>>)> {
        // Note: This lock must be held across the entire scope, due to asynchronous block responses
        let state = self.state.read();

        state
            .responses
            .iter()
            .filter_map(|(request_id, blocks)| {
                let Some(first_block) = blocks.first() else {
                    // This should never happen.
                    warn!("Response is empty");
                    return None;
                };

                if first_block.height() <= self.ledger.latest_block_height() {
                    return Some((*request_id, blocks.clone()));
                }

                if self.ledger.latest_block().hash() == first_block.previous_hash() {
                    return Some((*request_id, blocks.clone()));
                }

                for pending_chain in self.pending_chains.read().iter() {
                    if pending_chain.contains(&first_block.previous_hash()) {
                        return Some((*request_id, blocks.clone()));
                    }
                }

                None
            })
            .collect()
    }

    fn check_votes(&self, dag: &Subdag<N>, height: u32, previous_dag: &Subdag<N>) -> Result<bool> {
        let leader_certificate = previous_dag.leader_certificate();

        let commit_round = leader_certificate.round();
        let certificate_round = commit_round + 1;

        let authors: HashSet<_> = dag
            .certificates()
            .filter_map(|cert| {
                if cert.round() == certificate_round
                    && cert.previous_certificate_ids().contains(&leader_certificate.id())
                {
                    Some(cert.author())
                } else {
                    None
                }
            })
            .collect();

        let certificate_committee_lookback = self.ledger.get_committee_lookback_for_round(certificate_round)?;

        debug!("Validating sync block {height} at round {commit_round}...");
        // Check if the leader is ready to be committed.

        Ok(certificate_committee_lookback.is_availability_threshold_reached(&authors))
    }

    /// Attempts to advance synchronization by processing completed block responses.
    ///
    /// Validators will not call this function, but instead execute `snarkos_node_bft::Sync::try_advancing_block_synchronization`
    /// which also updates the BFT state.
    ///
    /// Returns the list of blocks that were newly applied to the ledger.
    #[inline]
    pub fn try_advancing_block_synchronization(&self) -> Vec<Block<N>> {
        // Acquire the lock to ensure this function is called only once at a time.
        // If the lock is already acquired, return early.
        let Some(_lock) = self.advance_with_sync_blocks_lock.try_lock() else {
            trace!("Skipping attempt to advance block synchronziation as it is already in progress");
            return vec![];
        };

        let mut ledger_height = self.ledger.latest_block_height();
        let mut pending_chains = self.pending_chains.write();

        // New blocks that have been confirmed.
        let mut advanced_by = vec![];

        for (request_id, blocks) in self.peek_next_blocks() {
            for block in blocks {
                let block_height = block.height();

                // First, find the pending chain to trueappend to.
                let chain_idx = if block_height == ledger_height + 1 {
                    let pending_chain = PendingChain::new();
                    pending_chains.push(pending_chain);
                    pending_chains.len() - 1
                } else {
                    let mut pending_idx = None;

                    for idx in 0..pending_chains.len() {
                        let chain = &pending_chains[idx];
                        if chain.contains(&block.previous_hash()) {
                            let Some(chain_height) = chain.current_height() else {
                                warn!("There is an empty pending chain.");
                                continue;
                            };

                            // Either extend or fork the chain.
                            if chain_height == block.height() {
                                pending_idx = Some(idx);
                            } else {
                                let new_chain = chain.fork_at(block.height());
                                pending_chains.push(new_chain);
                                pending_idx = Some(pending_chains.len() - 1);
                            }
                        }
                    }

                    match pending_idx {
                        Some(idx) => idx,
                        None => {
                            error!(
                                "Cannot find a suitable prefix for block {} at height {}",
                                block.hash(),
                                block.height()
                            );
                            break;
                        }
                    }
                };

                let pending_chain = &mut pending_chains[chain_idx];

                let block = match self.ledger.check_block_subdag(block, pending_chain.blocks()) {
                    Ok(pending) => pending,
                    Err(err) => {
                        warn!("Discarding invalid block - {err}");
                        //TODO remove empty pending chain here, if needed.
                        continue;
                    }
                };

                // Check if we can confirm blocks
                let has_votes = match block.authority() {
                    Authority::Beacon(_) => true,
                    Authority::Quorum(dag) => match pending_chain.head() {
                        Some(previous_block) => {
                            let Authority::Quorum(previous_dag) = previous_block.authority() else {
                                error!("Invalid authority for previous block");
                                continue;
                            };

                            match self.check_votes(dag, previous_block.height(), previous_dag) {
                                Ok(b) => b,
                                Err(err) => {
                                    warn!("Unexpected problem - {err}");
                                    continue;
                                }
                            }
                        }
                        None => {
                            // Pending chain is empty. Nothing to confirm.
                            false
                        }
                    },
                };

                let mut advanced = false;
                if has_votes {
                    for previous_block in pending_chain.blocks() {
                        let previous_block = previous_block.clone();
                        if let Some(pblock) = self.try_to_confirm_block(previous_block) {
                            advanced_by.push(pblock);
                            ledger_height += 1;
                            advanced = true;
                        } else {
                            break;
                        }
                    }
                }

                let block_hash = block.hash();

                if !pending_chain.append(block) {
                    error!("Pending chain already contained block?");
                }

                debug!("Added new pending block {} at height {}", &block_hash, block_height);

                if pending_chain.len() > 10 {
                    warn!("Pending chain is very long: {} blocks", pending_chain.len());
                }

                // If we successfully confirmed blocks, get rid of unneeded blocks.
                if advanced {
                    // Truncate chains.
                    for pending_chain in pending_chains.iter_mut() {
                        pending_chain.truncate_prefix(ledger_height);
                    }

                    // Remove any obsolete chain.
                    pending_chains.retain(|c| !c.is_empty());
                }
            }

            self.remove_block_request(&request_id);
        }

        if advanced_by.is_empty() {
            let mut last_update = self.last_update.lock();
            let elapsed = Instant::now() - *last_update;

            // Print debug message if we are (possibly) stuck.
            if elapsed > Duration::from_secs(60) && !self.is_block_synced() {
                error!("Block synchornization has not made progress for over a minute");
                // Update so we don't immediately print again.
                *last_update = Instant::now();
            }
        } else {
            debug!("Advanced by {} blocks", advanced_by.len());
            *self.last_update.lock() = Instant::now();
        }

        advanced_by
    }

    /// Try to apply the next pending block to the ledger.
    fn try_to_confirm_block(&self, pending_block: PendingBlock<N>) -> Option<Block<N>> {
        let block = match self.ledger.check_block_content(pending_block) {
            Ok(block) => block,
            Err(err) => {
                warn!("Failed to verify block contents: {err}");
                return None;
            }
        };

        info!("Syncing the ledger to block at height {}", block.height());

        match self.ledger.advance_to_next_block(&block) {
            Ok(_) => Some(block),
            Err(err) => {
                warn!("Failed to advance to next block (height: {}, hash: '{}'): {err}", block.height(), block.hash());
                None
            }
        }
    }
}

// Functionality related to sync peers.
impl<N: Network> BlockSync<N> {
    /// Returns the sync peers with their latest heights, and their minimum common ancestor, if the node can sync.
    /// This function returns peers that are consistent with each other, and have a block height
    /// that is greater than the ledger height of this node.
    pub fn find_sync_peers(&self) -> IndexMap<SocketAddr, u32> {
        self.find_sync_peers_at_height(self.ledger.latest_block_height())
    }

    /// Same as `Self::find_sync_peers`, but allows specifiying a custom height
    /// (must be greater than the ledger height).
    pub fn find_sync_peers_at_height(&self, current_height: u32) -> IndexMap<SocketAddr, u32> {
        let sync_peers = self.find_sync_peers_inner(current_height);
        // Map the locators into the latest height.
        sync_peers.into_iter().map(|(ip, locators)| (ip, locators.latest_locator_height())).collect()
    }

    /// Updates the block locators and common ancestors for the given peer IP.
    ///
    /// This function does not need to check that the block locators are well-formed,
    /// because that is already done in [`BlockLocators::new()`], as noted in [`BlockLocators`].
    ///
    /// This function does **not** check
    /// that the block locators are consistent with the peer's previous block locators or other peers' block locators.
    pub fn update_peer_locators(&self, peer_ip: SocketAddr, locators: BlockLocators<N>) -> Result<()> {
        // Update the locators entry for the given peer IP.
        self.locators.write().insert(peer_ip, locators.clone());
        Ok(())
    }

    /// Removes the peer from the sync pool, if they exist.
    pub fn remove_peer(&self, peer_ip: &SocketAddr) {
        // Remove the locators entry for the given peer IP.
        self.locators.write().remove(peer_ip);
        // Remove all block requests to the peer.
        self.remove_block_requests_to_peer(peer_ip);
    }
}

// Functionality related to requests.
impl<N: Network> BlockSync<N> {
    /// Returns a list of block requests and the sync peers, if the node needs to sync.
    ///
    /// You usually want to call `remove_timed_out_block_requests` before invoking this function.
    pub fn prepare_block_requests(&self) -> Result<Vec<BlockSyncRequest<N>>> {
        // Prepare the block requests.
        let ledger_height = self.ledger.latest_block_height();

        let current_height = ledger_height; //FIXME
        let sync_peers = self.find_sync_peers_inner(current_height);
        let state = self.state.read();
        let pending_chains = self.pending_chains.read();

        if sync_peers.is_empty() {
            if state.requests.is_empty() && state.responses.is_empty() {
                // Update `is_block_synced` if there are no pending requests or responses.
                trace!("All requests have been processed. Will set block synced to true.");
                // Update the state of `is_block_synced` for the sync module.
                self.update_is_block_synced(0, current_height, MAX_BLOCKS_BEHIND);
            } else {
                trace!("No new blocks can be requests, but there are still outstanding requests.");
            }

            // Return an empty list of block requests.
            return Ok(vec![]);
        };

        // Retrieve the highest block height.
        let greatest_peer_height = sync_peers.iter().map(|(_, l)| l.latest_locator_height()).max().unwrap_or(0);
        // Update the state of `is_block_synced` for the sync module.
        self.update_is_block_synced(greatest_peer_height, current_height, MAX_BLOCKS_BEHIND);

        let mut result = vec![];

        // The set of all blocks we are about to request
        let mut queued_requests = HashSet::default();

        for (peer_ip, locators) in sync_peers.iter() {
            let max_height = locators.latest_locator_height();
            let Some(hash) = locators.get_hash(max_height) else {
                bail!("Missing block hash in peer \"{peer_ip}\"'s locator at height {max_height}");
            };

            // First check if we should even create a request
            // (need at least one block that is not know/requested yet).
            if state.block_to_request.contains_key(&hash) {
                continue;
            }

            if queued_requests.contains(&(max_height, hash)) {
                continue;
            }

            for pending_chain in pending_chains.iter() {
                if pending_chain.contains(&hash) {
                    continue;
                }
            }

            // Then, find starting point.
            let mut start_height = max_height;
            let mut start_hash = hash;

            while start_height > ledger_height && start_height > 1 {
                let parent_height = start_height - 1;

                let Some(parent_hash) = locators.get_hash(parent_height) else {
                    bail!("Missing block hash in peer \"{peer_ip}\"'s locator at height {parent_height}");
                };

                let block_id = BlockId { hash: start_hash, height: start_height, previous_hash: parent_hash };

                // Stop if we detect a known ancestor.
                if self.is_previous_block_known(&block_id, &queued_requests)? {
                    break;
                }

                start_height -= 1;
                start_hash = parent_hash;
            }

            // Create request in chunks and ensure we don't exceed the set maximum.
            while start_height < max_height && result.len() < MAX_BLOCK_REQUESTS {
                let max_request_height =
                    (start_height + DataBlocks::<N>::MAXIMUM_NUMBER_OF_BLOCKS as u32).min(max_height);
                let req =
                    self.construct_request(start_height, max_request_height, peer_ip, locators, &queued_requests)?;

                // Make sure we don't request the same block again.
                for block in req.blocks.iter() {
                    queued_requests.insert((block.height, block.hash));
                }

                trace!(
                    "Generated new block request to peer \"{}\" for blocks from height {} to {}",
                    req.sync_peer,
                    req.start_height(),
                    req.end_height()
                );

                start_height = req.end_height();
                result.push(req);
            }
        }

        Ok(result)
    }

    /// Updates the state of `is_block_synced` for the sync module.
    fn update_is_block_synced(&self, greatest_peer_height: u32, current_height: u32, max_blocks_behind: u32) {
        // Retrieve the latest block height.
        let ledger_height = self.ledger.latest_block_height();
        trace!(
            "Updating is_block_synced: greatest_peer_height = {greatest_peer_height}, ledger_height = {ledger_height},
            current_height = {current_height}"
        );
        // Compute the number of blocks that we are behind by.
        let num_blocks_behind = greatest_peer_height.saturating_sub(current_height);
        // Determine if the primary is synced.
        let is_synced = num_blocks_behind <= max_blocks_behind;
        // Update the num blocks behind.
        self.num_blocks_behind.store(num_blocks_behind, Ordering::SeqCst);
        // Update the sync status.
        self.is_block_synced.store(is_synced, Ordering::SeqCst);
        // Update the `IS_SYNCED` metric.
        #[cfg(feature = "metrics")]
        metrics::gauge(metrics::bft::IS_SYNCED, is_synced);
    }

    /// Inserts a block request for the given height.
    fn insert_block_request(&self, request: BlockSyncRequest<N>) -> Result<()> {
        let mut state = self.state.write();
        let req_id = request.get_identifier();

        // Check for conflicts (even though we alread did this earlier).
        if state.requests.contains_key(&req_id) {
            bail!("Pending request already existed");
        }

        for block_id in &request.blocks {
            if state.block_to_request.contains_key(&block_id.hash) {
                bail!("Request for block {block_id:?} already existed");
            }
        }

        // Now insert.
        for block_id in &request.blocks {
            state.block_to_request.insert(block_id.hash, req_id);
        }

        state.requests.insert(req_id, request);

        Ok(())
    }

    /// Inserts the given block response, after checking that the request exists and the response is well-formed.
    /// On success, this function removes the peer IP from the requests map.
    /// On failure, this function removes all block requests from the given peer IP.
    fn insert_block_response(&self, peer_ip: SocketAddr, blocks: Vec<Block<N>>) -> Result<()> {
        let Some(first_block) = blocks.first() else {
            bail!("Block response contained no blocks");
        };

        // Build the request identifier
        let start_height = first_block.height();
        let num_blocks = blocks.len() as u32;
        let req_id = (peer_ip, start_height);

        // Ensure the block (response) from the peer is well-formed. On failure, remove all block requests to the peer.
        if let Err(error) = self.check_block_response(&peer_ip, first_block) {
            // Remove all block requests to the peer.
            self.remove_block_requests_to_peer(&peer_ip);
            return Err(error);
        }

        let mut state = self.state.write();

        // Insert the candidate block into the responses map.
        let prev = state.responses.insert(req_id, blocks);

        if prev.is_some() {
            bail!("Already received the same response");

            /* TODO decide what to do here
            // Remove the candidate block.
            responses.remove(&height);
            // Drop the write lock on the responses map.
            drop(responses);
            // Remove all block requests to the peer.
            self.remove_block_requests_to_peer(&peer_ip);
            bail!("Candidate block {height} from '{peer_ip}' is malformed"); */
        }

        trace!(
            "Got block response from \"{peer_ip}\" for blocks from height {start_height} to {}",
            start_height + num_blocks
        );
        Ok(())
    }

    /// For a request, check that it builds on a block that we already know about
    /// or that we are currently requesting.
    /// This will also fail if the ledger already confirmed a previous block with a different hash.
    fn is_previous_block_known(
        &self,
        block_id: &BlockId<N>,
        queued_requests: &HashSet<(u32, N::BlockHash)>,
    ) -> Result<bool> {
        let prev_height = block_id.height.saturating_sub(1);

        // Did the ledger already commit a previous block?
        if let Ok(hash) = self.ledger.get_block_hash(prev_height) {
            if hash == block_id.previous_hash {
                return Ok(true);
            } else {
                //TODO block peer here
                bail!("Previous block hash is incompatible with ledger");
            }
        }

        // Are we about to issue a new request for this?
        if queued_requests.contains(&(prev_height, block_id.previous_hash)) {
            return Ok(true);
        }

        // Did we already issue a request for this?
        if self.state.read().block_to_request.contains_key(&block_id.previous_hash) {
            return Ok(true);
        }

        Ok(false)
    }

    /* TODO is this still needed?
    /// Checks that a block request for the given block ID does not already exist.
    fn check_block_request(&self, block_id: &BlockId<N>, queued_requests: &HashSet<(u32, N::BlockHash)>) -> Result<()> {
        // Ensure the block height is not already in the ledger.
        if self.ledger.contains_block_height(block_id.height) {
            bail!("Failed to add block request, as block with height {} already exists in the ledger", block_id.height);
        }

        if queued_requests.contains(&(block_id.height, block_id.hash)) {
            bail!("Block was already requested");
        }

        // Ensure the block height is not already requested.
        if self.state.read().block_to_request.contains_key(&block_id.hash) {
            bail!("Failed to add block request, as an identical request already exists in the requests map");
        }

        Ok(())
    }*/

    /// Checks the given block (response) from a peer against the expected block hash and previous block hash.
    fn check_block_response(&self, peer_ip: &SocketAddr, block: &Block<N>) -> Result<()> {
        // Retrieve the block height.
        let height = block.height();

        // Retrieve the request entry for the candidate block.
        if let Some(request) = self.state.read().requests.get(&(*peer_ip, block.height())) {
            let Some(request_previous_hash) = request.previous_block_hash() else {
                bail!("Request is empty");
            };

            // Ensure the previous block hash matches if it exists.
            if block.previous_hash() != request_previous_hash {
                bail!("The previous block hash in candidate block {height} from '{peer_ip}' is incorrect")
            }
            // Ensure the sync pool requested this block from the given peer.
            if request.sync_peer != *peer_ip {
                bail!("The sync pool did not request block {height} from '{peer_ip}'")
            }
            return Ok(());
        } else if self.ledger.contains_block_height(height) {
            bail!("The sync request was removed because we already advanced")
        }

        bail!("The sync pool did not request block {height}")
    }

    /// Removes the block response and the associated request for the given height.
    ///
    /// This should only be called after `peek_next_block`, which checked if the request for the given height wasi
    /// complete, or when the ledger advanced to the next block.
    pub fn remove_block_request(&self, request_id: &BlockRequestId) {
        // Remove the request entry for the given height.
        let mut state = self.state.write();
        if state.requests.remove(request_id).is_some() {
            // Remove the response entry for the given height.
            state.responses.remove(request_id);
        }
    }

    /// Removes all block requests for the given peer IP.
    ///
    /// This is used when disconnecting from a peer or when a peer sends invalid block responses.
    fn remove_block_requests_to_peer(&self, peer_ip: &SocketAddr) {
        trace!("Block sync is removing all block requests to peer {peer_ip}...");

        // Remove the peer IP from the requests map. If any request entry is now empty,
        // and its corresponding response entry is also empty, then remove that request entry altogether.

        let mut state = self.state.write();
        let mut removed_requests = HashSet::new();

        // Remove timed-out and obsolete block requests.
        // TODO (kaimast): use extract_if here once it is stable
        state.requests.retain(|req_id, request| {
            let remove = request.sync_peer == *peer_ip;
            if remove {
                removed_requests.insert(*req_id);
            }
            !remove
        });

        state.block_to_request.retain(|_, req_id| !removed_requests.contains(req_id));

        for req_id in removed_requests {
            state.responses.remove(&req_id);
        }
    }

    /// Removes block requests that have timed out, i.e, requests we sent that did not receive a response in time, and requests that are obsolete, i.e., the ledger already has all corresponding blocks.
    ///
    /// This removes the corresponding block responses and returns the set of peers/addresses that timed out.
    pub fn remove_timed_out_block_requests(&self) -> HashSet<SocketAddr> {
        // Acquire the write lock on the requests map.
        let mut state = self.state.write();
        //  Acquire the write lock on the locators map.
        let mut locators = self.locators.write();

        // Retrieve the current time.
        let now = Instant::now();

        // Retrieve the current block height
        let current_height = self.ledger.latest_block_height();

        // Track the number of timed out block requests (only used to print a log message).
        let mut num_timed_out_block_requests = 0;

        // Track which peers should be banned due to unresponsiveness.
        let mut peers_to_ban: HashSet<SocketAddr> = HashSet::new();
        let mut removed_requests = HashSet::new();

        // Remove timed-out and obsolete block requests.
        // TODO (kaimast): use extract_if here once it is stable
        state.requests.retain(|req_id, request| {
            let is_obsolete = request.start_height() <= current_height;
            // Determine if the duration since the request timestamp has exceeded the request timeout.
            let is_timeout = now.duration_since(request.timestamp).as_secs() > BLOCK_REQUEST_TIMEOUT_IN_SECS;

            // If the request has timed out, or is obsolete, then remove it.
            if is_timeout || is_obsolete {
                trace!(
                    "Block request {req_id:?} has timed out: is_timeout = {is_timeout}, is_obsolete = {is_obsolete}"
                );

                debug!("Removing peer {} from block request {req_id:?}", request.sync_peer);
                // Remove the locators entry for the given peer IP.
                locators.remove(&request.sync_peer);
                if is_timeout {
                    peers_to_ban.insert(request.sync_peer);
                }

                // Increment the number of timed out block requests.
                num_timed_out_block_requests += 1;
            }
            // Retain if this is not a timeout and is not obsolete.
            let retain = !is_timeout && !is_obsolete;
            if !retain {
                removed_requests.insert(*req_id);
            }
            retain
        });

        state.block_to_request.retain(|_, req_id| !removed_requests.contains(req_id));

        for req_id in removed_requests {
            state.responses.remove(&req_id);
        }

        peers_to_ban
    }

    /// Returns the sync peers and their minimum common ancestor, if the node needs to sync.
    fn find_sync_peers_inner(&self, current_height: u32) -> Vec<(SocketAddr, BlockLocators<N>)> {
        // Pick a set of peers above the latest ledger height, and include their locators.
        // This will sort the peers by locator height in descending order.
        let mut sync_peers: Vec<_> = self
            .locators
            .read()
            .iter()
            .filter(|(_, locators)| locators.latest_locator_height() > current_height)
            .sorted_by_key(|(_, a)| a.latest_locator_height())
            .take(NUM_SYNC_CANDIDATE_PEERS)
            .map(|(peer_ip, locators)| (*peer_ip, locators.clone()))
            .collect();

        // Ensure we don't sync from the same peers in the same order every time.
        sync_peers.shuffle(&mut rand::thread_rng());

        sync_peers
    }

    // Given the sync peers, return a list of block requests.
    fn construct_request(
        &self,
        start_height: u32,
        max_height: u32,
        sync_peer: &SocketAddr,
        peer_locators: &BlockLocators<N>,
        _queued_requests: &HashSet<(u32, N::BlockHash)>,
    ) -> Result<BlockSyncRequest<N>> {
        // Compute the end height for the block request.
        // TODO restore this
        let _max_blocks_to_request = MAX_BLOCK_REQUESTS as u32 * DataBlocks::<N>::MAXIMUM_NUMBER_OF_BLOCKS as u32;
        // We can only fetch/verfiy blocks within GC
        let end_height = (start_height + BatchHeader::<N>::MAX_GC_ROUNDS as u32).min(max_height);

        if end_height <= start_height {
            bail!("Invalid range for blcok request");
        }

        // Construct the block hashes to request.
        let mut blocks = vec![];

        for height in start_height..=end_height {
            let Some(hash) = peer_locators.get_hash(height) else {
                bail!("Missing hash in block locator");
            };

            let Some(previous_hash) = peer_locators.get_hash(height.saturating_sub(1)) else {
                bail!("Missing previous block hash");
            };

            let block_id = BlockId { height, previous_hash, hash };
            blocks.push(block_id);
        }

        Ok(BlockSyncRequest { sync_peer: *sync_peer, blocks, timestamp: Instant::now() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::locators::{
        NUM_RECENT_BLOCKS,
        test_helpers::{
            sample_block_hash,
            sample_block_locators,
            sample_block_locators_with_fork,
            sample_forked_block_hash,
        },
    };

    use snarkos_node_bft_ledger_service::MockLedgerService;
    use snarkvm::{ledger::committee::Committee, prelude::TestRng};

    use indexmap::{IndexSet, indexset};
    use rand::Rng;
    use std::{
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::{
            Arc,
            atomic::{AtomicBool, AtomicU32, Ordering},
        },
    };

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    /// Returns the peer IP for the sync pool.
    fn sample_peer_ip(id: u16) -> SocketAddr {
        assert_ne!(id, 0, "The peer ID must not be 0 (reserved for local IP in testing)");
        SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), id)
    }

    /// Returns a sample committee.
    fn sample_committee() -> Committee<CurrentNetwork> {
        let rng = &mut TestRng::default();
        snarkvm::ledger::committee::test_helpers::sample_committee(rng)
    }

    /// Returns the ledger service, initialized to the given height.
    fn sample_ledger_service(height: u32) -> MockLedgerService<CurrentNetwork> {
        MockLedgerService::new_at_height(sample_committee(), height)
    }

    /// Returns the sync pool, with the ledger initialized to the given height.
    fn sample_sync_at_height(height: u32) -> BlockSync<CurrentNetwork> {
        BlockSync::<CurrentNetwork>::new(Arc::new(sample_ledger_service(height)))
    }

    /// Returns a duplicate (deep copy) of the sync pool with a different ledger height.
    fn duplicate_sync_at_new_height(sync: &BlockSync<CurrentNetwork>, height: u32) -> BlockSync<CurrentNetwork> {
        BlockSync::<CurrentNetwork> {
            last_update: Mutex::new(*sync.last_update.lock()),
            ledger: Arc::new(sample_ledger_service(height)),
            locators: RwLock::new(sync.locators.read().clone()),
            pending_chains: RwLock::new(sync.pending_chains.read().clone()),
            state: RwLock::new(sync.state.read().clone()),
            is_block_synced: AtomicBool::new(sync.is_block_synced.load(Ordering::SeqCst)),
            num_blocks_behind: AtomicU32::new(sync.num_blocks_behind.load(Ordering::SeqCst)),
            advance_with_sync_blocks_lock: Default::default(),
        }
    }

    /// Checks that the sync pool (starting at genesis) returns the correct requests.
    fn check_prepare_block_requests(sync: BlockSync<CurrentNetwork>, peers: IndexSet<SocketAddr>) {
        // Check test assumptions are met.
        assert_eq!(sync.ledger.latest_block_height(), 0, "This test assumes the sync pool is at genesis");

        // Determine the number of peers within range of this sync pool.
        let _num_peers_within_recent_range_of_ledger =
            peers.iter().filter(|peer_ip| sync.get_peer_height(peer_ip).unwrap() < NUM_RECENT_BLOCKS as u32).count();

        // Prepare the block requests.
        let requests = sync.prepare_block_requests().unwrap();

        // If there are no peers, then there should be no requests.
        if peers.is_empty() {
            assert!(requests.is_empty());
            return;
        }

        // Otherwise, there should be requests.
        assert!(requests.len() <= MAX_BLOCK_REQUESTS);

        let mut ids = HashSet::new();

        for request in requests.into_iter() {
            for block_id in request.blocks {
                let is_new = ids.insert(block_id.clone());
                assert!(is_new, "Duplicate request for the same block");

                // assert_eq!(block_id.height, 1 + idx as u32);
                assert_eq!(block_id.hash, sample_block_hash(block_id.height));
                assert_eq!(block_id.previous_hash, sample_block_hash(block_id.height - 1));
            }

            /* TODO
            if num_peers_within_recent_range_of_ledger >= REDUNDANCY_FACTOR {
                assert_eq!(sync_peers.len(), 1);
            } else {
                assert_eq!(sync_peers.len(), num_peers_within_recent_range_of_ledger);
                assert_eq!(sync_peers, peers);
            }*/
        }
    }

    /// Tests that height and hash values are set correctly using many different maximum block heights.
    #[test]
    fn test_get_block_height_and_hash() {
        for _ in 0..1000 {
            let height = rand::thread_rng().gen_range(0..100_002u32);

            let sync = sample_sync_at_height(height);

            // Check that the latest blokc height is the maximum height.
            assert_eq!(sync.ledger.latest_block_height(), height);

            // Check the hash to height mapping
            assert_eq!(sync.ledger.get_block_height(&sample_block_hash(0)).unwrap(), 0);
            assert_eq!(sync.ledger.get_block_height(&sample_block_hash(height)).unwrap(), height);

            // Check the height to hash mapping
            assert_eq!(sync.ledger.get_block_hash(0).unwrap(), sample_block_hash(0));
            assert_eq!(sync.ledger.get_block_hash(height).unwrap(), sample_block_hash(height));
        }
    }

    #[test]
    fn test_prepare_block_requests() {
        for num_peers in 0..111 {
            println!("Testing with {num_peers} peers");

            let sync = sample_sync_at_height(0);

            let mut peers = indexset![];

            for peer_id in 1..=num_peers {
                // Add a peer.
                sync.update_peer_locators(sample_peer_ip(peer_id), sample_block_locators(10)).unwrap();
                // Add the peer to the set of peers.
                peers.insert(sample_peer_ip(peer_id));
            }

            // If all peers are ahead, then requests should be prepared.
            check_prepare_block_requests(sync, peers);
        }
    }

    #[test]
    fn test_prepare_block_requests_with_leading_fork_at_11() {
        let sync = sample_sync_at_height(0);

        // Intuitively, peer 1's fork is above peer 2 and peer 3's height.
        // So from peer 2 and peer 3's perspective, they don't even realize that peer 1 is on a fork.
        // Thus, you can sync up to block 10 from any of the 3 peers.

        // When there are NUM_REDUNDANCY peers ahead, and 1 peer is on a leading fork at 11,
        // then the sync pool should request blocks 1..=10 from the NUM_REDUNDANCY peers.
        // This is safe because the leading fork is at 11, and the sync pool is at 0,
        // so all candidate peers are at least 10 blocks ahead of the sync pool.

        const MAX_HEIGHT: u32 = 20;
        const FORK_HEIGHT: u32 = 11;

        // Add a peer (fork).
        let peer_1 = sample_peer_ip(1);
        sync.update_peer_locators(peer_1, sample_block_locators_with_fork(MAX_HEIGHT, FORK_HEIGHT)).unwrap();

        // Add a peer.
        let peer_2 = sample_peer_ip(2);
        sync.update_peer_locators(peer_2, sample_block_locators(FORK_HEIGHT - 1)).unwrap();

        // Add a peer.
        let peer_3 = sample_peer_ip(3);
        sync.update_peer_locators(peer_3, sample_block_locators(FORK_HEIGHT - 1)).unwrap();

        // Prepare the block requests.
        let requests = sync.prepare_block_requests().unwrap();

        assert!(requests.len() > 1);

        // Check the requests.
        let mut total_num_blocks = 0;
        //
        for request in requests.into_iter() {
            total_num_blocks += request.blocks.len() as u32;

            for block_id in request.blocks {
                if block_id.height >= FORK_HEIGHT {
                    assert_eq!(block_id.hash, sample_forked_block_hash(block_id.height));
                } else {
                    assert_eq!(block_id.hash, sample_block_hash(block_id.height));
                }

                if block_id.height > FORK_HEIGHT {
                    assert_eq!(block_id.previous_hash, sample_forked_block_hash(block_id.height - 1));
                } else {
                    assert_eq!(block_id.previous_hash, sample_block_hash(block_id.height - 1));
                }

                //TODO assert_eq!(sync_peers.len(), 1); // Only 1 needed since we have redundancy factor on this (recent locator) hash.
            }
        }

        // Make sure all blocks are requested.
        assert_eq!(MAX_HEIGHT, total_num_blocks);
    }

    #[test]
    fn test_prepare_block_requests_with_leading_fork_at_10() {
        let sync = sample_sync_at_height(0);

        // Intuitively, peer 1's fork is at peer 2 and peer 3's height.
        // So from peer 2 and peer 3's perspective, they recognize that peer 1 has forked.
        // Thus, you don't have NUM_REDUNDANCY peers to sync to block 10.
        //
        // Now, while you could in theory sync up to block 9 from any of the 3 peers,
        // we choose not to do this as either side is likely to disconnect from us,
        // and we would rather wait for enough redundant peers before syncing.

        // When there are NUM_REDUNDANCY peers ahead, and 1 peer is on a leading fork at 10,
        // then the sync pool should not request blocks as 1 peer conflicts with the other NUM_REDUNDANCY-1 peers.
        // We choose to sync with a cohort of peers that are *consistent* with each other,
        // and prioritize from descending heights (so the highest peer gets priority).

        const FORK_HEIGHT: u32 = 10;
        const MAX_HEIGHT: u32 = 20;

        // Add a peer (fork).
        let peer_1 = sample_peer_ip(1);
        sync.update_peer_locators(peer_1, sample_block_locators_with_fork(MAX_HEIGHT, FORK_HEIGHT)).unwrap();

        // Add a peer.
        let peer_2 = sample_peer_ip(2);
        sync.update_peer_locators(peer_2, sample_block_locators(FORK_HEIGHT)).unwrap();

        // Add a peer.
        let peer_3 = sample_peer_ip(3);
        sync.update_peer_locators(peer_3, sample_block_locators(FORK_HEIGHT)).unwrap();

        // Prepare the block requests.
        let requests = sync.prepare_block_requests().unwrap();

        // There is a duplicate block at FORK_HEIGHT.
        assert!(requests.len() > 1);

        let mut total_num_blocks = 0;

        // Check the requests.
        for request in requests.into_iter() {
            total_num_blocks += request.blocks.len() as u32;

            for block_id in request.blocks {
                match block_id.height.cmp(&FORK_HEIGHT) {
                    std::cmp::Ordering::Less => {
                        assert_eq!(block_id.hash, sample_block_hash(block_id.height));
                    }
                    std::cmp::Ordering::Greater => {
                        assert_eq!(request.sync_peer, peer_1);
                        assert_eq!(block_id.hash, sample_forked_block_hash(block_id.height));
                    }
                    std::cmp::Ordering::Equal => {
                        if request.sync_peer == peer_1 {
                            assert_eq!(block_id.hash, sample_forked_block_hash(block_id.height));
                        } else {
                            assert_eq!(block_id.hash, sample_block_hash(block_id.height));
                        }
                    }
                }

                if block_id.height > FORK_HEIGHT {
                    assert_eq!(block_id.previous_hash, sample_forked_block_hash(block_id.height - 1));
                } else {
                    assert_eq!(block_id.previous_hash, sample_block_hash(block_id.height - 1));
                }

                //TODO assert_eq!(sync_ips.len(), 1); // Only 1 needed since we have redundancy factor on this (recent locator) hash.
                //TODO assert_ne!(sync_ips[0], peer_1); // It should never be the forked peer.
            }
        }

        // Make sure all blocks are requested.
        // (There are two blocks at FORK_HEIGHT)
        assert_eq!(MAX_HEIGHT + 1, total_num_blocks);
    }

    /*
    #[test]
    fn test_prepare_block_requests_with_trailing_fork_at_9() {
        let sync = sample_sync_at_height(0);

        // Peer 1 and 2 diverge from peer 3 at block 10.
        const MAX_HEIGHT: usize = 20;
        const FORK_HEIGHT: usize = 10;

        // Add a peer.
        let peer_1 = sample_peer_ip(1);
        sync.update_peer_locators(peer_1, sample_block_locators(MAX_HEIGHT)).unwrap();

        // Add a peer.
        let peer_2 = sample_peer_ip(2);
        sync.update_peer_locators(peer_2, sample_block_locators(MAX_HEIGHT)).unwrap();

        // Add a peer (fork).
        let peer_3 = sample_peer_ip(3);
        sync.update_peer_locators(peer_3, sample_block_locators_with_fork(MAX_HEIGHT, FORK_HEIGHT)).unwrap();

        // Add a peer.
        let peer_4 = sample_peer_ip(4);
        sync.update_peer_locators(peer_4, sample_block_locators(10)).unwrap();

        // Prepare the block requests.
        let requests = sync.prepare_block_requests();
        assert!(requests.len() > 0);

        // Check the requests.
        for (idx, (block_id, sync_ips)) in requests.into_iter().enumerate() {
            // Construct the sync IPs.
            assert_eq!(block_id.height, 1 + idx as u32);
            assert_eq!(block_id.block_hash, Field::<CurrentNetwork>::from_u32(block_id.height).into());
            assert_eq!(block_id.previous_block_hash, Field::<CurrentNetwork>::from_u32(block_id.height - 1).into());
            assert_eq!(sync_ips.len(), 1); // Only 1 needed since we have redundancy factor on this (recent locator) hash.
            assert_ne!(sync_ips[0], peer_3); // It should never be the forked peer.
        }
    }*/

    #[test]
    fn test_insert_block_requests() {
        const LOCATOR_HEIGHT: u32 = 14;
        const START_HEIGHT: u32 = 0;

        // Ensure there will be more than one request.
        assert!(LOCATOR_HEIGHT > DataBlocks::<CurrentNetwork>::MAXIMUM_NUMBER_OF_BLOCKS as u32);

        let sync = sample_sync_at_height(START_HEIGHT);

        // Add a peer.
        sync.update_peer_locators(sample_peer_ip(1), sample_block_locators(LOCATOR_HEIGHT)).unwrap();

        // Prepare the block requests.
        let requests = sync.prepare_block_requests().unwrap();
        assert!(requests.len() > 1);

        for request in requests.clone() {
            let req_id = request.get_identifier();
            let sync_peer = request.sync_peer;

            // Insert the block request.
            sync.insert_block_request(request).unwrap();
            // Check that the block requests were inserted.
            let request = sync.get_block_request(&req_id).unwrap();
            assert_eq!(request.sync_peer, sync_peer);
        }

        for request in requests.clone() {
            let req_id = request.get_identifier();
            let sync_peer = request.sync_peer;

            // Check that the block requests are still inserted.
            let request = sync.get_block_request(&req_id).unwrap();
            assert_eq!(request.sync_peer, sync_peer);
        }

        for request in requests {
            let req_id = request.get_identifier();
            let sync_peer = request.sync_peer;

            // Check that block requests cannot be inserted twice.
            assert!(sync.insert_block_request(request).is_err());
            // Check that the block requests are still inserted.
            let request = sync.get_block_request(&req_id).unwrap();
            assert_eq!(request.sync_peer, sync_peer);
        }
    }

    /* TODO
    #[test]
    fn test_insert_block_requests_fails() {
        let sync = sample_sync_at_height(9);

        // Add a peer.
        sync.update_peer_locators(sample_peer_ip(1), sample_block_locators(10)).unwrap();

        // Inserting a block height that is already in the ledger should fail.
        sync.insert_block_request(9, (None, None, indexset![sample_peer_ip(1)])).unwrap_err();
        // Inserting a block height that is not in the ledger should succeed.
        sync.insert_block_request(10, (None, None, indexset![sample_peer_ip(1)])).unwrap();
    }*/

    #[test]
    fn test_update_peer_locators() {
        let sync = sample_sync_at_height(0);

        let peer_ip = sample_peer_ip(1);
        for peer_height in 0..500u32 {
            sync.update_peer_locators(peer_ip, sample_block_locators(peer_height)).unwrap();
            assert_eq!(sync.get_peer_height(&peer_ip), Some(peer_height));
        }
    }

    #[test]
    fn test_remove_peer() {
        let sync = sample_sync_at_height(0);

        let peer_ip = sample_peer_ip(1);
        sync.update_peer_locators(peer_ip, sample_block_locators(100)).unwrap();
        assert_eq!(sync.get_peer_height(&peer_ip), Some(100));

        sync.remove_peer(&peer_ip);
        assert_eq!(sync.get_peer_height(&peer_ip), None);

        sync.update_peer_locators(peer_ip, sample_block_locators(200)).unwrap();
        assert_eq!(sync.get_peer_height(&peer_ip), Some(200));

        sync.remove_peer(&peer_ip);
        assert_eq!(sync.get_peer_height(&peer_ip), None);
    }

    #[test]
    fn test_locators_insert_remove_insert() {
        let sync = sample_sync_at_height(0);

        let peer_ip = sample_peer_ip(1);
        sync.update_peer_locators(peer_ip, sample_block_locators(100)).unwrap();
        assert_eq!(sync.get_peer_height(&peer_ip), Some(100));

        sync.remove_peer(&peer_ip);
        assert_eq!(sync.get_peer_height(&peer_ip), None);

        sync.update_peer_locators(peer_ip, sample_block_locators(200)).unwrap();
        assert_eq!(sync.get_peer_height(&peer_ip), Some(200));
    }

    #[test]
    fn test_requests_insert_remove_insert() {
        let peer_height = 3;

        // Ensure all blocks fit in  a single request.
        assert!(peer_height <= DataBlocks::<CurrentNetwork>::MAXIMUM_NUMBER_OF_BLOCKS as usize);

        let sync = sample_sync_at_height(0);

        // Add a peer.
        let peer_ip = sample_peer_ip(1);
        sync.update_peer_locators(peer_ip, sample_block_locators(peer_height as u32)).unwrap();

        // Prepare the block requests.
        let requests = sync.prepare_block_requests().unwrap();
        assert_eq!(requests.len(), 1);

        for request in requests.clone() {
            assert_eq!(request.start_height(), 1);
            assert_eq!(request.blocks.len(), peer_height);

            // Insert the block request.
            sync.insert_block_request(request.clone()).unwrap();
            // Check that the block requests were inserted.
            let stored_request = sync.get_block_request(&request.get_identifier()).unwrap();
            assert_eq!(request.sync_peer, stored_request.sync_peer);
        }

        // Remove the peer.
        sync.remove_peer(&peer_ip);

        for request in requests {
            // Check that the block requests were removed.
            assert!(sync.get_block_request(&request.get_identifier()).is_none());
        }

        // As there is no peer, it should not be possible to prepare block requests.
        let requests = sync.prepare_block_requests().unwrap();
        assert_eq!(requests.len(), 0);

        // Add the peer again.
        sync.update_peer_locators(peer_ip, sample_block_locators(peer_height as u32)).unwrap();

        // Prepare the block requests.
        let requests = sync.prepare_block_requests().unwrap();
        assert_eq!(requests.len(), 1);

        let request = requests.first().unwrap();

        assert_eq!(request.start_height(), 1);
        assert_eq!(request.blocks.len(), peer_height);
        // Insert the block request.
        sync.insert_block_request(request.clone()).unwrap();
        // Check that the block requests were inserted.
        let stored_request = sync.get_block_request(&request.get_identifier()).unwrap();
        assert_eq!(request.sync_peer, stored_request.sync_peer);
    }

    #[test]
    fn test_obsolete_block_request() {
        let rng = &mut TestRng::default();
        let sync = sample_sync_at_height(0);

        // Set the height to some multiple of the maximum message size.i
        let min_height = 2 * DataBlocks::<CurrentNetwork>::MAXIMUM_NUMBER_OF_BLOCKS as u32;
        let max_height = 21 * DataBlocks::<CurrentNetwork>::MAXIMUM_NUMBER_OF_BLOCKS as u32;

        let locator_height = rng.gen_range(min_height..max_height);

        // Add a peer.
        let locators = sample_block_locators(locator_height);
        sync.update_peer_locators(sample_peer_ip(1), locators.clone()).unwrap();

        // Construct block requests
        let requests = sync.prepare_block_requests().unwrap();

        // The blocks cannot fit in a single request.
        assert!(requests.len() as u32 > 1);

        // Add the block requests to the sync module.
        for request in requests.clone() {
            // Insert the block request.
            sync.insert_block_request(request.clone()).unwrap();
            // Check that the block requests were inserted.
            let stored_request = sync.get_block_request(&request.get_identifier()).unwrap();
            assert_eq!(stored_request.sync_peer, request.sync_peer);
        }

        // Duplicate a new sync module with a different height to simulate block advancement.
        // This range needs to be inclusive, so that the range is never empty,
        // even with a locator height of 0.
        let ledger_height = rng.gen_range(0..=locator_height);
        let new_sync = duplicate_sync_at_new_height(&sync, ledger_height);

        // Check that the number of requests is the same.
        assert_eq!(new_sync.state.read().requests.len(), requests.len());

        // Remove timed out block requests.
        new_sync.remove_timed_out_block_requests();

        // Check that the number of requests is reduced based on the ledger height.
        for (_, request) in new_sync.state.read().requests.iter() {
            assert!(request.end_height() > ledger_height);
        }
    }

    // TODO: duplicate responses, ensure fails.
}
