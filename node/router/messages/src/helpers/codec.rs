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

use crate::Message;
use snarkvm::prelude::{FromBytes, Network, ToBytes};

use ::bytes::{Buf, BufMut, BytesMut};
use core::marker::PhantomData;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

/// The maximum size of a message that can be transmitted during the handshake.
const MAXIMUM_HANDSHAKE_MESSAGE_SIZE: usize = 1024 * 1024; // 1 MiB

/// The maximum size of a message that can be transmitted in the network.
pub(crate) const MAXIMUM_MESSAGE_SIZE: usize = 128 * 1024 * 1024; // 128 MiB

/// The codec used to decode and encode network `Message`s.
pub struct MessageCodec<N: Network> {
    codec: LengthDelimitedCodec,
    _phantom: PhantomData<N>,
}

impl<N: Network> MessageCodec<N> {
    pub fn handshake() -> Self {
        let mut codec = Self::default();
        codec.codec.set_max_frame_length(MAXIMUM_HANDSHAKE_MESSAGE_SIZE);
        codec
    }
}

impl<N: Network> Default for MessageCodec<N> {
    fn default() -> Self {
        Self {
            codec: LengthDelimitedCodec::builder().max_frame_length(MAXIMUM_MESSAGE_SIZE).little_endian().new_codec(),
            _phantom: Default::default(),
        }
    }
}

impl<N: Network> Encoder<Message<N>> for MessageCodec<N> {
    type Error = std::io::Error;

    fn encode(&mut self, message: Message<N>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Serialize the payload directly into dst.
        message
            .write_le(&mut dst.writer())
            // This error should never happen, the conversion is for greater compatibility.
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "serialization error"))?;

        let serialized_message = dst.split_to(dst.len()).freeze();

        self.codec.encode(serialized_message, dst)
    }
}

/// The width, in bytes, of the length prefix on the wire. Matches `LengthDelimitedCodec`'s
/// default (`.little_endian()` is the only builder option `MessageCodec` sets besides
/// `max_frame_length`, so the rest of the format - a 4-byte length field, not itself included in
/// the count, immediately followed by that many payload bytes - is `LengthDelimitedCodec`'s
/// default framing).
const LENGTH_PREFIX_SIZE: usize = 4;

/// The width, in bytes, of the message ID that leads every payload (see `Message::id`/`ToBytes`).
const ID_SIZE: usize = 2;

impl<N: Network> Decoder for MessageCodec<N> {
    type Error = std::io::Error;
    type Item = Message<N>;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Wait for the length prefix to fully arrive.
        if source.len() < LENGTH_PREFIX_SIZE {
            return Ok(None);
        }
        let declared_len = u32::from_le_bytes(source[..LENGTH_PREFIX_SIZE].try_into().unwrap()) as usize;

