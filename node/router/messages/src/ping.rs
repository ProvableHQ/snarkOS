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

use super::*;

use snarkos_node_network::NodeType;
use snarkos_node_sync_locators::{MAX_CHECKPOINTS, NUM_RECENT_BLOCKS};
use snarkvm::prelude::{FromBytes, ToBytes};

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ping<N: Network> {
    pub version: u32,
    pub node_type: NodeType,
    pub block_locators: Option<BlockLocators<N>>,
}

/// The maximum size of a `Ping` frame, kept next to `write_le` below since that is what actually
/// defines the fixed part of this layout (4 + 1 + 1 bytes, plus the message ID that
/// `Message::write_le` prepends). Unlike every other message type, `Ping` has no fixed shape:
/// `block_locators` grows with the height of the chain, up to `NUM_RECENT_BLOCKS` recent entries
/// plus `MAX_CHECKPOINTS` checkpoints (see `snarkos_node_sync_locators::block_locators`), each a
/// `(u32, BlockHash)` pair - 4 bytes plus a 32-byte field element.
///
/// This is `u32::MAX / CHECKPOINT_INTERVAL` checkpoints' worth - the wire format's own ceiling,
/// derived rather than measured, and asserted below against the real locator-entry size so it
/// can't silently drift from what `BlockLocators` actually enforces. It is far above anything a
/// realistic chain height produces today; see the PR that introduced this cap for the case that
/// a tighter, revisitable number might be preferable instead.
pub const MAX_PING_MESSAGE_SIZE: usize = 16 * 1024 * 1024; // 16 MiB

const _: () = {
    let locator_entry = size_of::<u32>() + 32; // a block height, plus a `BlockHash` field element
    let ping = 2 // the message ID
        + 4 // `version`
        + 1 // `node_type`
        + 1 // the `Some`/`None` marker on `block_locators`
        + 4 // the `recents` map's length prefix
        + 4 // the `checkpoints` map's length prefix
        + (NUM_RECENT_BLOCKS + MAX_CHECKPOINTS) * locator_entry;
    assert!(ping <= MAX_PING_MESSAGE_SIZE, "MAX_PING_MESSAGE_SIZE is below the largest Ping the wire format permits");
};

impl<N: Network> MessageTrait for Ping<N> {
    /// Returns the message name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        "Ping".into()
    }
}

impl<N: Network> ToBytes for Ping<N> {
    fn write_le<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.version.write_le(&mut writer)?;
        self.node_type.write_le(&mut writer)?;
        if let Some(locators) = &self.block_locators {
            1u8.write_le(&mut writer)?;
            locators.write_le(&mut writer)?;
        } else {
            0u8.write_le(&mut writer)?;
        }

        Ok(())
    }
}

impl<N: Network> FromBytes for Ping<N> {
    fn read_le<R: io::Read>(mut reader: R) -> io::Result<Self> {
        let version = u32::read_le(&mut reader)?;
        let node_type = NodeType::read_le(&mut reader)?;

        let selector = u8::read_le(&mut reader)?;
        let block_locators = match selector {
            0 => None,
            1 => Some(BlockLocators::read_le(&mut reader)?),
            _ => return Err(error("Invalid block locators marker")),
        };

        Ok(Self { version, node_type, block_locators })
    }
}

impl<N: Network> Ping<N> {
    pub fn new(node_type: NodeType, block_locators: Option<BlockLocators<N>>) -> Self {
        Self { version: <Message<N>>::latest_message_version(), node_type, block_locators }
    }
}

#[cfg(test)]
pub mod prop_tests {
    use crate::{Ping, challenge_request::prop_tests::any_node_type};
    use snarkos_node_sync_locators::{
        BlockLocators,
        CHECKPOINT_INTERVAL,
        MAX_CHECKPOINTS,
        NUM_RECENT_BLOCKS,
        test_helpers::sample_block_locators,
    };
    use snarkvm::{
        prelude::Field,
        utilities::{FromBytes, ToBytes},
    };

