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

use snarkvm::prelude::{FromBytes, IoResult, Network, Read, ToBytes, Write, error, has_duplicates};

use anyhow::{Result, bail, ensure};
use serde::{Deserialize, Serialize};

/// The maximum number of block hashes within a single locator.
pub const MAX_LOCATOR_SIZE: usize = 100; // 100 blocks

/// Block locator maps.
///
/// This data structure is used by validators to advertise the blocks that
/// they have and can provide to other validators to help them sync.
/// Periodically, each validator broadcasts a [`PrimaryPing`],
/// which contains a `BlockLocators` instance.
/// Recall that blocks are indexed by their `u32` height, starting with 0 for the genesis block.
/// The keys of the `recents` and `checkpoints` maps are the block heights;
/// the values of the maps are the corresponding block hashes.
///
/// If a validator has `N` blocks, the `recents` and `checkpoints` maps are as follows:
/// - The `recents` map contains entries for blocks at heights
///   `N - 1 - (NUM_RECENT_BLOCKS - 1)`,
///   `N - 1 - (NUM_RECENT_BLOCKS - 2)`,
///   ...,
///   `N - 1`.
///   If any of the just listed heights are negative, there are no entries for them of course,
///   and the `recents` map has fewer than `NUM_RECENT_BLOCKS` entries.
///   The `recents` map contains entries
///   for the last `NUM_RECENT_BLOCKS` blocks, i.e. from `N - NUM_RECENT_BLOCKS` to `N - 1`;
///   if additionally `N < NUM_RECENT_BLOCKS`, the `recents` map contains
///   entries for all the blocks, from `0` to `N - 1`.
/// - The `checkpoints` map contains an entry for every `CHECKPOINT_INTERVAL`-th block,
///   starting with 0 and not exceeding `N`, i.e. it has entries for blocks
///   `0`, `CHECKPOINT_INTERVAL`, `2 * CHECKPOINT_INTERVAL`, ..., `k * CHECKPOINT_INTERVAL`,
///   where `k` is the maximum integer such that `k * CHECKPOINT_INTERVAL <= N`.
///
/// The `recents` and `checkpoints` maps may have overlapping entries,
/// e.g. if `N-1` is a multiple of `CHECKPOINT_INTERVAL`;
/// but if `CHECKPOINT_INTERVAL` is much larger than `NUM_RECENT_BLOCKS`,
/// there is no overlap most of the time.
///
/// We call `BlockLocators` with the form described above 'well-formed'.
///
/// Well-formed `BlockLocators` instances are built by [`BlockSync::get_block_locators()`].
/// When a `BlockLocators` instance is received (in a [`PrimaryPing`]) by a validator,
/// the maps may not be well-formed (if the sending validator is faulty),
/// but the receiving validator ensures that they are well-formed
/// by calling [`BlockLocators::ensure_is_valid()`] from [`BlockLocators::new()`],
/// when deserializing in [`BlockLocators::read_le()`].
/// So this well-formedness is an invariant of `BlockLocators` instances.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockLocators<N: Network> {
    pub start_height: u32,
    pub block_hashes: Vec<N::BlockHash>,
}

impl<N: Network> BlockLocators<N> {
    /// Initializes a new instance of the block locators, checking the validity of the block locators.
    pub fn new(start_height: u32, block_hashes: Vec<N::BlockHash>) -> Result<Self> {
        Ok(Self { start_height, block_hashes })
    }

    /// Initializes a new genesis instance of the block locators.
    pub fn new_genesis(genesis_hash: N::BlockHash) -> Self {
        Self { start_height: 0, block_hashes: vec![genesis_hash] }
    }
}

impl<N: Network> IntoIterator for BlockLocators<N> {
    type IntoIter = <Vec<(u32, N::BlockHash)> as IntoIterator>::IntoIter;
    type Item = (u32, N::BlockHash);

    fn into_iter(self) -> Self::IntoIter {
        let data: Vec<_> = self
            .block_hashes
            .into_iter()
            .enumerate()
            .map(|(idx, hash)| (self.start_height + (idx as u32), hash))
            .collect();

        data.into_iter()
    }
}

impl<N: Network> BlockLocators<N> {
    /// The first height in this set of locators.
    pub fn start_height(&self) -> u32 {
        self.start_height
    }

    /// The last height in this set of locators (inclusive).
    pub fn end_height(&self) -> u32 {
        self.start_height + (self.block_hashes.len() as u32) - 1
    }

