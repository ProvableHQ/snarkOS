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
    helpers::{PeerPair, PrepareSyncRequest, SyncRequest},
    locators::BlockLocators,
};
use snarkos_node_bft_ledger_service::LedgerService;
use snarkos_node_network::PeerPoolHandling;
use snarkos_node_router::messages::DataBlocks;
use snarkos_node_sync_communication_service::CommunicationService;

use snarkvm::{
    console::network::{ConsensusVersion, Network},
    prelude::block::Block,
    utilities::ensure_equals,
};

use anyhow::{Result, bail, ensure};
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
#[cfg(feature = "locktick")]
use locktick::{parking_lot::Mutex, parking_lot::RwLock, tokio::Mutex as TMutex};
#[cfg(not(feature = "locktick"))]
use parking_lot::{Mutex, RwLock};
use rand::seq::{IteratorRandom, SliceRandom};
use std::{
    collections::{BTreeMap, HashMap, HashSet, hash_map},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::{Duration, Instant},
};

#[cfg(not(feature = "locktick"))]
use tokio::sync::Mutex as TMutex;
use tokio::sync::Notify;

mod helpers;
use helpers::rangify_heights;

mod sync_state;
use sync_state::SyncState;

mod metrics;
use metrics::BlockSyncMetrics;

// The redundancy factor decreases the possibility of a malicious peers sending us an invalid block locator
// by requiring multiple peers to advertise the same (prefix of) block locators.
// However, we do not use this in production yet.
#[cfg(not(test))]
pub const REDUNDANCY_FACTOR: usize = 1;
#[cfg(test)]
pub const REDUNDANCY_FACTOR: usize = 3;

/// The time nodes wait between issuing batches of block requests to avoid triggering spam detection.
///
/// The current rate limit for all messages is around 160k  per second (see [`Gateway::max_cache_events`]).
/// This constant limits number of block requests to a much lower 100 per second.
///
// TODO(kaimast): base rate limits on how many requests were sent to each peer instead.
pub const BLOCK_REQUEST_BATCH_DELAY: Duration = Duration::from_millis(10);

const EXTRA_REDUNDANCY_FACTOR: usize = REDUNDANCY_FACTOR * 3;
const NUM_SYNC_CANDIDATE_PEERS: usize = REDUNDANCY_FACTOR * 5;

const BLOCK_REQUEST_TIMEOUT: Duration = Duration::from_secs(600);

/// The maximum number of outstanding block requests.
/// Once a node hits this limit, it will not issue any new requests until existing requests time out or receive responses.
const MAX_BLOCK_REQUESTS: usize = 50; // 50 requests

/// The maximum number of blocks tolerated before the primary is considered behind its peers.
pub const MAX_BLOCKS_BEHIND: u32 = 1; // blocks

/// This is a dummy IP address that is used to represent the local node.
/// Note: This here does not need to be a real IP address, but it must be unique/distinct from all other connections.
pub const DUMMY_SELF_IP: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 0);

/// Handle to an outstanding requested, containing the request itself and its timestamp.
/// This does not contain the response so that checking for responses does not require iterating over all requests.
#[derive(Clone)]
struct OutstandingRequest<N: Network> {
    request: SyncRequest<N>,
    timestamp: Instant,
    /// The corresponding response (if any).
    /// This is guaranteed to be Some if sync_ips for the given request are empty.
    response: Option<Block<N>>,
}

/// Information about a block request (used for the REST API).
#[derive(Clone, serde::Serialize)]
pub struct BlockRequestInfo {
    /// Seconds since the request was created
    elapsed: u64,
    /// Has the request been responded to?
    done: bool,
}

/// Summary of completed all in-flight requests.
#[derive(Clone, serde::Serialize)]
pub struct BlockRequestsSummary {
    outstanding: String,
    completed: String,
}

impl<N: Network> OutstandingRequest<N> {
    /// Get a reference to the IPs of peers that have not responded to the request (yet).
    fn sync_ips(&self) -> &IndexSet<SocketAddr> {
        let (_, _, sync_ips) = &self.request;
        sync_ips
    }

    /// Get a mutable reference to the IPs of peers that have not responded to the request (yet).
    fn sync_ips_mut(&mut self) -> &mut IndexSet<SocketAddr> {
        let (_, _, sync_ips) = &mut self.request;
        sync_ips
    }
}

struct BlockHeights {
    /// Advertised block height and last requested sync height for each peers.
    peer_heights: HashMap<SocketAddr, (u32, u32)>,
    /// The position at which we are syncing right now.
    sync_height: u32,
}

enum RemovePeerHeightResult {
    Unchanged,
    NoSuchPeer,
    NewHeight(u32),
    NoPeersLeft,
}

