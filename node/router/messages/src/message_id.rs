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

use crate::{MAXIMUM_MESSAGE_SIZE, MESSAGE_ID_SIZE};

use snarkos_node_sync_locators::{CHECKPOINT_INTERVAL, NUM_RECENT_BLOCKS};
use snarkvm::prelude::Network;

/// The wire ID of a [`crate::Message`] variant: the `u16` that leads every message payload.
///
/// This exists so that the codec can decide how large a message is allowed to be while holding
/// nothing but the ID - i.e. before the body has been read off the socket. Keeping the IDs in a
/// type rather than as bare integers is what makes that decision total: both
/// [`crate::Message::id`] and [`MessageId::max_size`] are written as `match`es without a fallback
/// arm, so a new `Message` variant that has no ID, or a new ID that has no size, does not compile.
///
/// The one direction that cannot be enforced by the compiler is [`MessageId::from_u16`], since a
/// `u16` can hold values no variant claims; `message_id_round_trip` covers it instead.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u16)]
pub enum MessageId {
    BlockRequest = 0,
    BlockResponse = 1,
    ChallengeRequest = 2,
    ChallengeResponse = 3,
    Disconnect = 4,
    PeerRequest = 5,
    PeerResponse = 6,
    Ping = 7,
    Pong = 8,
    PuzzleRequest = 9,
    PuzzleResponse = 10,
    UnconfirmedSolution = 11,
    UnconfirmedTransaction = 12,
}

/// The maximum size of the small, fixed-shape messages - everything except `Ping`,
/// `BlockResponse` and `UnconfirmedTransaction`. None of these carry more than a handful of
/// fixed-size fields, or (for `PeerResponse`) a peer list bounded by the `u8` count that
/// `PeerResponse::write_le` enforces. The `PEER_RESPONSE_WORST_CASE` assertion below pins the
/// largest of them against this value.
pub const MAX_SMALL_MESSAGE_SIZE: usize = 8 * 1024; // 8 KiB

/// The maximum size of a `Ping` message.
///
/// Unlike the messages above, a `Ping` does not have a fixed shape: its `block_locators` grow
/// with the height of the chain, up to `NUM_RECENT_BLOCKS` recent entries plus the
/// `BlockLocators` deserializer's own ceiling of `u32::MAX / CHECKPOINT_INTERVAL` checkpoints.
/// The `PING_WORST_CASE` assertion below is the arithmetic for that bound, so this constant
/// cannot silently drift below the largest `Ping` the protocol permits.
///
/// Note this is far above what any realistic chain height produces (a few tens of KiB); it is the
/// ceiling the wire format allows, not a target. It is not the memory an unfinished frame costs -
/// the decoder no longer pre-allocates the declared length - so an attacker has to actually
/// transmit these bytes to make a node hold them.
pub const MAX_PING_MESSAGE_SIZE: usize = 16 * 1024 * 1024; // 16 MiB

/// The bytes an `UnconfirmedTransaction` frame carries in addition to the transaction itself.
///
/// This has to be added to the transaction cap, rather than the cap being applied to the frame
/// directly: a transaction of exactly `LATEST_MAX_TRANSACTION_SIZE` is valid - both the ledger
/// service and the REST endpoint accept it - so the message carrying it has to fit on the wire,
/// and that message is necessarily larger than the transaction it carries.
///
/// The figures below are what the encoder actually emits; `max_size_unconfirmed_transaction_is_accepted`
/// pins them, so a change to any of these encodings fails the build rather than silently
/// shrinking the transaction size the network will relay.
pub const UNCONFIRMED_TRANSACTION_OVERHEAD: usize = MESSAGE_ID_SIZE
    + 32 // the transaction ID preceding the transaction body
    + 5; // `Data`'s own variant tag and length prefix

// The caps above are only useful if they are above what an honest node actually sends. For the
// two messages whose worst case is not obviously tiny, that arithmetic is written out here so the
// build fails rather than the network quietly starting to disconnect honest peers. The
// `largest_honest_peer_response_is_accepted` and `largest_possible_ping_is_accepted` tests check
// the same bounds against what the serializer really produces.
const _: () = {
    // The largest `PeerResponse` that `PeerResponse::write_le` will emit: a peer count of
    // `u8::MAX` - which that function enforces - with every peer an IPv6 address carrying a
    // height.
    let peer_response = MESSAGE_ID_SIZE
        + 1 // the version indicator
        + 1 // the peer count
        + (u8::MAX as usize)
            * (1     // the IPv6 tag
                + 16 // the address
                + 2  // the port
                + 1  // the `Some` tag on the height
                + 4); // the height
    assert!(
        peer_response <= MAX_SMALL_MESSAGE_SIZE,
        "MAX_SMALL_MESSAGE_SIZE is below the largest PeerResponse an honest node will send"
    );

    // The largest `Ping` the wire format permits: a full recent-block window, plus the ceiling on
    // checkpoints that `BlockLocators::read_le` enforces, each entry a `(u32, BlockHash)` pair.
    let locator_entry = size_of::<u32>() + 32;
    let ping = MESSAGE_ID_SIZE
        + 4 // the version
        + 1 // the node type
        + 1 // the `Some` tag on the block locators
        + 4 // the recents map length
        + 4 // the checkpoints map length
        + (NUM_RECENT_BLOCKS + (u32::MAX / CHECKPOINT_INTERVAL) as usize) * locator_entry;
    assert!(ping <= MAX_PING_MESSAGE_SIZE, "MAX_PING_MESSAGE_SIZE is below the largest Ping the wire format permits");
};