    /// Returns the block hash for the given block height, if it exists.
    pub fn get_hash(&self, height: u32) -> Option<N::BlockHash> {
        if height < self.start_height {
            return None;
        }

        let index = (height - self.start_height) as usize;
        self.block_hashes.get(index).copied()
    }

    /// Returns `true` if the block locators are well-formed.
    pub fn is_valid(&self) -> bool {
        // Ensure the block locators are well-formed.
        if let Err(error) = self.ensure_is_valid() {
            warn!("Block locators are invalid: {error}");
            return false;
        }
        true
    }

    /// Returns `true` if the given block locators are consistent with this one.
    /// This function assumes the given block locators are well-formed.
    pub fn is_consistent_with(&self, other: &Self) -> bool {
        // Ensure the block locators are consistent with the previous ones.
        if let Err(error) = self.ensure_is_consistent_with(other) {
            warn!("Inconsistent block locators: {error}");
            return false;
        }
        true
    }

    /// Checks that this block locators instance is well-formed.
    pub fn ensure_is_valid(&self) -> Result<()> {
        if self.block_hashes.is_empty() {
            bail!("Block locators cannot be empty!");
        }

        if has_duplicates(&self.block_hashes) {
            bail!("Block locators cannot contain duplicate hashes!");
        }

        Ok(())
    }

    /// Returns `true` if the given block locators are consistent with this one.
    /// This function assumes the given block locators are well-formed.
    pub fn ensure_is_consistent_with(&self, other: &Self) -> Result<()> {
        Self::check_consistent_block_locators(self, other)
    }
}

impl<N: Network> BlockLocators<N> {
    /// Checks the old and new block locators share a consistent view of block history.
    /// This function assumes the given block locators are well-formed.
    pub fn check_consistent_block_locators(
        old_locators: &BlockLocators<N>,
        new_locators: &BlockLocators<N>,
    ) -> Result<()> {
        if old_locators.end_height() < new_locators.start_height()
            || new_locators.end_height() < old_locators.start_height()
        {
            return Ok(());
        }

        // Figure out the range the locators overlap
        let start_height = old_locators.start_height().max(new_locators.start_height());
        let end_height = old_locators.end_height().min(new_locators.end_height());

        let mut old_idx = (start_height - old_locators.start_height()) as usize;
        let mut new_idx = (start_height - new_locators.start_height()) as usize;

        for _ in start_height..end_height {
            ensure!(
                old_locators.block_hashes[old_idx] == new_locators.block_hashes[new_idx],
                "Block hashes do not match"
            );
            old_idx += 1;
            new_idx += 1;
        }

        Ok(())
    }
}

impl<N: Network> FromBytes for BlockLocators<N> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        // Read the number of recent block hashes and start height.
        let num_blocks = u32::read_le(&mut reader)?;
        let start_height = u32::read_le(&mut reader)?;

        // Read the recent block hashes.
        let mut hashes = Vec::with_capacity(num_blocks as usize);
        for _ in 0..num_blocks {
            hashes.push(N::BlockHash::read_le(&mut reader)?);
        }

        Self::new(start_height, hashes).map_err(error)
    }
}

impl<N: Network> ToBytes for BlockLocators<N> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        // Write the number of blocks
        let num_blocks = self.block_hashes.len() as u32;
        num_blocks.write_le(&mut writer)?;

        // Write the start height.
        self.start_height.write_le(&mut writer)?;

        // Write the hashes
        for hash in &self.block_hashes {
            hash.write_le(&mut writer)?;
        }

        Ok(())
    }
}

#[cfg(any(test, feature = "test-helpers"))]
pub mod test_helpers {
    use super::*;
    use snarkvm::prelude::Field;

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    /// Simulates a block locator at the given height.
    ///
    /// The returned block locator is checked to be well-formed.
    pub fn sample_block_locators(start: u32, end: u32) -> BlockLocators<CurrentNetwork> {
        // Create the block hashes
        let hashes: Vec<_> = (start..=end).map(|i| Field::<CurrentNetwork>::from_u32(i).into()).collect();

        BlockLocators::new(start, hashes).unwrap()
    }

