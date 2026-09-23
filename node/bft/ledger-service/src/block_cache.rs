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

use snarkvm::{ledger::Block, prelude::Network};

use bytes::Bytes;
#[cfg(feature = "locktick")]
use locktick::parking_lot::Mutex;
use lru::LruCache;
#[cfg(not(feature = "locktick"))]
use parking_lot::Mutex;

use std::{num::NonZeroUsize, sync::Arc};

/// Pretty JSON of recent blocks, keyed by block hash.
///
/// `advance_to_next_block` inserts a block. The REST API only reads the cache.
pub struct BlockCache<N: Network> {
    blocks: Mutex<LruCache<N::BlockHash, Bytes>>,
}

impl<N: Network> Default for BlockCache<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N: Network> BlockCache<N> {
    /// Number of blocks retained.
    pub const CAPACITY: usize = 128;

    /// An empty cache.
    pub fn new() -> Self {
        Self { blocks: Mutex::new(LruCache::new(NonZeroUsize::new(Self::CAPACITY).expect("capacity is non-zero"))) }
    }

    /// An empty cache that already holds `block`.
    pub fn with_block(block: &Block<N>) -> Arc<Self> {
        let cache = Arc::new(Self::new());
        cache.insert(block);
        cache
    }

    /// Stores the pretty JSON of `block`.
    ///
    /// Serialization runs on the caller. `LedgerUpdate::advance_to_next_block` calls this while
    /// that update still holds the ledger update lock.
    pub fn insert(&self, block: &Block<N>) {
        let hash = block.hash();
        match serde_json::to_vec_pretty(block) {
            Ok(json) => self.insert_json(hash, json),
            Err(error) => {
                tracing::warn!("Failed to serialize block {hash} for the REST block cache: {error}");
            }
        }
    }

    /// Stores caller-supplied pretty JSON for `hash`.
    pub fn insert_json(&self, hash: N::BlockHash, json: impl Into<Bytes>) {
        self.blocks.lock().put(hash, json.into());
    }

    /// Pretty JSON previously stored for `hash`.
    pub fn get(&self, hash: &N::BlockHash) -> Option<Bytes> {
        self.blocks.lock().get(hash).cloned()
    }

    /// The cached JSON for each hash, in order. A missing hash is `None`.
    pub fn get_each(&self, hashes: &[N::BlockHash]) -> Vec<Option<Bytes>> {
        let mut blocks = self.blocks.lock();
        hashes.iter().map(|hash| blocks.get(hash).cloned()).collect()
    }

    /// `true` when `hash` is present.
    pub fn contains(&self, hash: &N::BlockHash) -> bool {
        self.blocks.lock().contains(hash)
    }
}