impl BlockHeights {
    /// Removes a peer and its peer height.
    fn remove_peer(&mut self, peer_ip: &SocketAddr) -> RemovePeerHeightResult {
        let Some((removed, _)) = self.peer_heights.remove(peer_ip) else {
            return RemovePeerHeightResult::NoSuchPeer;
        };

        let Some(new_max) = self.peer_heights.values().map(|(h, _)| h).max().copied() else {
            return RemovePeerHeightResult::NoPeersLeft;
        };

        if new_max < removed { RemovePeerHeightResult::NewHeight(new_max) } else { RemovePeerHeightResult::Unchanged }
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
pub struct BlockSync<N: Network> {
    /// The ledger.
    ledger: Arc<dyn LedgerService<N>>,

    /// The map of peer IP to their block locators.
    /// The block locators are consistent with the ledger and every other peer's block locators.
    locators: RwLock<HashMap<SocketAddr, BlockLocators<N>>>,

    /// The map of peer-to-peer to their common ancestor.
    /// This map is used to determine which peers to request blocks from.
    ///
    /// Lock ordering: when locking both, `common_ancestors` and `locators`, `common_ancestors` must be locked first.
    common_ancestors: RwLock<IndexMap<PeerPair, u32>>,

    /// The block requests in progress and their responses.
    requests: RwLock<BTreeMap<u32, OutstandingRequest<N>>>,

    /// The boolean indicator of whether the node is synced up to the latest block (within the given tolerance).
    ///
    /// Lock ordering: if you lock `sync_state` and `requests`, you must lock `sync_state` first.
    sync_state: RwLock<SyncState>,

    /// The lock used to ensure that [`Self::advance_with_sync_blocks()`] is called by one task at a time.
    advance_with_sync_blocks_lock: TMutex<()>,

    /// Gets notified when there was an update to the locators or a peer disconnected.
    peer_notify: Notify,

    /// Gets notified when we received a new block response.
    response_notify: Notify,

    /// Tracks sync speed
    metrics: BlockSyncMetrics,

    /// The peer heights and current sync height.
    block_heights: Arc<Mutex<BlockHeights>>,
}

impl<N: Network> BlockSync<N> {
    /// Initializes a new block sync module.
    pub fn new(ledger: Arc<dyn LedgerService<N>>) -> Self {
        // Make sync state aware of the blocks that already exist on disk at startup.
        let sync_state = SyncState::new_with_height(ledger.latest_block_height());
        let block_heights = BlockHeights { peer_heights: Default::default(), sync_height: 0 };

        Self {
            ledger,
            sync_state: RwLock::new(sync_state),
            peer_notify: Default::default(),
            response_notify: Default::default(),
            block_heights: Arc::new(Mutex::new(block_heights)),
            locators: Default::default(),
            requests: Default::default(),
            common_ancestors: Default::default(),
            advance_with_sync_blocks_lock: Default::default(),
            metrics: Default::default(),
        }
    }

    /// Blocks until something about a peer changes,
    /// or block request has been fully processed (either successfully or unsuccessfully).
    ///
    /// Used by the outgoing task.
    pub async fn wait_for_peer_update(&self) {
        self.peer_notify.notified().await
    }

    /// Blocks until there is a new response to a block request.
    ///
    /// Used by the incoming task.
    pub async fn wait_for_block_responses(&self) {
        self.response_notify.notified().await
    }

    /// Returns `true` if the node is synced up to the latest block (within the given tolerance).
    #[inline]
    pub fn is_block_synced(&self) -> bool {
        self.sync_state.read().is_block_synced()
    }

    /// Returns `true` if there a blocks to fetch or responses to process.
    ///
    /// This will always return true if [`Self::is_block_synced`] returns false,
    /// but it can return true when [`Self::is_block_synced`] returns true
    /// (due to the latter having a tolerance of one block).
    #[inline]
    pub fn can_block_sync(&self) -> bool {
        self.sync_state.read().can_block_sync() || self.has_pending_responses()
    }

    /// Returns the number of blocks the node is behind the greatest peer height,
    /// or `None` if no peers are connected yet.
    #[inline]
    pub fn num_blocks_behind(&self) -> Option<u32> {
        self.sync_state.read().num_blocks_behind()
    }

    /// Returns the greatest block height of any connected peer.
    #[inline]
    pub fn greatest_peer_block_height(&self) -> Option<u32> {
        self.sync_state.read().get_greatest_peer_height()
    }

    /// Returns the current sync height of this node.
    /// The sync height is always greater or equal to the ledger height.
    #[inline]
    pub fn get_sync_height(&self) -> u32 {
        self.sync_state.read().get_sync_height()
    }

    /// Returns the number of blocks we requested from peers, but have not received yet.
    #[inline]
    pub fn num_outstanding_block_requests(&self) -> usize {
        self.requests.read().iter().filter(|(_, e)| !e.sync_ips().is_empty()).count()
    }

    /// The total number of block request, including the ones that have been answered already but not processed yet.
    #[inline]
    pub fn num_total_block_requests(&self) -> usize {
        self.requests.read().len()
    }

    //// Returns the latest locator height for all known peers.
    pub fn get_peer_heights(&self) -> HashMap<SocketAddr, u32> {
        self.block_heights.lock().peer_heights.iter().map(|(addr, (h, _))| (*addr, *h)).collect()
    }

    //// Returns information about all in-flight block requests.
    pub fn get_block_requests_info(&self) -> BTreeMap<u32, BlockRequestInfo> {
        self.requests
            .read()
            .iter()
            .map(|(height, request)| {
                (*height, BlockRequestInfo {
                    done: request.sync_ips().is_empty(),
                    elapsed: request.timestamp.elapsed().as_secs(),
                })
            })
            .collect()
    }

    /// Returns a summary of all in-flight requests.
    pub fn get_block_requests_summary(&self) -> BlockRequestsSummary {
        let completed = self
            .requests
            .read()
            .iter()
            .filter_map(|(h, e)| if e.sync_ips().is_empty() { Some(*h) } else { None })
            .collect::<Vec<_>>();

        let outstanding = self
            .requests
            .read()
            .iter()
            .filter_map(|(h, e)| if !e.sync_ips().is_empty() { Some(*h) } else { None })
            .collect::<Vec<_>>();

        BlockRequestsSummary { completed: rangify_heights(&completed), outstanding: rangify_heights(&outstanding) }
    }

    pub fn get_sync_speed(&self) -> f64 {
        self.metrics.get_sync_speed()
    }

    /// Returns the latest block height of the given peer IP.
    pub fn get_peer_height(&self, peer_ip: &SocketAddr) -> Option<u32> {
        self.block_heights.lock().peer_heights.get(peer_ip).map(|(h, _)| *h)
    }
}

#[cfg(test)]
impl<N: Network> BlockSync<N> {
    /*    /// Returns the common ancestor for the given peer pair, if it exists.
    fn get_common_ancestor(&self, peer_a: SocketAddr, peer_b: SocketAddr) -> Option<u32> {
        self.common_ancestors.read().get(&PeerPair(peer_a, peer_b)).copied()
    }*/

    /// Returns the block request for the given height, if it exists.
    fn get_block_request(&self, height: u32) -> Option<SyncRequest<N>> {
        self.requests.read().get(&height).map(|e| e.request.clone())
    }

    /// Returns the timestamp of the last time the block was requested, if it exists.
    fn get_block_request_timestamp(&self, height: u32) -> Option<Instant> {
        self.requests.read().get(&height).map(|e| e.timestamp)
    }
}

impl<N: Network> BlockSync<N> {
    /// Returns block locators for the specified range.
    #[inline]
    pub fn get_block_locators(&self, start: u32, end: u32) -> Result<BlockLocators<N>> {
        ensure!(start < end, "Invalid locator range");
        ensure!((end - start) < 1000, "Locator range too big");

        let mut hashes = vec![];

        for h in start..end {
            hashes.push(self.ledger.get_block_hash(h)?);
        }

        BlockLocators::new(start, hashes)
    }

    /// Returns true if there are pending responses to block requests that need to be processed.
    pub fn has_pending_responses(&self) -> bool {
        self.requests.read().iter().filter(|(_, req)| req.response.is_some() && req.sync_ips().is_empty()).count() > 0
    }

    /// Send a batch of block requests.
    pub async fn send_block_requests<C: CommunicationService>(
        &self,
        communication: &C,
        sync_peers: &IndexMap<SocketAddr, BlockLocators<N>>,
        requests: &[(u32, PrepareSyncRequest<N>)],
    ) -> bool {
        let (start_height, max_num_sync_ips) = match requests.first() {
            Some((height, (_, _, max_num_sync_ips))) => (*height, *max_num_sync_ips),
            None => {
                warn!("Block sync failed - no block requests");
                return false;
            }
        };

        debug!("Sending {len} block requests to peer(s) at {peers:?}", len = requests.len(), peers = sync_peers.keys());

        // Use a randomly sampled subset of the sync IPs.
        let sync_ips: IndexSet<_> =
            sync_peers.keys().copied().choose_multiple(&mut rand::thread_rng(), max_num_sync_ips).into_iter().collect();

        // Calculate the end height.
        let end_height = start_height.saturating_add(requests.len() as u32);

        // Insert the chunk of block requests.
        for (height, (hash, previous_hash, _)) in requests.iter() {
            // Insert the block request into the sync pool using the sync IPs from the last block request in the chunk.
            if let Err(error) = self.insert_block_request(*height, (*hash, *previous_hash, sync_ips.clone())) {
                warn!("Block sync failed - {error}");
                return false;
            }
        }

        /* Send the block request to the peers */

        // Construct the message.
        let message = C::prepare_block_request(start_height, end_height);

        // Send the message to the peers.
        let mut tasks = Vec::with_capacity(sync_ips.len());
        for sync_ip in sync_ips {
            let sender = communication.send(sync_ip, message.clone()).await;
            let task = tokio::spawn(async move {
                // Ensure the request is sent successfully.
                match sender {
                    Some(sender) => {
                        if let Err(err) = sender.await {
                            warn!("Failed to send block request to peer '{sync_ip}': {err}");
                            false
                        } else {
                            true
                        }
                    }
                    None => {
                        warn!("Failed to send block request to peer '{sync_ip}': no such peer");
                        false
                    }
                }
            });

            tasks.push(task);
        }

        // Wait for all sends to finish at the same time.
        for result in futures::future::join_all(tasks).await {
            let success = match result {
                Ok(success) => success,
                Err(err) => {
                    error!("tokio join error: {err}");
                    false
                }
            };

            // If sending fails for any peer, remove the block request from the sync pool.
            if !success {
                // Remove the entire block request from the sync pool.
                let mut requests = self.requests.write();
                for height in start_height..end_height {
                    requests.remove(&height);
                }
                // Break out of the loop.
                return false;
            }
        }
        true
    }

    /// Inserts a new block response from the given peer IP.
    ///
    /// Returns an error if the block was malformed, or we already received a different block for this height.
    /// This function also removes all block requests from the given peer IP on failure.
    ///
    /// Note, that this only queues the response. After this, you most likely want to call `Self::try_advancing_block_synchronization`.
    ///
    #[inline]
    pub fn insert_block_responses(
        &self,
        peer_ip: SocketAddr,
        blocks: Vec<Block<N>>,
        latest_consensus_version: Option<ConsensusVersion>,
    ) -> Result<()> {
        let Some(last_height) = blocks.as_slice().last().map(|b| b.height()) else {
            bail!("Empty block response");
        };

        let expected_consensus_version = N::CONSENSUS_VERSION(last_height)?;

        // Perform consensus version check, if possible.
        // This check is only enabled after nodes have reached V12.
        if expected_consensus_version >= ConsensusVersion::V12 {
            if let Some(latest_consensus_version) = latest_consensus_version {
                ensure_equals!(
                    expected_consensus_version,
                    latest_consensus_version,
                    "the peer's consensus version for height {last_height} does not match ours"
                );
            } else {
                bail!("The peer did not send a consensus version");
            }
        }

        // Insert the candidate blocks into the sync pool.
        for block in blocks {
            if let Err(error) = self.insert_block_response(peer_ip, block) {
                self.remove_block_requests_to_peer(&peer_ip);
                bail!("{error}");
            }
        }
        Ok(())
    }

    /// Returns the next block for the given `next_height` if the request is complete,
    /// or `None` otherwise. This does not remove the block from the `responses` map.
    #[inline]
    pub fn peek_next_block(&self, next_height: u32) -> Option<Block<N>> {
        // Determine if the request is complete:
        // either there is no request for `next_height`, or the request has no peer socket addresses left.
        if let Some(entry) = self.requests.read().get(&next_height) {
            let is_complete = entry.sync_ips().is_empty();
            if !is_complete {
                return None;
            }

            // If the request is complete, return the block from the responses, if there is one.
            if entry.response.is_none() {
                warn!("Request for height {next_height} is complete but no response exists");
            }
            entry.response.clone()
        } else {
            None
        }
    }

    /// Attempts to advance synchronization by processing completed block responses.
    ///
    /// Returns true, if new blocks were added to the ledger.
    ///
    /// # Usage
    /// This is only called in [`Client::try_block_sync`] and should not be called concurrently by multiple tasks.
    /// Validators do not call this function, and instead invoke
    /// [`snarkos_node_bft::Sync::try_advancing_block_synchronization`] which also updates the BFT state.
    #[inline]
    pub async fn try_advancing_block_synchronization(&self) -> Result<bool> {
        // Acquire the lock to ensure this function is called only once at a time.
        // If the lock is already acquired, return early.
        //
        // Note: This lock should not be needed anymore as there is only one place we call it from,
        // but we keep it for now out of caution.
        // TODO(kaimast): remove this eventually.
        let Ok(_lock) = self.advance_with_sync_blocks_lock.try_lock() else {
            trace!("Skipping attempt to advance block synchronziation as it is already in progress");
            return Ok(false);
        };

        // Start with the current height.
        let mut current_height = self.ledger.latest_block_height();
        let start_height = current_height;
        trace!(
            "Try advancing with block responses (at block {current_height}, current sync speed is {})",
            self.get_sync_speed()
        );

        loop {
            let next_height = current_height + 1;

            let Some(block) = self.peek_next_block(next_height) else {
                break;
            };

            // Ensure the block height matches.
            if block.height() != next_height {
                warn!("Block height mismatch: expected {}, found {}", current_height + 1, block.height());
                break;
            }

            let ledger = self.ledger.clone();
            let advanced = tokio::task::spawn_blocking(move || {
                // Try to check the next block and advance to it.
                match ledger.check_next_block(&block) {
                    Ok(_) => match ledger.advance_to_next_block(&block) {
                        Ok(_) => true,
                        Err(err) => {
                            warn!(
                                "Failed to advance to next block (height: {}, hash: '{}'): {err}",
                                block.height(),
                                block.hash()
                            );
                            false
                        }
                    },
                    Err(err) => {
                        warn!(
                            "The next block (height: {}, hash: '{}') is invalid - {err}",
                            block.height(),
                            block.hash()
                        );
                        false
                    }
                }
            })
            .await?;

            // Only count successful requests.
            if advanced {
                self.count_request_completed();
            }

            // Remove the block response.
            self.remove_block_response(next_height);

            // If advancing failed, exit the loop.
            if !advanced {
                break;
            }

            // Update the latest height.
            current_height = next_height;
        }

        if current_height > start_height {
            self.set_sync_height(current_height);
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl<N: Network> BlockSync<N> {
    /// Updates the block locators and common ancestors for the given peer IP.
    ///
    /// This function does not need to check that the block locators are well-formed,
    /// because that is already done in [`BlockLocators::new()`], as noted in [`BlockLocators`].
    ///
    /// This function does **not** check
    /// that the block locators are consistent with the peer's previous block locators or other peers' block locators.
    pub async fn update_peer_block_locators(&self, peer_ip: SocketAddr, locators: BlockLocators<N>) -> Result<()> {
        let peer_height = locators.end_height();

        // Update the peer height in the block_heights structure first
        {
            let mut block_heights = self.block_heights.lock();
            match block_heights.peer_heights.entry(peer_ip) {
                hash_map::Entry::Occupied(mut e) => {
                    let (_, last_sync) = e.get();
                    e.insert((peer_height, *last_sync));
                }
                hash_map::Entry::Vacant(e) => {
                    e.insert((peer_height, 0));
                }
            }
        }

        // If the locators match the existing locators for the peer, return early.
        if self.locators.read().get(&peer_ip) == Some(&locators) {
            return Ok(());
        }

        // Update the locators entry for the given peer IP.
        // We perform this update atomically, and drop the lock as soon as we are done with the update.
        match self.locators.write().entry(peer_ip) {
            hash_map::Entry::Occupied(mut e) => {
                // Return early if the block locators did not change.
                if *e.get() == locators {
                    return Ok(());
                }

                let old_height = e.get().end_height();
                let new_height = locators.end_height();

                if old_height > new_height {
                    debug!("Block height for peer {peer_ip} decreased from {old_height} to {new_height}",);
                }
                e.insert(locators.clone());
            }
            hash_map::Entry::Vacant(e) => {
                e.insert(locators.clone());
            }
        }

        // Compute the common ancestor with this node.
        let new_local_ancestor = {
            let mut ancestor = 0;
            // Attention: Please do not optimize this loop, as it performs fork-detection. In addition,
            // by iterating upwards, it also early-terminates malicious block locators at the *first* point
            // of bifurcation in their ledger history, which is a critical safety guarantee provided here.
            for (height, hash) in locators.clone().into_iter() {
                if let Ok(ledger_hash) = self.ledger.get_block_hash(height) {
                    match ledger_hash == hash {
                        true => ancestor = height,
                        false => {
                            debug!("Detected fork with peer \"{peer_ip}\" at height {height}");
                            break;
                        }
                    }
                }
            }
            ancestor
        };

        // Compute the common ancestor with every other peer.
        // Do not hold write lock to `common_ancestors` here, because this can take a while with many peers.
        let ancestor_updates: Vec<_> = self
            .locators
            .read()
            .iter()
            .filter_map(|(other_ip, other_locators)| {
                // Skip if the other peer is the given peer.
                if other_ip == &peer_ip {
                    return None;
                }
                // Compute the common ancestor with the other peer.
                let mut ancestor = 0;
                for (height, hash) in other_locators.clone().into_iter() {
                    if let Some(expected_hash) = locators.get_hash(height) {
                        match expected_hash == hash {
                            true => ancestor = height,
                            false => {
                                debug!(
                                    "Detected fork between peers \"{other_ip}\" and \"{peer_ip}\" at height {height}"
                                );
                                break;
                            }
                        }
                    }
                }

                Some((PeerPair(peer_ip, *other_ip), ancestor))
            })
            .collect();

        // Update the map of common ancestors.
        // Scope the lock, so it is dropped before locking `sync_state`.
        {
            let mut common_ancestors = self.common_ancestors.write();
            common_ancestors.insert(PeerPair(DUMMY_SELF_IP, peer_ip), new_local_ancestor);

            for (peer_pair, new_ancestor) in ancestor_updates.into_iter() {
                common_ancestors.insert(peer_pair, new_ancestor);
            }

            // Also ensure all peer-to-peer relationships are computed
            // This is needed for find_sync_peers to work correctly
            let current_locators = self.locators.read();
            for (peer_a, locators_a) in current_locators.iter() {
                for (peer_b, locators_b) in current_locators.iter() {
                    if peer_a >= peer_b {
                        continue; // Skip duplicate pairs and self-pairs
                    }

                    let pair = PeerPair(*peer_a, *peer_b);
                    if !common_ancestors.contains_key(&pair) {
                        // Compute common ancestor between these two peers
                        let mut ancestor = 0;
                        for (height, hash_a) in locators_a.clone().into_iter() {
                            if let Some(hash_b) = locators_b.get_hash(height) {
                                if hash_a == hash_b {
                                    ancestor = height;
                                } else {
                                    break;
                                }
                            }
                        }
                        common_ancestors.insert(pair, ancestor);
                    }
                }
            }
        }

        // Update `is_synced`.
        if let Some(greatest_peer_height) = self.block_heights.lock().peer_heights.values().map(|(h, _)| h).max() {
            self.sync_state.write().set_greatest_peer_height(*greatest_peer_height);
        }

        // Notify the sync loop that something changed.
        self.peer_notify.notify_one();

        Ok(())
    }

    /// TODO (howardwu): Remove the `common_ancestor` entry. But check that this is safe
    ///  (that we don't rely upon it for safety when we re-connect with the same peer).
    /// Removes the peer from the sync pool, if they exist.
    pub fn remove_peer(&self, peer_ip: &SocketAddr) {
        trace!("Removing peer {peer_ip} from block sync");

        // Remove the locators entry for the given peer IP.
        self.locators.write().remove(peer_ip);
        // Remove all common ancestor entries for this peers.
        self.common_ancestors.write().retain(|pair, _| !pair.contains(peer_ip));
        // Remove all block requests to the peer.
        self.remove_block_requests_to_peer(peer_ip);

        // Remove from block_heights
        // Use try_lock to avoid blocking, but if it fails we'll spawn a task
        let peer_ip = *peer_ip;
        match self.block_heights.lock().remove_peer(&peer_ip) {
            RemovePeerHeightResult::Unchanged | RemovePeerHeightResult::NoSuchPeer => (),
            RemovePeerHeightResult::NewHeight(new_height) => {
                self.sync_state.write().set_greatest_peer_height(new_height);
            }
            RemovePeerHeightResult::NoPeersLeft => {
                self.sync_state.write().clear_greatest_peer_height();
            }
        }

        // Notify the sync loop that something changed.
        self.peer_notify.notify_one();
    }
}

// Helper type for prepare_block_requests
pub type BlockRequestBatch<N> = (Vec<(u32, PrepareSyncRequest<N>)>, IndexMap<SocketAddr, BlockLocators<N>>);

impl<N: Network> BlockSync<N> {
    /// Returns a list of block requests and the sync peers, if the node needs to sync.
    ///
    /// You usually want to call `remove_timed_out_block_requests` before invoking this function.
    ///
    /// # Concurrency
    /// This should be called by at most one task at a time.
    ///
    /// # Usage
    ///  - For validators, the primary spawns exactly one task that periodically calls
    ///    `bft::Sync::try_issuing_block_requests`. There is no possibility of concurrent calls to it.
    ///  - For clients, `Client::initialize_sync` spawn exactly one task that periodically calls
    ///    `Client::try_issuing_block_requests` which calls this function.
    ///  - Provers do not call this function.
    pub async fn prepare_block_requests<C: CommunicationService>(&self, communication: &C) -> BlockRequestBatch<N> {
        // Used to print more information when we max out on requests.
        let print_requests = || {
            if tracing::enabled!(tracing::Level::TRACE) {
                let summary = self.get_block_requests_summary();

                trace!("The following requests are complete but not processed yet: {:?}", summary.completed);
                trace!("The following requests are still outstanding: {:?}", summary.outstanding);
            }
        };

        // Do not hold lock here as, currently, `find_sync_peers_inner` can take a while.
        let current_height = self.get_sync_height();

        // Ensure to not exceed the maximum number of outstanding block requests.
        let max_outstanding_block_requests =
            (MAX_BLOCK_REQUESTS as u32) * (DataBlocks::<N>::MAXIMUM_NUMBER_OF_BLOCKS as u32);

        // Ensure there is a finite bound on the number of block respnoses we receive, that have not been processed yet.
        let max_total_requests = 4 * max_outstanding_block_requests;

        let max_new_blocks_to_request =
            max_outstanding_block_requests.saturating_sub(self.num_outstanding_block_requests() as u32);

        // Prepare the block requests.
        let (block_requests, sync_peers) = if self.num_total_block_requests() >= max_total_requests as usize {
            trace!(
                "We are already requested at least {max_total_requests} blocks that have not been fully processed yet. Will not issue more."
            );

            print_requests();
            Default::default()
        } else if max_new_blocks_to_request == 0 {
            trace!(
                "Already reached the maximum number of outstanding blocks ({max_outstanding_block_requests}). Will not issue more."
            );

            print_requests();

            // Return an empty list of block requests.
            (Default::default(), Default::default())
        } else if let Some((sync_peers, min_common_ancestor)) = self.find_sync_peers(current_height) {
            // Retrieve the highest block height.
            let greatest_peer_height = sync_peers.values().map(|l| l.end_height()).max().unwrap_or(0);
            // Update the state of `is_block_synced` for the sync module.
            self.sync_state.write().set_greatest_peer_height(greatest_peer_height);
            // Return the list of block requests.
            (
                self.construct_requests(
                    &sync_peers,
                    current_height,
                    min_common_ancestor,
                    max_new_blocks_to_request,
                    greatest_peer_height,
                ),
                sync_peers,
            )
        } else {
            // Update `is_block_synced` if there are no pending requests.
            if self.requests.read().is_empty() {
                trace!("All requests have been processed. Will set block synced to true.");
                // Update the state of `is_block_synced` for the sync module.
                self.sync_state.write().set_greatest_peer_height(0);
            } else {
                trace!("No new blocks can be requests, but there are still outstanding requests.");
            }

            // Return an empty list of block requests.
            (Default::default(), Default::default())
        };

        // Can we advance with block locators?
        if block_requests.is_empty() {
            self.fetch_new_block_locators(communication).await;
        }

        (block_requests, sync_peers)
    }

    /*    async fn fetch_new_block_locators<C: CommunicationService>(&self, communication: &C) {
            let (block_requests, new_sync_height) = {
                let lock = self.block_heights.read();
                let ledger_height = self.ledger.latest_block_height();

                // Check if we are synced with current block locators and can advance.
                if lock.sync_height > ledger_height {
                    // Not ready yet.
                    return;
                }

                let max_peer_height = *lock.peer_heights.values().map(|(advertised, _)| advertised).max().unwrap_or(&0);

                let new_sync_height = (lock.sync_height + 100).min(max_peer_height);
                trace!("Moving from sync_height {} to {new_sync_height}", lock.sync_height);

                let mut messages = vec![];

                for (peer_ip, (advertised, last_sync)) in lock.peer_heights.iter() {
                    if *last_sync < new_sync_height && *advertised > *last_sync {
                        let new_sync = new_sync_height.min(*advertised);
                        let msg = C::prepare_block_locators_request(new_sync.saturating_sub(100), new_sync);

                        messages.push((*peer_ip, new_sync, msg));
                    }
                }

                (messages, new_sync_height)
            };

            // Avoid holding the lock across await points.
            let mut results = vec![];
            for (peer_ip, new_sync, message) in block_requests.into_iter() {
                let Some(fut) = communication.send(peer_ip, message).await else {
                    error!("Failed to send message to peer {peer_ip}");
                    continue;
                };

                results.push((peer_ip, new_sync, fut.await));
            }

            // The number of peers we successfully request new block locators from.
            let mut count = 0;

            {
                let mut lock = self.block_heights.lock().await;
                for (peer_ip, new_sync, result) in results.into_iter() {
                    match result {
                        Ok(_) => {
                            if let Some((_, last_sync)) = lock.peer_heights.get_mut(&peer_ip) {
                                *last_sync = new_sync;
                                count += 1;
                            } else {
                                warn!("Missing entry for {peer_ip}");
                            }
                        }
                        Err(err) => {
                            error!("Failed to request block locators: {err}");
                        }
                    }
                }

                //TODO (kaimast): can count be zero here, ever?
                if count > 0 {
                    debug!("Requested new block locators from {count} peers");
                }

                lock.sync_height = new_sync_height;
            }

            // Can we advance with block locators?
            if .is_empty() {
                self.fetch_new_block_locators(communication).await;
            }

            (block_requests, sync_peers)
        }
    */
    async fn fetch_new_block_locators<C: CommunicationService>(&self, communication: &C) {
        // Generate new requests.
        let futures = {
            let mut lock = self.block_heights.lock();
            let max_peer_height = *lock.peer_heights.values().map(|(advertised, _)| advertised).max().unwrap_or(&0);
            let ledger_height = self.ledger.latest_block_height();

            // Check if we are synced with current block locators and can advance.
            if lock.sync_height > ledger_height {
                // Not ready yet.
                return;
            }

            let new_sync_height = (lock.sync_height + 1000).min(max_peer_height);
            trace!("Moving from sync_height {} to {new_sync_height}", lock.sync_height);

            let futures: Vec<_> = lock
                .peer_heights
                .iter_mut()
                .filter_map(|(peer_ip, (advertised, last_sync))| {
                    if *last_sync < new_sync_height && *advertised > *last_sync {
                        *last_sync = new_sync_height.min(*advertised);
                        let msg = C::prepare_block_locators_request(ledger_height, *last_sync);

                        Some((*peer_ip, communication.send(*peer_ip, msg)))
                    } else {
                        None
                    }
                })
                .collect();

            lock.sync_height = new_sync_height;
            futures
        };

        // Wait for requests to complete.
        let mut count = 0;
        for (peer_ip, fut) in futures.into_iter() {
            let Some(fut) = fut.await else {
                error!("Failed to send message to peer {peer_ip}");
                continue;
            };

            if let Err(err) = fut.await {
                error!("Failed to request block locators: {err}");
            } else {
                count += 1;
            }
        }

        //TODO (kaimast): can count be zero here, ever?
        if count > 0 {
            debug!("Requested new block locators from {count} peers");
        }
    }

    /// Should only be called by validators when they successfully process a block request.
    /// (for other nodes this will be automatically called internally)
    ///
    /// TODO(kaimast): remove this public function once the sync logic is fully unified `BlockSync`.
    pub fn count_request_completed(&self) {
        self.metrics.count_request_completed();
    }

    /// Set the sync height to a the given value.
    /// This is a no-op if `new_height` is equal or less to the current sync height.
    pub fn set_sync_height(&self, new_height: u32) {
        // Scope state lock to avoid locking state and metrics at the same time.
        let fully_synced = {
            let mut state = self.sync_state.write();
            state.set_sync_height(new_height);
            !state.can_block_sync()
        };

        if fully_synced {
            self.metrics.mark_fully_synced();
        }
    }

    /// Inserts a block request for the given height.
    fn insert_block_request(&self, height: u32, (hash, previous_hash, sync_ips): SyncRequest<N>) -> Result<()> {
        // Ensure the block request does not already exist.
        self.check_block_request(height)?;
        // Ensure the sync IPs are not empty.
        ensure!(!sync_ips.is_empty(), "Cannot insert a block request with no sync IPs");
        // Insert the block request.
        self.requests.write().insert(height, OutstandingRequest {
            request: (hash, previous_hash, sync_ips),
            timestamp: Instant::now(),
            response: None,
        });
        Ok(())
    }

    /// Inserts the given block response, after checking that the request exists and the response is well-formed.
    /// On success, this function removes the peer IP from the request sync peers and inserts the response.
    fn insert_block_response(&self, peer_ip: SocketAddr, block: Block<N>) -> Result<()> {
        // Retrieve the block height.
        let height = block.height();
        let mut requests = self.requests.write();

        if self.ledger.contains_block_height(height) {
            bail!("The sync request was removed because we already advanced");
        }

        let Some(entry) = requests.get_mut(&height) else { bail!("The sync pool did not request block {height}") };

        // Retrieve the request entry for the candidate block.
        let (expected_hash, expected_previous_hash, sync_ips) = &entry.request;

        // Ensure the candidate block hash matches the expected hash.
        if let Some(expected_hash) = expected_hash {
            if block.hash() != *expected_hash {
                bail!("The block hash for candidate block {height} from '{peer_ip}' is incorrect")
            }
        }
        // Ensure the previous block hash matches if it exists.
        if let Some(expected_previous_hash) = expected_previous_hash {
            if block.previous_hash() != *expected_previous_hash {
                bail!("The previous block hash in candidate block {height} from '{peer_ip}' is incorrect")
            }
        }
        // Ensure the sync pool requested this block from the given peer.
        if !sync_ips.contains(&peer_ip) {
            bail!("The sync pool did not request block {height} from '{peer_ip}'")
        }

        // Remove the peer IP from the request entry.
        entry.sync_ips_mut().swap_remove(&peer_ip);

        if let Some(existing_block) = &entry.response {
            // If the candidate block was already present, ensure it is the same block.
            if block != *existing_block {
                bail!("Candidate block {height} from '{peer_ip}' is malformed");
            }
        } else {
            entry.response = Some(block.clone());
        }

        trace!("Received a new and valid block response for height {height}");

        // Notify the sync loop that something changed.
        self.response_notify.notify_one();

        Ok(())
    }

    pub fn update_peer_block_height(&self, peer_ip: SocketAddr, new_advertised: u32) -> Result<()> {
        let mut lock = self.block_heights.lock();

        match lock.peer_heights.entry(peer_ip) {
            hash_map::Entry::Occupied(mut e) => {
                let (last_advertised, last_sync) = e.get();
                ensure!(new_advertised >= *last_advertised, "Peer height cannot decrease!");
                e.insert((new_advertised, *last_sync));
            }
            hash_map::Entry::Vacant(e) => {
                e.insert((new_advertised, 0));
            }
        }

        Ok(())
    }

    /// Checks that a block request for the given height does not already exist.
    fn check_block_request(&self, height: u32) -> Result<()> {
        // Ensure the block height is not already in the ledger.
        if self.ledger.contains_block_height(height) {
            bail!("Failed to add block request, as block {height} exists in the ledger");
        }
        // Ensure the block height is not already requested.
        if self.requests.read().contains_key(&height) {
            bail!("Failed to add block request, as block {height} exists in the requests map");
        }

        Ok(())
    }

    /// Removes the block request and response for the given height
    /// This may only be called after `peek_next_block`, which checked if the request for the given height was complete.
    ///
    /// Precondition: This may only be called after `peek_next_block` has returned `Some`,
    /// which has checked if the request for the given height is complete
    /// and there is a block with the given `height` in the `responses` map.
    pub fn remove_block_response(&self, height: u32) {
        // Remove the request entry for the given height.
        if let Some(e) = self.requests.write().remove(&height) {
            trace!(
                "Block request for height {height} was completed in {}ms (sync speed is {})",
                e.timestamp.elapsed().as_millis(),
                self.get_sync_speed()
            );

            // Notify the sending task that less requests are in-flight.
            self.peer_notify.notify_one();
        }
    }

    /// Removes all block requests for the given peer IP.
    ///
    /// This is used when disconnecting from a peer or when a peer sends invalid block responses.
    fn remove_block_requests_to_peer(&self, peer_ip: &SocketAddr) {
        trace!("Block sync is removing all block requests to peer {peer_ip}...");

        // Remove the peer IP from the requests map. If any request entry is now empty,
        // and its corresponding response entry is also empty, then remove that request entry altogether.
        self.requests.write().retain(|height, e| {
            let had_peer = e.sync_ips_mut().swap_remove(peer_ip);

            // Only remove requests that were sent to this peer, that have no other peer that can respond instead,
            // and that were not completed yet.
            let retain = !had_peer || !e.sync_ips().is_empty() || e.response.is_some();
            if !retain {
                trace!("Removed block request timestamp for {peer_ip} at height {height}");
            }
            retain
        });

        // No need to remove responses here, because requests with responses will be retained.
    }

    /// Removes block requests that have timed out, i.e, requests we sent that did not receive a response in time.
    ///
    /// This removes the corresponding block responses and returns the set of peers/addresses that timed out.
    /// It will ask the peer pool handling service to ban any timed-out peers.
    ///
    /// # Return Value
    /// On success it will return `None` if there is nothing to re-request, or a set of new of block requests that replaced the timed-out requests.
    /// This set of new requests can also replace requests that timed out earlier, and which we were not able to re-request yet.
    ///
    /// This function will return an error if it cannot re-request blocks due to a lack of peers.
    /// In this case, the current iteration of block synchronization should not continue and the node should re-try later instead.
    pub fn handle_block_request_timeouts<P: PeerPoolHandling<N>>(
        &self,
        _peer_pool_handler: &P,
    ) -> Result<Option<BlockRequestBatch<N>>> {
        // Acquire the write lock on the requests map.
        let mut requests = self.requests.write();

        // Retrieve the current time.
        let now = Instant::now();

        // Retrieve the current block height
        let current_height = self.ledger.latest_block_height();

        // Track the number of timed out block requests (only used to print a log message).
        let mut timed_out_requests = vec![];

        // Track which peers should be banned due to unresponsiveness.
        let mut peers_to_ban: HashSet<SocketAddr> = HashSet::new();

        // Remove timed out block requests.
        requests.retain(|height, e| {
            let is_obsolete = *height <= current_height;
            // Determine if the duration since the request timestamp has exceeded the request timeout.
            let timer_elapsed = now.duration_since(e.timestamp) > BLOCK_REQUEST_TIMEOUT;
            // Determine if the request is incomplete.
            let is_complete = e.sync_ips().is_empty();

            // Determine if the request has timed out.
            let is_timeout = timer_elapsed && !is_complete;

            // Retain if this is not a timeout and is not obsolete.
            let retain = !is_timeout && !is_obsolete;

            if is_timeout {
                trace!("Block request at height {height} has timed out: timer_elapsed={timer_elapsed}, is_complete={is_complete}, is_obsolete={is_obsolete}");

                // Increment the number of timed out block requests.
                timed_out_requests.push(*height);
            } else if is_obsolete {
                trace!("Block request at height {height} became obsolete (current_height={current_height})");
            }

            // If the request timed out, also remove and ban given peer.
            if is_timeout {
                for peer_ip in e.sync_ips().iter() {
                    peers_to_ban.insert(*peer_ip);
                }
            }

            retain
        });

        if !timed_out_requests.is_empty() {
            debug!("{num} block requests timed out", num = timed_out_requests.len());
        }

        let next_request_height = requests.iter().next().map(|(h, _)| *h);

        // Avoid locking `locators` and `requests` at the same time.
        drop(requests);

        // Now remove and ban any unresponsive peers
        for peer_ip in peers_to_ban {
            self.remove_peer(&peer_ip);
            // TODO: Uncomment this when we have a more rigorous analysis and testing of peer banning.
            // peer_pool_handler.ip_ban_peer(peer_ip, Some("timed out on block requests"));
        }

        // Determine if we need to re-issue any timed-out requests.
        // If there are no requests remaining or no gap at the beginning,
        // we do not need to re-issue requests and will just issue them regularly.
        //
        // This needs to be checked even if timed_out_requests is empty, because we might not be able to re-issue
        // requests immediately if there are no other peers at a given time.
        // Further, this only closes the first gap. So multiple calls to this might be needed.
        let sync_height = self.get_sync_height();
        let start_height = sync_height + 1;

        let end_height = if let Some(next_height) = next_request_height
            && next_height > start_height
        {
            // The end height is exclusive, so use the height of the first existing block requests as the end
            next_height
        } else {
            // Nothing to do.
            // Do not log here as this check happens frequently.
            return Ok(None);
        };

        let Some((sync_peers, min_common_ancestor)) = self.find_sync_peers(start_height) else {
            warn!("Block requests timed out, but found no other peers to re-request from");
            return Ok(None);
        };

        // Retrieve the greatest block height of any connected peer.
        let Some(greatest_peer_height) = sync_peers.values().map(|l| l.end_height()).max() else {
            // This should never happen because `sync_peers` is guaranteed to be non-empty.
            bail!("Cannot re-request blocks because no or not enough peers are connected");
        };

        // Set the maximum number of blocks, so that they do not exceed the end height.
        let max_new_blocks_to_request = end_height - start_height;

        // (Try to) construct the requests.
        let requests = self.construct_requests(
            &sync_peers,
            sync_height,
            min_common_ancestor,
            max_new_blocks_to_request,
            greatest_peer_height,
        );

        // If the ledger advanced concurrenctly, there may be no requests to issue after all.
        // The given height may also be greater `start_height` due to concurerent block advancement.
        if let Some((height, _)) = requests.as_slice().first() {
            debug!("Re-requesting blocks starting at height {height}");
            Ok(Some((requests, sync_peers)))
        } else {
            // Do not log here as this constitutes a benign race condition.
            Ok(None)
        }
    }

    /// Finds the peers to sync from and the shared common ancestor, starting at the give height.
    fn find_sync_peers(&self, current_height: u32) -> Option<(IndexMap<SocketAddr, BlockLocators<N>>, u32)> {
        // Retrieve the latest ledger height.
        let latest_ledger_height = self.ledger.latest_block_height();

        // Pick a set of peers above the latest ledger height, and include their locators.
        // This will sort the peers by locator height in descending order.
        let candidate_locators: IndexMap<_, _> = self
            .locators
            .read()
            .iter()
            .filter(|(_, locators)| locators.end_height() > current_height)
            .sorted_by(|(_, a), (_, b)| b.end_height().cmp(&a.end_height()))
            .take(NUM_SYNC_CANDIDATE_PEERS)
            .map(|(peer_ip, locators)| (*peer_ip, locators.clone()))
            .collect();

        // Case 0: If there are no candidate peers, return `None`.
        if candidate_locators.is_empty() {
            trace!("Found no sync peers with height greater {current_height}");
            return None;
        }

        // TODO (howardwu): Change this to the highest cumulative weight for Phase 3.
        // Case 1: If all of the candidate peers share a common ancestor below the latest ledger height,
        // then pick the peer with the highest height, and find peers (up to extra redundancy) with
        // a common ancestor above the block request range. Set the end height to their common ancestor.

        // Determine the threshold number of peers to sync from.
        let threshold_to_request = candidate_locators.len().min(REDUNDANCY_FACTOR);

        // Breaks the loop when the first threshold number of peers are found, biasing for the peer with the highest height
        // and a cohort of peers who share a common ancestor above this node's latest ledger height.
        for (idx, (peer_ip, peer_locators)) in candidate_locators.iter().enumerate() {
            // The height of the common ancestor shared by all selected peers.
            let mut min_common_ancestor = peer_locators.end_height();

            // The peers we will synchronize from.
            // As the previous iteration did not succeed, restart with the next candidate peers.
            let mut sync_peers = vec![(*peer_ip, peer_locators.clone())];

            // Try adding other peers consistent with this one to the sync peer set.
            for (other_ip, other_locators) in candidate_locators.iter().skip(idx + 1) {
                // Check if these two peers have a common ancestor above the latest ledger height.
                if let Some(common_ancestor) = self.common_ancestors.read().get(&PeerPair(*peer_ip, *other_ip)) {
                    // If so, then check that their block locators are consistent.
                    if *common_ancestor > latest_ledger_height && peer_locators.is_consistent_with(other_locators) {
                        // If their common ancestor is less than the minimum common ancestor, then update it.
                        min_common_ancestor = min_common_ancestor.min(*common_ancestor);

                        // Add the other peer to the list of sync peers.
                        sync_peers.push((*other_ip, other_locators.clone()));
                    }
                }
            }

            // If we have enough sync peers above the latest ledger height, finish and return them.
            if min_common_ancestor > latest_ledger_height && sync_peers.len() >= threshold_to_request {
                // Shuffle the sync peers prior to returning. This ensures the rest of the stack
                // does not rely on the order of the sync peers, and that the sync peers are not biased.
                sync_peers.shuffle(&mut rand::thread_rng());

                // Collect into an IndexMap and return.
                return Some((sync_peers.into_iter().collect(), min_common_ancestor));
            }
        }

        // If there is not enough peers with a minimum common ancestor above the latest ledger height, return None.
        None
    }

    /// Given the sync peers and their minimum common ancestor, return a list of block requests.
    #[allow(dead_code)]
    fn construct_requests(
        &self,
        sync_peers: &IndexMap<SocketAddr, BlockLocators<N>>,
        sync_height: u32,
        min_common_ancestor: u32,
        max_blocks_to_request: u32,
        greatest_peer_height: u32,
    ) -> Vec<(u32, PrepareSyncRequest<N>)> {
        // Compute the start height for the block requests.
        let start_height = {
            let requests = self.requests.read();
            let ledger_height = self.ledger.latest_block_height();

            // Do not issue requests for blocks already contained in the ledger.
            let mut start_height = ledger_height.max(sync_height + 1);

            // Do not issue requests that already exist.
            while requests.contains_key(&start_height) {
                start_height += 1;
            }

            start_height
        };

        // If the minimum common ancestor is below the start height, then return early.
        if min_common_ancestor < start_height {
            if start_height < greatest_peer_height {
                trace!(
                    "No request to construct. Height for the next block request is {start_height}, but minimum common block locator ancestor is only {min_common_ancestor} (sync_height={sync_height} greatest_peer_height={greatest_peer_height})"
                );
            }
            return Default::default();
        }

        // Compute the end height for the block request.
        let end_height = (min_common_ancestor + 1).min(start_height + max_blocks_to_request);

        // Construct the block hashes to request.
        let mut request_hashes = IndexMap::with_capacity((start_height..end_height).len());
        // Track the largest number of sync IPs required for any block request in the sequence of requests.
        let mut max_num_sync_ips = 1;

        for height in start_height..end_height {
            // Ensure the current height is not in the ledger or already requested.
            if let Err(err) = self.check_block_request(height) {
                trace!("{err}");

                // If the sequence of block requests is interrupted, then return early.
                // Otherwise, continue until the first start height that is new.
                match request_hashes.is_empty() {
                    true => continue,
                    false => break,
                }
            }

            // Construct the block request.
            let (hash, previous_hash, num_sync_ips, is_honest) = construct_request(height, sync_peers);

            // Handle the dishonest case.
            if !is_honest {
                // TODO (howardwu): Consider performing an integrity check on peers (to disconnect).
                warn!("Detected dishonest peer(s) when preparing block request");
                // If there are not enough peers in the dishonest case, then return early.
                if sync_peers.len() < num_sync_ips {
                    break;
                }
            }

            // Update the maximum number of sync IPs.
            max_num_sync_ips = max_num_sync_ips.max(num_sync_ips);

            // Append the request.
            request_hashes.insert(height, (hash, previous_hash));
        }

        // Construct the requests with the same sync ips.
        request_hashes
            .into_iter()
            .map(|(height, (hash, previous_hash))| (height, (hash, previous_hash, max_num_sync_ips)))
            .collect()
    }
}

/// If any peer is detected to be dishonest in this function, it will not set the hash or previous hash,
/// in order to allow the caller to determine what to do.
#[allow(dead_code)]
fn construct_request<N: Network>(
    height: u32,
    sync_peers: &IndexMap<SocketAddr, BlockLocators<N>>,
) -> (Option<N::BlockHash>, Option<N::BlockHash>, usize, bool) {
    let mut hash = None;
    let mut hash_redundancy: usize = 0;
    let mut previous_hash = None;
    let mut is_honest = true;

    for peer_locators in sync_peers.values() {
        if let Some(candidate_hash) = peer_locators.get_hash(height) {
            match hash {
                // Increment the redundancy count if the hash matches.
                Some(hash) if hash == candidate_hash => hash_redundancy += 1,
                // Some peer is dishonest.
                Some(_) => {
                    hash = None;
                    hash_redundancy = 0;
                    previous_hash = None;
                    is_honest = false;
                    break;
                }
                // Set the hash if it is not set.
                None => {
                    hash = Some(candidate_hash);
                    hash_redundancy = 1;
                }
            }
        }
        if let Some(candidate_previous_hash) = peer_locators.get_hash(height.saturating_sub(1)) {
            match previous_hash {
                // Increment the redundancy count if the previous hash matches.
                Some(previous_hash) if previous_hash == candidate_previous_hash => (),
                // Some peer is dishonest.
                Some(_) => {
                    hash = None;
                    hash_redundancy = 0;
                    previous_hash = None;
                    is_honest = false;
                    break;
                }
                // Set the previous hash if it is not set.
                None => previous_hash = Some(candidate_previous_hash),
            }
        }
    }

    // Note that we intentionally do not just pick the peers that have the hash we have chosen,
    // to give stronger confidence that we are syncing during times when the network is consistent/stable.
    let num_sync_ips = {
        // Extra redundant peers - as the block hash was dishonest.
        if !is_honest {
            // Choose up to the extra redundancy factor in sync peers.
            EXTRA_REDUNDANCY_FACTOR
        }
        // No redundant peers - as we have redundancy on the block hash.
        else if hash.is_some() && hash_redundancy >= REDUNDANCY_FACTOR {
            // Choose one sync peer.
            1
        }
        // Redundant peers - as we do not have redundancy on the block hash.
        else {
            // Choose up to the redundancy factor in sync peers.
            REDUNDANCY_FACTOR
        }
    };

    (hash, previous_hash, num_sync_ips, is_honest)
}

#[cfg(any(feature = "test-helpers", test))]
pub mod test_helpers;

#[cfg(test)]
mod tests;
