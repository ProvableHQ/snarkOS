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

use super::MAX_BLOCKS_BEHIND;

use std::{cmp::Ordering, time::Instant};

#[derive(Clone)]
pub(super) struct SyncState {
    /// The height we synced to already
    /// Note: This can be greater than the current ledger height,
    ///       if blocks are not fully committed yet
    sync_height: u32,
    /// The largest height of a peer's block locator.
    /// Is `None` if we never received a peer loator
    greatest_peer_height: Option<u32>,
    /// Are we synced?
    /// Allows keeping track of when the sync state changes.
    is_synced: bool,
    /// Last time the sync state changed
    last_change: Instant,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            sync_height: 0,
            greatest_peer_height: None,
            // Start as "synced" by default. Otherwise, validators will never propose certificates.
            is_synced: true,
            last_change: Instant::now(),
        }
    }
}

impl SyncState {
    /// Initialize the sync state at the given height.
    /// Useful, when starting a node that already has blocks in its local storage.
    pub fn new_with_height(height: u32) -> Self {
        Self { sync_height: height, ..Default::default() }
    }

    /// Did we catch up with the greatest known peer height?
    /// This will return false if we never synced from a peer.
    pub fn is_block_synced(&self) -> bool {
        self.is_synced
    }

    /// Returns `true` if there a blocks to sync from other nodes.
    /// Returns `false` if the node has fully caught up with the rest of the network.
    pub fn can_block_sync(&self) -> bool {
        // Return true if sync state is false even if we there are no known blocks to fetch,
        // because otherwise nodes will never  switch to synced at startup.
        if let Some(num_behind) = self.num_blocks_behind() {
            num_behind > 0
        } else {
            debug!("Cannot block sync: the node has not received block locators yet");
            false
        }
    }

    /// Returns the sync height (this is always greater or equal than the ledger height).
    pub fn get_sync_height(&self) -> u32 {
        self.sync_height
    }

    // Compute the number of blocks that we are behind by.
    // Returns None, if there is no known peer height.
    pub fn num_blocks_behind(&self) -> Option<u32> {
        self.greatest_peer_height.map(|peer_height| peer_height.saturating_sub(self.sync_height))
    }

    /// Returns the greatest block height of any connected peer.
    pub fn get_greatest_peer_height(&self) -> Option<u32> {
        self.greatest_peer_height
    }

    /// Update the height we are synced to.
    /// If the value is lower than the current height, the sync height remains unchanged.
    pub fn set_sync_height(&mut self, sync_height: u32) {
        if sync_height <= self.sync_height {
            return;
        }

        trace!("Sync height increased from {old_height} to {sync_height}", old_height = self.sync_height);
        self.sync_height = sync_height;
        self.update_is_block_synced();
    }

    /// Update the greatest known height of a connected peer.
    pub fn set_greatest_peer_height(&mut self, peer_height: u32) {
        if let Some(old_height) = self.greatest_peer_height {
            match old_height.cmp(&peer_height) {
                Ordering::Equal => return,
                Ordering::Greater => warn!("Greatest peer height reduced from {old_height} to {peer_height}"),
                Ordering::Less => trace!("Greatest peer height increased from {old_height} to {peer_height}"),
            }
        }

        self.greatest_peer_height = Some(peer_height);
        self.update_is_block_synced();
    }

    /// Updates the state of `is_block_synced` for the sync module.
    fn update_is_block_synced(&mut self) {
        trace!(
            "Updating is_block_synced: greatest_peer_height={greatest_peer:?}, current_height={current}, is_synced={is_synced}",
            greatest_peer = self.greatest_peer_height,
            current = self.sync_height,
            is_synced = self.is_synced,
        );

        let num_blocks_behind = self.num_blocks_behind();
        let old_sync_val = self.is_synced;

        // If there are no block locators, we consider ourselves synced.
        // Otherwise, validators will never propose certificates.
        let new_sync_val = num_blocks_behind.is_none_or(|num| num <= MAX_BLOCKS_BEHIND);

        // Print a message if the state changed
        if new_sync_val != old_sync_val {
            // Measure how long sync took.
            let now = Instant::now();
            let elapsed = now.saturating_duration_since(self.last_change).as_secs();
            self.last_change = now;

            if new_sync_val {
                let elapsed =
                    if elapsed < 60 { format!("{elapsed} seconds") } else { format!("{} minutes", elapsed / 60) };

                debug!("Block sync state changed to \"synced\". It took {elapsed} to catch up with the network.");
            } else {
                // num_blocks_behind should never be None at this point,
                // but we still use `unwrap_or` just in case.
                let behind_msg = num_blocks_behind.map(|n| n.to_string()).unwrap_or("unknown".to_string());

                debug!("Block sync state changed to \"syncing\". We are {behind_msg} blocks behind.");
            }
        }

        self.is_synced = new_sync_val;

        // Update the `IS_SYNCED` metric.
        #[cfg(feature = "metrics")]
        metrics::gauge(metrics::bft::IS_SYNCED, new_sync_val);
    }
}