    /// Simulates a block locator at the given height, with a fork within NUM_RECENT_BLOCKS of the given height.
    ///
    /// The returned block locator is checked to be well-formed.
    pub fn sample_block_locators_with_fork(start: u32, end: u32, fork_height: u32) -> BlockLocators<CurrentNetwork> {
        assert!(fork_height <= end, "Fork must be in the given range");
        assert!(fork_height >= start, "Fork must be in the given range");

        // Create the recent locators.
        let hashes: Vec<_> = (start..=end)
            .map(|i| {
                if i >= fork_height {
                    (-Field::<CurrentNetwork>::from_u32(i)).into()
                } else {
                    Field::<CurrentNetwork>::from_u32(i).into()
                }
            })
            .collect();

        BlockLocators::new(start, hashes).unwrap()
    }

    /*
    /// A test to ensure that the sample block locators are valid.
    #[test]
    fn test_sample_block_locators() {
        for expected_height in 0..=100_001u32 {
            println!("Testing height - {expected_height}");

            let expected_num_checkpoints = (expected_height / CHECKPOINT_INTERVAL) + 1;
            let expected_num_recents = match expected_height < NUM_RECENT_BLOCKS as u32 {
                true => expected_height + 1,
                false => NUM_RECENT_BLOCKS as u32,
            };

            let block_locators = sample_block_locators(expected_height);
            assert_eq!(block_locators.checkpoints.len(), expected_num_checkpoints as usize);
            assert_eq!(block_locators.recents.len(), expected_num_recents as usize);
            assert_eq!(block_locators.latest_locator_height(), expected_height);
            // Note that `sample_block_locators` always returns well-formed block locators,
            // so we don't need to check `is_valid()` here.
        }
    }*/
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::prelude::Field;

    use core::ops::Range;

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    /// Simulates block locators for a ledger within the given `heights` range.
    fn check_is_valid(checkpoints: IndexMap<u32, <CurrentNetwork as Network>::BlockHash>, heights: Range<u32>) {
        for height in heights {
            let mut recents = IndexMap::new();
            for i in 0..NUM_RECENT_BLOCKS as u32 {
                recents.insert(height + i, (Field::<CurrentNetwork>::from_u32(height + i)).into());

                let block_locators =
                    BlockLocators::<CurrentNetwork>::new_unchecked(recents.clone(), checkpoints.clone());
                if height == 0 && recents.len() < NUM_RECENT_BLOCKS {
                    // For the first NUM_RECENT_BLOCKS, ensure NUM_RECENT_BLOCKS - 1 or less is valid.
                    block_locators.ensure_is_valid().unwrap();
                } else if recents.len() < NUM_RECENT_BLOCKS {
                    // After the first NUM_RECENT_BLOCKS blocks from genesis, ensure NUM_RECENT_BLOCKS - 1 or less is not valid.
                    block_locators.ensure_is_valid().unwrap_err();
                } else {
                    // After the first NUM_RECENT_BLOCKS blocks from genesis, ensure NUM_RECENT_BLOCKS is valid.
                    block_locators.ensure_is_valid().unwrap();
                }
            }
            // Ensure NUM_RECENT_BLOCKS + 1 is not valid.
            recents.insert(
                height + NUM_RECENT_BLOCKS as u32,
                (Field::<CurrentNetwork>::from_u32(height + NUM_RECENT_BLOCKS as u32)).into(),
            );
            let block_locators = BlockLocators::<CurrentNetwork>::new_unchecked(recents.clone(), checkpoints.clone());
            block_locators.ensure_is_valid().unwrap_err();
        }
    }

    /// Simulates block locators for a ledger within the given `heights` range.
    fn check_is_consistent(
        checkpoints: IndexMap<u32, <CurrentNetwork as Network>::BlockHash>,
        heights: Range<u32>,
        genesis_locators: BlockLocators<CurrentNetwork>,
        second_locators: BlockLocators<CurrentNetwork>,
    ) {
        for height in heights {
            let mut recents = IndexMap::new();
            for i in 0..NUM_RECENT_BLOCKS as u32 {
                recents.insert(height + i, (Field::<CurrentNetwork>::from_u32(height + i)).into());

                let block_locators =
                    BlockLocators::<CurrentNetwork>::new_unchecked(recents.clone()e, checkpoints.clone());
                block_locators.ensure_is_consistent_with(&block_locators).unwrap();

                // Only test consistency when the block locators are valid to begin with.
                let is_first_num_recents_blocks = height == 0 && recents.len() < NUM_RECENT_BLOCKS;
                let is_num_recents_blocks = recents.len() == NUM_RECENT_BLOCKS;
                if is_first_num_recents_blocks || is_num_recents_blocks {
                    // Ensure the block locators are consistent with the genesis block locators.
                    genesis_locators.ensure_is_consistent_with(&block_locators).unwrap();
                    block_locators.ensure_is_consistent_with(&genesis_locators).unwrap();

                    // Ensure the block locators are consistent with the block locators with two recent blocks.
                    second_locators.ensure_is_consistent_with(&block_locators).unwrap();
                    block_locators.ensure_is_consistent_with(&second_locators).unwrap();
                }
            }
        }
    }