        // Reject a frame that couldn't be valid for any message type, without waiting for the
        // rest of a peer-declared, possibly enormous, body to arrive.
        if declared_len > self.codec.max_frame_length() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "frame is too large"));
        }

        // Wait for the message ID - which leads the payload - to arrive. This is at most
        // `LENGTH_PREFIX_SIZE + ID_SIZE` bytes, arriving effectively immediately regardless of how
        // large the declared frame is, since it's the front of whatever the peer sends first.
        if source.len() < LENGTH_PREFIX_SIZE + ID_SIZE {
            return Ok(None);
        }
        let id_bytes = source[LENGTH_PREFIX_SIZE..LENGTH_PREFIX_SIZE + ID_SIZE].try_into().unwrap();
        let id = u16::from_le_bytes(id_bytes);

        // Reject a declared length that exceeds what this specific message type could
        // legitimately need - before waiting for, and buffering, the rest of the body. An
        // unrecognized ID is rejected outright rather than falling through to the generic ceiling.
        match Message::<N>::max_size_for_id(id) {
            Some(max_size) if declared_len <= max_size => {}
            _ => {
                return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "message is too large for its type"));
            }
        }

        // Wait for the rest of the frame body to arrive.
        let frame_len = LENGTH_PREFIX_SIZE + declared_len;
        if source.len() < frame_len {
            source.reserve(frame_len - source.len());
            return Ok(None);
        }

        // Extract the frame body.
        source.advance(LENGTH_PREFIX_SIZE);
        let bytes = source.split_to(declared_len);

        // Convert the bytes to a message, or fail if it is not valid.
        let reader = bytes.reader();
        match Message::read_le(reader) {
            Ok(message) => Ok(Some(message)),
            Err(error) => {
                warn!("Failed to deserialize a message - {}", error);
                Err(std::io::ErrorKind::InvalidData.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{
        PuzzleResponse,
        UnconfirmedSolution,
        UnconfirmedTransaction,
        puzzle_response::prop_tests::any_large_puzzle_response,
        unconfirmed_solution::prop_tests::any_large_unconfirmed_solution,
        unconfirmed_transaction::prop_tests::{any_large_unconfirmed_transaction, any_unconfirmed_transaction},
    };

    use proptest::prelude::ProptestConfig;
    use test_strategy::proptest;

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    #[proptest]
    fn unconfirmed_transaction(#[strategy(any_unconfirmed_transaction())] tx: UnconfirmedTransaction<CurrentNetwork>) {
        let mut bytes = BytesMut::new();
        let mut codec = MessageCodec::<CurrentNetwork>::default();
        assert!(codec.encode(Message::UnconfirmedTransaction(tx), &mut bytes).is_ok());
        assert!(codec.decode(&mut bytes).is_ok());
    }

    #[proptest(ProptestConfig { cases : 10, ..ProptestConfig::default() })]
    fn overly_large_unconfirmed_transaction(
        #[strategy(any_large_unconfirmed_transaction())] tx: UnconfirmedTransaction<CurrentNetwork>,
    ) {
        let mut bytes = BytesMut::new();
        let mut codec = MessageCodec::<CurrentNetwork>::default();
        assert!(codec.encode(Message::UnconfirmedTransaction(tx), &mut bytes).is_ok());
        assert!(matches!(codec.decode(&mut bytes), Err(err) if err.kind() == std::io::ErrorKind::InvalidData));
    }

    #[proptest(ProptestConfig { cases : 10, ..ProptestConfig::default() })]
    fn overly_large_puzzle_response(#[strategy(any_large_puzzle_response())] message: PuzzleResponse<CurrentNetwork>) {
        let mut bytes = BytesMut::new();
        let mut codec = MessageCodec::<CurrentNetwork>::default();
        assert!(codec.encode(Message::PuzzleResponse(message), &mut bytes).is_ok());
        assert!(matches!(codec.decode(&mut bytes), Err(err) if err.kind() == std::io::ErrorKind::InvalidData));
    }

    #[proptest(ProptestConfig { cases : 10, ..ProptestConfig::default() })]
    fn overly_large_unconfirmed_solution(
        #[strategy(any_large_unconfirmed_solution())] solution: UnconfirmedSolution<CurrentNetwork>,
    ) {
        let mut bytes = BytesMut::new();
        let mut codec = MessageCodec::<CurrentNetwork>::default();
        assert!(codec.encode(Message::UnconfirmedSolution(solution), &mut bytes).is_ok());
        assert!(matches!(codec.decode(&mut bytes), Err(err) if err.kind() == std::io::ErrorKind::InvalidData));
    }

    /// The property that actually matters: an oversized message for a capped type is rejected
    /// the moment its length prefix and ID are visible - without ever waiting for (and buffering)
    /// the rest of a peer-declared body that may not even be sent.
    #[test]
    fn oversized_message_is_rejected_without_buffering_its_body() {
        let declared_len: u32 = 100_000_000; // far beyond any small-message cap, never actually sent
        let id: u16 = 10; // PuzzleResponse

        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(&declared_len.to_le_bytes());
        bytes.extend_from_slice(&id.to_le_bytes());
        // Deliberately no further bytes: a real peer sending this much data would need to
        // actually transmit `declared_len` bytes, which we never provide here.

        let mut codec = MessageCodec::<CurrentNetwork>::default();
        let result = codec.decode(&mut bytes);
        assert!(matches!(result, Err(err) if err.kind() == std::io::ErrorKind::InvalidData));
    }

    /// A message ID this node doesn't recognize is rejected outright, not treated as unbounded.
    #[test]
    fn unrecognized_message_id_is_rejected() {
        let declared_len: u32 = 1;
        let id: u16 = 9999;

        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(&declared_len.to_le_bytes());
        bytes.extend_from_slice(&id.to_le_bytes());
        bytes.extend_from_slice(&[0u8]);

        let mut codec = MessageCodec::<CurrentNetwork>::default();
        let result = codec.decode(&mut bytes);
        assert!(matches!(result, Err(err) if err.kind() == std::io::ErrorKind::InvalidData));
    }
}