    use bytes::{Buf, BufMut, BytesMut};
    use indexmap::IndexMap;
    use proptest::prelude::{BoxedStrategy, Strategy, any};
    use test_strategy::proptest;

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    pub fn any_block_locators() -> BoxedStrategy<BlockLocators<CurrentNetwork>> {
        any::<u32>().prop_map(sample_block_locators).boxed()
    }

    pub fn any_ping() -> BoxedStrategy<Ping<CurrentNetwork>> {
        (any::<u32>(), any_block_locators(), any_node_type())
            .prop_map(|(version, bls, node_type)| Ping { version, block_locators: Some(bls), node_type })
            .boxed()
    }

    #[proptest]
    fn ping_roundtrip(#[strategy(any_ping())] ping: Ping<CurrentNetwork>) {
        let mut bytes = BytesMut::default().writer();
        ping.write_le(&mut bytes).unwrap();
        let decoded = Ping::<CurrentNetwork>::read_le(&mut bytes.into_inner().reader()).unwrap();
        assert_eq!(ping, decoded);
    }

    /// The largest `Ping` the wire format permits: a full recent-block window, plus the
    /// checkpoint ceiling `BlockLocators::read_le` itself enforces - not just the largest number
    /// of entries `write_le` will emit, but one that actually satisfies `BlockLocators::new`'s own
    /// validity rules, since `read_le` calls it and this needs to round-trip through the codec.
    /// That means checkpoints genuinely spaced `CHECKPOINT_INTERVAL` apart (not just sequential
    /// heights), with `recents` ending within one interval of the last checkpoint - so this is
    /// only reachable at a chain height near `u32::MAX`, which is exactly the point.
    ///
    /// Content doesn't matter here beyond satisfying `BlockLocators::new`'s uniqueness
    /// requirement (`recents` and, separately, `checkpoints` may not contain a repeated hash) -
    /// so each entry's hash is just its own height embedded as a field element, which is cheap
    /// and trivially unique within each map, rather than paying for hundreds of thousands of
    /// fresh random field elements. Uniqueness is not required *across* the two maps, only
    /// within each, and this construction never gives them an overlapping height anyway.
    ///
    /// Note this is `MAX_CHECKPOINTS`, not `MAX_CHECKPOINTS + 1`: the latter isn't a way to get a
    /// frame that's too large for `MAX_PING_MESSAGE_SIZE` (there's real headroom between the two;
    /// `MAX_PING_MESSAGE_SIZE`'s own doc comment has the arithmetic), it's a way to get a frame
    /// that `BlockLocators::read_le` rejects for an unrelated reason - a different mechanism than
    /// the one this cap is testing.
    pub fn largest_possible_ping() -> Ping<CurrentNetwork> {
        let hash_for = |height: u32| Field::<CurrentNetwork>::from_u64(height as u64).into();

        let last_checkpoint_height = (MAX_CHECKPOINTS as u32 - 1) * CHECKPOINT_INTERVAL;
        let checkpoints =
            (0..MAX_CHECKPOINTS as u32).map(|i| (i * CHECKPOINT_INTERVAL, hash_for(i))).collect::<IndexMap<_, _>>();

        // `recents` must end within [last_checkpoint_height, last_checkpoint_height +
        // CHECKPOINT_INTERVAL) - put its window at the very top of that range.
        let last_recent_height = last_checkpoint_height + CHECKPOINT_INTERVAL - 1;
        let recents = ((last_recent_height + 1 - NUM_RECENT_BLOCKS as u32)..=last_recent_height)
            .map(|h| (h, hash_for(h)))
            .collect::<IndexMap<_, _>>();

        Ping {
            version: 0,
            node_type: snarkos_node_network::NodeType::Client,
            block_locators: Some(BlockLocators { recents, checkpoints }),
        }
    }
}