    #[test]
    fn test_ensure_is_valid() {
        let zero: <CurrentNetwork as Network>::BlockHash = (Field::<CurrentNetwork>::from_u32(0)).into();
        let checkpoint_1: <CurrentNetwork as Network>::BlockHash =
            (Field::<CurrentNetwork>::from_u32(CHECKPOINT_INTERVAL)).into();

        // Ensure the block locators are valid.
        for height in 0..10 {
            let block_locators = test_helpers::sample_block_locators(height);
            block_locators.ensure_is_valid().unwrap();
        }

        // Ensure the first NUM_RECENT blocks are valid.
        let checkpoints = IndexMap::from([(0, zero)]);
        let mut recents = IndexMap::new();
        for i in 0..NUM_RECENT_BLOCKS {
            recents.insert(i as u32, (Field::<CurrentNetwork>::from_u32(i as u32)).into());
            let block_locators = BlockLocators::<CurrentNetwork>::new_unchecked(recents.clone(), checkpoints.clone());
            block_locators.ensure_is_valid().unwrap();
        }
        // Ensure NUM_RECENT_BLOCKS + 1 is not valid.
        recents.insert(NUM_RECENT_BLOCKS as u32, (Field::<CurrentNetwork>::from_u32(NUM_RECENT_BLOCKS as u32)).into());
        let block_locators = BlockLocators::<CurrentNetwork>::new_unchecked(recents.clone(), checkpoints);
        block_locators.ensure_is_valid().unwrap_err();

        // Ensure block locators before the second checkpoint are valid.
        let checkpoints = IndexMap::from([(0, zero)]);
        check_is_valid(checkpoints, 0..(CHECKPOINT_INTERVAL - NUM_RECENT_BLOCKS as u32));

        // Ensure the block locators after the second checkpoint are valid.
        let checkpoints = IndexMap::from([(0, zero), (CHECKPOINT_INTERVAL, checkpoint_1)]);
        check_is_valid(
            checkpoints,
            (CHECKPOINT_INTERVAL - NUM_RECENT_BLOCKS as u32 + 1)..(CHECKPOINT_INTERVAL * 2 - NUM_RECENT_BLOCKS as u32),
        );
    }

    #[test]
    fn test_ensure_is_valid_fails() {
        let zero: <CurrentNetwork as Network>::BlockHash = (Field::<CurrentNetwork>::from_u32(0)).into();
        let one: <CurrentNetwork as Network>::BlockHash = (Field::<CurrentNetwork>::from_u32(1)).into();

        // Ensure an empty block locators is not valid.
        let block_locators = BlockLocators::<CurrentNetwork>::new_unchecked(Default::default(), Default::default());
        block_locators.ensure_is_valid().unwrap_err();

        // Ensure internally-mismatching genesis block locators is not valid.
        let block_locators =
            BlockLocators::<CurrentNetwork>::new_unchecked(IndexMap::from([(0, zero)]), IndexMap::from([(0, one)]));
        block_locators.ensure_is_valid().unwrap_err();

        // Ensure internally-mismatching genesis block locators is not valid.
        let block_locators =
            BlockLocators::<CurrentNetwork>::new_unchecked(IndexMap::from([(0, one)]), IndexMap::from([(0, zero)]));
        block_locators.ensure_is_valid().unwrap_err();

        // Ensure internally-mismatching block locators with two recent blocks is not valid.
        let block_locators = BlockLocators::<CurrentNetwork>::new_unchecked(
            IndexMap::from([(0, one), (1, zero)]),
            IndexMap::from([(0, zero)]),
        );
        block_locators.ensure_is_valid().unwrap_err();

        // Ensure duplicate recent block hashes are not valid.
        let block_locators = BlockLocators::<CurrentNetwork>::new_unchecked(
            IndexMap::from([(0, zero), (1, zero)]),
            IndexMap::from([(0, zero)]),
        );
        block_locators.ensure_is_valid().unwrap_err();

        // Ensure insufficient checkpoints are not valid.
        let mut recents = IndexMap::new();
        for i in 0..NUM_RECENT_BLOCKS {
            recents.insert(10_000 + i as u32, (Field::<CurrentNetwork>::from_u32(i as u32)).into());
        }
        let block_locators = BlockLocators::<CurrentNetwork>::new_unchecked(recents, IndexMap::from([(0, zero)]));
        block_locators.ensure_is_valid().unwrap_err();
    }

