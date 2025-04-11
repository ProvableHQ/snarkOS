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

use snarkvm::prelude::Network;

use core::hash::Hash;
use std::{net::SocketAddr, time::Instant};

/// We can uniequely identify a block request by the sync peer and the starting height
pub type BlockRequestId = (SocketAddr, u32);

/// A block is uniquely identified by its
///
/// This is needed because peers can lie about hashes
/// So the same block hash could appear at different heights, and with different predecessors.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BlockId<N: Network> {
    /// The height of the block we are trying to fetch.
    pub height: u32,
    /// The hash of the parent of the block we are trying to fetch.
    pub previous_hash: N::BlockHash,
    // The hash of the block we are trying to fetch.
    pub hash: N::BlockHash,
}

/// Contains information about a pending sync request
#[derive(Debug, Clone)]
pub struct BlockSyncRequest<N: Network> {
    /// The sequence of blocks
    ///
    /// Invariant: blocks[0].height + blocks.len() = blocks.last().unwrap().height
    /// Invariant: this vector is never empty.
    pub blocks: Vec<BlockId<N>>,
    /// The peer we are trying to fetch the blocks from.
    pub sync_peer: SocketAddr,
    /// When was this request first created?
    pub timestamp: Instant,
}

impl<N: Network> BlockSyncRequest<N> {
    pub fn get_identifier(&self) -> BlockRequestId {
        (self.sync_peer, self.start_height())
    }

    /// The block hash this request "builds on",
    /// i.e., the hash of predecessor of the first block in this request.
    pub fn previous_block_hash(&self) -> Option<N::BlockHash> {
        self.blocks.first().map(|b| b.previous_hash)
    }

    /// The lowest block height in this request (inclusive).
    pub fn start_height(&self) -> u32 {
        self.blocks.first().map(|b| b.height).unwrap_or(0)
    }

    /// The greatest block height of this requests (exclusive).
    pub fn end_height(&self) -> u32 {
        self.blocks.last().map(|b| b.height + 1).unwrap_or(1)
    }
}