impl MessageId {
    /// Every message ID, for tests that need to enumerate them.
    #[cfg(test)]
    pub(crate) const ALL: [Self; 13] = [
        Self::BlockRequest,
        Self::BlockResponse,
        Self::ChallengeRequest,
        Self::ChallengeResponse,
        Self::Disconnect,
        Self::PeerRequest,
        Self::PeerResponse,
        Self::Ping,
        Self::Pong,
        Self::PuzzleRequest,
        Self::PuzzleResponse,
        Self::UnconfirmedSolution,
        Self::UnconfirmedTransaction,
    ];

    /// Returns the ID as it appears on the wire.
    pub const fn as_u16(self) -> u16 {
        self as u16
    }

    /// Returns the ID for the given wire value, or `None` if no message this node understands
    /// uses it. Callers are expected to treat `None` as a rejection: `Message::read_le` has never
    /// been able to deserialize such a payload either.
    pub const fn from_u16(id: u16) -> Option<Self> {
        Some(match id {
            0 => Self::BlockRequest,
            1 => Self::BlockResponse,
            2 => Self::ChallengeRequest,
            3 => Self::ChallengeResponse,
            4 => Self::Disconnect,
            5 => Self::PeerRequest,
            6 => Self::PeerResponse,
            7 => Self::Ping,
            8 => Self::Pong,
            9 => Self::PuzzleRequest,
            10 => Self::PuzzleResponse,
            11 => Self::UnconfirmedSolution,
            12 => Self::UnconfirmedTransaction,
            13.. => return None,
        })
    }

    /// Returns the largest frame, in bytes, that a message with this ID may legitimately be.
    ///
    /// This is checked against the declared frame length before the body is read off the wire,
    /// so it bounds work the node does on behalf of a peer that has sent only six bytes. The size
    /// is of the whole frame - the message ID included - because that is what the length prefix
    /// counts.
    pub fn max_size<N: Network>(self) -> usize {
        match self {
            // `BlockResponse` is the one message that is legitimately large - up to a handful of
            // full blocks, bounded by block count rather than by byte size. It is deliberately
            // left at the general frame ceiling; tightening it needs the requester's own
            // pending-request state, which this framing layer does not have.
            Self::BlockResponse => MAXIMUM_MESSAGE_SIZE,
            Self::Ping => MAX_PING_MESSAGE_SIZE,
            // `UnconfirmedTransaction` is the one message type with a network-defined,
            // consensus-version-dependent cap. The envelope has to be added to it: a transaction
            // of exactly `LATEST_MAX_TRANSACTION_SIZE` is valid, and the message carrying it is
            // necessarily larger than the transaction itself.
            Self::UnconfirmedTransaction => {
                N::LATEST_MAX_TRANSACTION_SIZE().saturating_add(UNCONFIRMED_TRANSACTION_OVERHEAD)
            }
            Self::BlockRequest
            | Self::ChallengeRequest
            | Self::ChallengeResponse
            | Self::Disconnect
            | Self::PeerRequest
            | Self::PeerResponse
            | Self::Pong
            | Self::PuzzleRequest
            | Self::PuzzleResponse
            | Self::UnconfirmedSolution => MAX_SMALL_MESSAGE_SIZE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `from_u16` is the one mapping the compiler cannot check, so check it here.
    #[test]
    fn message_id_round_trip() {
        for id in MessageId::ALL {
            assert_eq!(MessageId::from_u16(id.as_u16()), Some(id), "{id:?} does not round-trip through its wire value");
        }
    }

    /// Every ID is distinct, and they are the contiguous range the wire format assumes.
    #[test]
    fn message_ids_are_contiguous_from_zero() {
        for (index, id) in MessageId::ALL.iter().enumerate() {
            assert_eq!(id.as_u16() as usize, index, "{id:?} is not at its expected wire value");
        }
        assert_eq!(MessageId::from_u16(MessageId::ALL.len() as u16), None, "an unclaimed ID was accepted");
    }

    /// Every ID has a size that is at least large enough to hold the ID itself, and no capped
    /// message may exceed the general frame ceiling.
    #[test]
    fn max_sizes_are_sane() {
        type CurrentNetwork = snarkvm::prelude::MainnetV0;

        for id in MessageId::ALL {
            let max_size = id.max_size::<CurrentNetwork>();
            assert!(max_size >= MESSAGE_ID_SIZE, "{id:?} cannot hold its own message ID");
            assert!(max_size <= MAXIMUM_MESSAGE_SIZE, "{id:?} is allowed to exceed the general frame ceiling");
        }
    }
}