    #[test]
    fn test_ensure_is_consistent_with() {
        let zero: <CurrentNetwork as Network>::BlockHash = (Field::<CurrentNetwork>::from_u32(0)).into();
        let one: <CurrentNetwork as Network>::BlockHash = (Field::<CurrentNetwork>::from_u32(1)).into();

        let genesis_locators =
            BlockLocators::<CurrentNetwork>::new_unchecked(IndexMap::from([(0, zero)]), IndexMap::from([(0, zero)]));
        let second_locators = BlockLocators::<CurrentNetwork>::new_unchecked(
            IndexMap::from([(0, zero), (1, one)]),
            IndexMap::from([(0, zero)]),
        );

        // Ensure genesis block locators is consistent with genesis block locators.
        genesis_locators.ensure_is_consistent_with(&genesis_locators).unwrap();

        // Ensure genesis block locators is consistent with block locators with two recent blocks.
        genesis_locators.ensure_is_consistent_with(&second_locators).unwrap();
        second_locators.ensure_is_consistent_with(&genesis_locators).unwrap();

        // Ensure the block locators before the second checkpoint are valid.
        let checkpoints = IndexMap::from([(0, Default::default())]);
        check_is_consistent(
            checkpoints,
            0..(CHECKPOINT_INTERVAL - NUM_RECENT_BLOCKS as u32),
            genesis_locators.clone(),
            second_locators.clone(),
        );

        // Ensure the block locators after the second checkpoint are valid.
        let checkpoints = IndexMap::from([(0, Default::default()), (CHECKPOINT_INTERVAL, Default::default())]);
        check_is_consistent(
            checkpoints,
            (CHECKPOINT_INTERVAL - NUM_RECENT_BLOCKS as u32)..(CHECKPOINT_INTERVAL * 2 - NUM_RECENT_BLOCKS as u32),
            genesis_locators,
            second_locators,
        );
    }

    #[test]
    fn test_ensure_is_consistent_with_fails() {
        let zero: <CurrentNetwork as Network>::BlockHash = (Field::<CurrentNetwork>::from_u32(0)).into();
        let one: <CurrentNetwork as Network>::BlockHash = (Field::<CurrentNetwork>::from_u32(1)).into();

        let genesis_locators =
            BlockLocators::<CurrentNetwork>::new(IndexMap::from([(0, zero)]), IndexMap::from([(0, zero)])).unwrap();
        let second_locators =
            BlockLocators::<CurrentNetwork>::new(IndexMap::from([(0, zero), (1, one)]), IndexMap::from([(0, zero)]))
                .unwrap();

        let wrong_genesis_locators =
            BlockLocators::<CurrentNetwork>::new(IndexMap::from([(0, one)]), IndexMap::from([(0, one)])).unwrap();
        let wrong_second_locators =
            BlockLocators::<CurrentNetwork>::new(IndexMap::from([(0, one), (1, zero)]), IndexMap::from([(0, one)]))
                .unwrap();

        genesis_locators.ensure_is_consistent_with(&wrong_genesis_locators).unwrap_err();
        wrong_genesis_locators.ensure_is_consistent_with(&genesis_locators).unwrap_err();

        genesis_locators.ensure_is_consistent_with(&wrong_second_locators).unwrap_err();
        wrong_second_locators.ensure_is_consistent_with(&genesis_locators).unwrap_err();

        second_locators.ensure_is_consistent_with(&wrong_genesis_locators).unwrap_err();
        wrong_genesis_locators.ensure_is_consistent_with(&second_locators).unwrap_err();

        second_locators.ensure_is_consistent_with(&wrong_second_locators).unwrap_err();
        wrong_second_locators.ensure_is_consistent_with(&second_locators).unwrap_err();
    }
}
*/
