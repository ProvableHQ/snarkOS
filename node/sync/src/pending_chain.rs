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

use snarkvm::{ledger::PendingBlock, prelude::Network};

use std::collections::HashSet;

/// Represents a chain of not-yet-confirmed blocks.
#[derive(Clone)]
pub(crate) struct PendingChain<N: Network> {
    blocks: Vec<PendingBlock<N>>,
    block_hashes: HashSet<N::BlockHash>,
}

impl<N: Network> PartialEq for PendingChain<N> {
    fn eq(&self, other: &PendingChain<N>) -> bool {
        if self.len() != other.len() {
            return false;
        }

        for idx in 0..self.len() {
            if self.blocks[idx] != other.blocks[idx] {
                return false;
            }
        }

        true
    }
}

impl<N: Network> PendingChain<N> {
    pub fn new() -> Self {
        Self { blocks: Default::default(), block_hashes: Default::default() }
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn head(&self) -> Option<&PendingBlock<N>> {
        self.blocks.get(self.len().saturating_sub(1))
    }

    /// Returns the height of the head of the chain (if any)
    pub fn current_height(&self) -> Option<u32> {
        self.head().map(|b| b.height())
    }

    /// Get the pending blocks (ordered by height)
    pub fn blocks(&self) -> &[PendingBlock<N>] {
        &self.blocks
    }

    /// Returns true if this pending chain contain the specified block hash.
    pub fn contains(&self, block_hash: &N::BlockHash) -> bool {
        self.block_hashes.contains(block_hash)
    }

    /// Append a new block to this chain
    #[must_use]
    pub fn append(&mut self, block: PendingBlock<N>) -> bool {
        let compatible = match self.current_height() {
            Some(h) => h + 1 == block.height(),
            None => true,
        };

        if compatible {
            self.block_hashes.insert(block.hash());
            self.blocks.push(block);
        }

        compatible
    }

    /// Remove all blocks up to the given height.
    pub fn truncate_prefix(&mut self, height: u32) {
        self.blocks.retain(|b| b.height() > height);
    }

    /// Create a copy of this chain up to the given height.
    pub fn fork_at(&self, height: u32) -> Self {
        let mut fork = Self::new();
        for block in &self.blocks {
            if block.height() >= height {
                return fork;
            }

            let _ = fork.append(block.clone());
        }
        fork
    }
}
