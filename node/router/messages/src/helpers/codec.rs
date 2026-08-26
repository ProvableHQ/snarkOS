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

use crate::{MESSAGE_ID_SIZE, Message, MessageId};
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
            codec: LengthDelimitedCodec::builder()
                .max_frame_length(MAXIMUM_MESSAGE_SIZE)
                .length_field_type::<FrameLength>()
                .little_endian()
                .new_codec(),
            _phantom: Default::default(),
        }
    }
}

/// The type of a frame's length prefix on the wire.
///
/// Both directions below write and parse this prefix by hand, while `self.codec` is what enforces
/// `max_frame_length`; passing the type to `length_field_type` on the builder keeps the three from
/// disagreeing about the framing, rather than leaving it to `LengthDelimitedCodec`'s defaults.
type FrameLength = u32;

/// The width, in bytes, of the length prefix on the wire. The prefix is not itself included in
/// the length it declares.
const LENGTH_PREFIX_SIZE: usize = size_of::<FrameLength>();

impl<N: Network> Encoder<Message<N>> for MessageCodec<N> {
    type Error = std::io::Error;

    fn encode(&mut self, message: Message<N>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Reserve the frame's length prefix, remembering where it has to be written.
        //
        // The length is not known until the message has been serialized, so the alternative is to
        // serialize the message elsewhere and copy the result in once its size is known.
        //
        // Note that `dst` is the connection's write buffer and is not necessarily empty: it may
        // still hold a frame that has not been flushed yet. Everything below is therefore relative
        // to where this frame starts, not to the start of the buffer - taking the whole buffer
        // would fold an already-encoded frame into this one's payload. This mirrors what
        // `EventCodec` does, for the same reason.
        let frame_offset = dst.len();
        dst.extend_from_slice(&[0u8; LENGTH_PREFIX_SIZE]);

        // Serialize the payload directly into dst.
        if let Err(error) = message.write_le(&mut dst.writer()) {
            // Leave the buffer as it was found, so a failed encode cannot corrupt the stream.
            dst.truncate(frame_offset);
            error!("Failed to serialize a message - {error}");
            // This error should never happen, the conversion is for greater compatibility.
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "serialization error"));
        }

        // Determine the length of what was just written, and ensure it is a permitted frame size.
        let frame_len = dst.len() - frame_offset - LENGTH_PREFIX_SIZE;
        if frame_len > self.codec.max_frame_length() {
            dst.truncate(frame_offset);
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "frame size too big"));
        }

        // Fill in the length prefix, in the same little-endian `FrameLength` framing the decoder
        // below reads back. The roundtrip tests cover that agreement.
        let frame_len = FrameLength::try_from(frame_len)
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "frame size too big"))?;
        dst[frame_offset..frame_offset + LENGTH_PREFIX_SIZE].copy_from_slice(&frame_len.to_le_bytes());

        Ok(())
    }
}

impl<N: Network> Decoder for MessageCodec<N> {
    type Error = std::io::Error;
    type Item = Message<N>;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Wait for the length prefix to fully arrive.
        if source.len() < LENGTH_PREFIX_SIZE {
            return Ok(None);
        }
        let declared_len = FrameLength::from_le_bytes(source[..LENGTH_PREFIX_SIZE].try_into().unwrap()) as usize;

        // Reject a frame that couldn't be valid for any message type, without waiting for the
        // rest of a peer-declared, possibly enormous, body to arrive.
        if declared_len > self.codec.max_frame_length() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "frame is too large"));
        }

        // Reject a frame too short to carry a message ID. Such a frame is *already complete* at
        // this point, so it has to be rejected here rather than waited on: no further bytes are
        // coming that would make it decodable, and reading an ID past its end would read bytes
        // belonging to the next frame.
        if declared_len < MESSAGE_ID_SIZE {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "frame is too short to hold a message ID",
            ));
        }

        // Wait for the message ID - which leads the payload - to arrive. This is at most
        // `LENGTH_PREFIX_SIZE + MESSAGE_ID_SIZE` bytes, arriving effectively immediately
        // regardless of how large the declared frame is, since it's the front of whatever the
        // peer sends first.
        if source.len() < LENGTH_PREFIX_SIZE + MESSAGE_ID_SIZE {
            return Ok(None);
        }
        let id_bytes = source[LENGTH_PREFIX_SIZE..LENGTH_PREFIX_SIZE + MESSAGE_ID_SIZE].try_into().unwrap();
        let id = u16::from_le_bytes(id_bytes);

        // Reject an ID that no message this node understands uses. `Message::read_le` could not
        // have deserialized such a payload either, so this only moves the rejection earlier - but
        // it is reported separately from a size violation, since a peer speaking a protocol we
        // don't know and a peer sending an oversized frame are different problems.
        let Some(id) = MessageId::from_u16(id) else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "unrecognized message ID"));
        };

        // Reject a declared length that exceeds what this specific message type could
        // legitimately need - before waiting for, and buffering, the rest of the body.
        if declared_len > id.max_size::<N>() {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "message is too large for its type"));
        }

        // Wait for the rest of the frame body to arrive.
        //
        // Note there is deliberately no `reserve` of the declared length here. Reserving it would
        // hand a peer that has sent six bytes an allocation of whatever size it named - which is
        // the cost this check exists to avoid. Letting the read buffer grow as bytes actually
        // arrive keeps the memory a connection holds proportional to what it has really sent.
        let frame_len = LENGTH_PREFIX_SIZE + declared_len;
        if source.len() < frame_len {
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
        PeerRequest,
        PeerResponse,
        Ping,
        PuzzleRequest,
        PuzzleResponse,
        UnconfirmedSolution,
        UnconfirmedTransaction,
        puzzle_response::prop_tests::any_large_puzzle_response,
        unconfirmed_solution::prop_tests::any_large_unconfirmed_solution,
        unconfirmed_transaction::prop_tests::{
            any_max_size_unconfirmed_transaction,
            any_oversized_unconfirmed_transaction,
            any_unconfirmed_transaction,
        },
    };

    use crate::message_id::UNCONFIRMED_TRANSACTION_OVERHEAD;
    use snarkos_node_network::NodeType;
    use snarkos_node_sync_locators::{CHECKPOINT_INTERVAL, test_helpers::sample_block_locators};

    use proptest::prelude::ProptestConfig;
    use std::net::{Ipv6Addr, SocketAddr};
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
        #[strategy(any_oversized_unconfirmed_transaction())] tx: UnconfirmedTransaction<CurrentNetwork>,
    ) {
        let mut bytes = BytesMut::new();
        let mut codec = MessageCodec::<CurrentNetwork>::default();
        assert!(codec.encode(Message::UnconfirmedTransaction(tx), &mut bytes).is_ok());
        assert!(matches!(codec.decode(&mut bytes), Err(err) if err.kind() == std::io::ErrorKind::InvalidData));
    }

    /// A transaction of exactly the maximum permitted size is valid, so the message carrying it
    /// has to survive the wire. The cap is on the frame, and the frame is larger than the
    /// transaction by the message ID and the transaction ID that precede it.
    #[proptest(ProptestConfig { cases : 10, ..ProptestConfig::default() })]
    fn max_size_unconfirmed_transaction_is_accepted(
        #[strategy(any_max_size_unconfirmed_transaction())] tx: UnconfirmedTransaction<CurrentNetwork>,
    ) {
        let mut bytes = BytesMut::new();
        let mut codec = MessageCodec::<CurrentNetwork>::default();
        assert!(codec.encode(Message::UnconfirmedTransaction(tx), &mut bytes).is_ok());

        // The frame is exactly the transaction plus its envelope, and that is exactly the cap.
        let frame_len = bytes.len() - LENGTH_PREFIX_SIZE;
        assert_eq!(frame_len, CurrentNetwork::LATEST_MAX_TRANSACTION_SIZE() + UNCONFIRMED_TRANSACTION_OVERHEAD);
        assert_eq!(frame_len, MessageId::UnconfirmedTransaction.max_size::<CurrentNetwork>());

        assert_eq!(codec.decode(&mut bytes).unwrap().unwrap().id(), MessageId::UnconfirmedTransaction);
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

    /// A message ID this node doesn't recognize is rejected outright, not treated as unbounded,
    /// and is reported as an unknown ID rather than as a size problem.
    #[test]
    fn unrecognized_message_id_is_rejected() {
        let mut bytes = header(3, 9999);
        bytes.extend_from_slice(&[0u8]);

        let mut codec = MessageCodec::<CurrentNetwork>::default();
        let err = codec.decode(&mut bytes).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        assert!(err.to_string().contains("unrecognized message ID"), "misreported as: {err}");
    }

    /// A frame too short to hold a message ID is already complete when its length prefix lands,
    /// so it has to be rejected there. Waiting for more bytes would park the connection until the
    /// idle timeout on a frame that can never become decodable, and reading an ID past the end of
    /// the frame would read bytes belonging to the frame after it.
    #[test]
    fn short_frame_is_rejected_rather_than_awaited() {
        for declared_len in 0..MESSAGE_ID_SIZE {
            // The frame alone, with nothing following it.
            let mut bytes = BytesMut::new();
            bytes.extend_from_slice(&(declared_len as FrameLength).to_le_bytes());
            bytes.extend_from_slice(&vec![0u8; declared_len]);

            let mut codec = MessageCodec::<CurrentNetwork>::default();
            let err = codec.decode(&mut bytes).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);

            // The same frame with a well-formed frame behind it, whose bytes must not be
            // mistaken for this frame's message ID.
            let mut codec = MessageCodec::<CurrentNetwork>::default();
            let mut bytes = BytesMut::new();
            bytes.extend_from_slice(&(declared_len as FrameLength).to_le_bytes());
            bytes.extend_from_slice(&vec![0u8; declared_len]);
            codec.encode(Message::PeerRequest(PeerRequest), &mut bytes).unwrap();

            let err = codec.decode(&mut bytes).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidData);
        }
    }

    /// The size gate is checked against the declared length exactly: a frame declaring precisely
    /// the type's maximum is let through to wait for its body, one byte more is rejected. This
    /// pins every ID's boundary at once, including the envelope `UnconfirmedTransaction` carries
    /// on top of the transaction itself.
    #[test]
    fn size_gate_boundary_is_exact() {
        for id in MessageId::ALL {
            let max_size = id.max_size::<CurrentNetwork>();

            // Exactly at the cap: accepted by the gate, still waiting for the body, which is
            // deliberately never supplied.
            let mut codec = MessageCodec::<CurrentNetwork>::default();
            let mut bytes = header(max_size, id.as_u16());
            assert!(
                matches!(codec.decode(&mut bytes), Ok(None)),
                "{id:?} rejected a frame of exactly its maximum size ({max_size})"
            );

            // One byte over. `BlockResponse` sits at the general frame ceiling, so for it this is
            // the frame-level rejection rather than the per-type one; either way it is refused.
            let mut codec = MessageCodec::<CurrentNetwork>::default();
            let mut bytes = header(max_size + 1, id.as_u16());
            let err = codec.decode(&mut bytes).unwrap_err();
            assert_eq!(err.kind(), std::io::ErrorKind::InvalidData, "{id:?} admitted a frame one byte over its cap");
        }
    }

    /// The decoder must not allocate a peer's declared length before that peer has sent it.
    #[test]
    fn declared_length_is_not_preallocated() {
        for id in [MessageId::Ping, MessageId::BlockResponse] {
            let declared_len = id.max_size::<CurrentNetwork>();

            let mut codec = MessageCodec::<CurrentNetwork>::default();
            let mut bytes = header(declared_len, id.as_u16());
            let capacity_before = bytes.capacity();

            assert!(matches!(codec.decode(&mut bytes), Ok(None)));
            assert!(
                bytes.capacity() <= capacity_before,
                "{id:?}: {} bytes on the wire grew the read buffer to {} bytes",
                LENGTH_PREFIX_SIZE + MESSAGE_ID_SIZE,
                bytes.capacity()
            );
        }
    }

    /// `decode` is called again on every read, with whatever has arrived so far, so it has to be
    /// correct at every chunk boundary - not just when a frame happens to arrive whole. Feeding a
    /// real message one byte at a time also exercises the agreement between the hand-parsed
    /// length prefix and the prefix `self.codec` writes.
    #[proptest(ProptestConfig { cases : 16, ..ProptestConfig::default() })]
    fn decodes_at_every_chunk_boundary(
        #[strategy(any_unconfirmed_transaction())] tx: UnconfirmedTransaction<CurrentNetwork>,
    ) {
        let expected = Message::UnconfirmedTransaction(tx);

        let mut encoded = BytesMut::new();
        MessageCodec::<CurrentNetwork>::default().encode(expected.clone(), &mut encoded).unwrap();

        let mut codec = MessageCodec::<CurrentNetwork>::default();
        let mut source = BytesMut::new();
        for (index, byte) in encoded.iter().enumerate() {
            source.extend_from_slice(&[*byte]);
            let is_last = index + 1 == encoded.len();
            match codec.decode(&mut source) {
                Ok(None) if !is_last => {}
                Ok(Some(decoded)) if is_last => {
                    assert_eq!(decoded.id(), expected.id());
                    assert!(source.is_empty(), "the frame was not fully consumed");
                }
                other => panic!("at byte {}/{}: {other:?}", index + 1, encoded.len()),
            }
        }
    }

    /// Encoding into a buffer that already holds an unflushed frame must append a second frame,
    /// not fold the first one into the second one's payload - `FramedWrite::feed` encodes into a
    /// persistent write buffer, so the codec cannot assume it is handed an empty one. Decoding
    /// the result then has to yield the two frames in order, without either one's bytes leaking
    /// into the other.
    #[test]
    fn encoding_appends_to_a_non_empty_buffer() {
        let mut codec = MessageCodec::<CurrentNetwork>::default();

        let mut bytes = BytesMut::new();
        codec.encode(Message::PeerRequest(PeerRequest), &mut bytes).unwrap();
        codec.encode(Message::PuzzleRequest(PuzzleRequest), &mut bytes).unwrap();

        assert_eq!(codec.decode(&mut bytes).unwrap().unwrap().id(), MessageId::PeerRequest);
        assert_eq!(codec.decode(&mut bytes).unwrap().unwrap().id(), MessageId::PuzzleRequest);
        assert!(matches!(codec.decode(&mut bytes), Ok(None)));
        assert!(bytes.is_empty());
    }

    /// The caps have to admit the largest message an honest node will actually send, or peers
    /// start disconnecting each other. `PeerResponse` is the tightest of the small messages: a
    /// full peer list of IPv6 addresses, every one of them carrying a height.
    #[test]
    fn largest_honest_peer_response_is_accepted() {
        let peers = (0..u8::MAX as u16)
            .map(|i| (SocketAddr::new(Ipv6Addr::new(0x2001, 0xdb8, i, i, i, i, i, i).into(), 4130), Some(u32::MAX)))
            .collect::<Vec<_>>();

        let mut codec = MessageCodec::<CurrentNetwork>::default();
        let mut bytes = BytesMut::new();
        codec.encode(Message::PeerResponse(PeerResponse { peers }), &mut bytes).unwrap();

        assert!(
            bytes.len() - LENGTH_PREFIX_SIZE <= MessageId::PeerResponse.max_size::<CurrentNetwork>(),
            "a full PeerResponse of IPv6 peers with heights is {} bytes, over the cap",
            bytes.len() - LENGTH_PREFIX_SIZE
        );
        assert_eq!(codec.decode(&mut bytes).unwrap().unwrap().id(), MessageId::PeerResponse);
    }

    /// `Ping` is the one capped message whose size is not fixed: it grows with the height of the
    /// chain. The cap has to hold the largest locator set the wire format permits, which is the
    /// arithmetic `MAX_PING_MESSAGE_SIZE` asserts against - checked here against what the
    /// serializer really produces rather than against that arithmetic.
    ///
    /// The height used is the largest one whose checkpoint count `BlockLocators::read_le` still
    /// accepts, i.e. exactly `u32::MAX / CHECKPOINT_INTERVAL` checkpoints.
    #[test]
    fn largest_possible_ping_is_accepted() {
        let max_checkpoints = u32::MAX / CHECKPOINT_INTERVAL;
        let height = max_checkpoints * CHECKPOINT_INTERVAL - 1;

        let locators = sample_block_locators(height);
        assert_eq!(locators.checkpoints.len(), max_checkpoints as usize, "not the largest acceptable locator set");

        let ping = Ping {
            version: Message::<CurrentNetwork>::latest_message_version(),
            node_type: NodeType::Validator,
            block_locators: Some(locators),
        };

        let mut codec = MessageCodec::<CurrentNetwork>::default();
        let mut bytes = BytesMut::new();
        codec.encode(Message::Ping(ping), &mut bytes).unwrap();

        assert!(
            bytes.len() - LENGTH_PREFIX_SIZE <= MessageId::Ping.max_size::<CurrentNetwork>(),
            "a Ping at the maximum chain height is {} bytes, over the cap",
            bytes.len() - LENGTH_PREFIX_SIZE
        );
        assert_eq!(codec.decode(&mut bytes).unwrap().unwrap().id(), MessageId::Ping);
    }

    /// Builds a bare frame header: a length prefix and a message ID, with no body behind it.
    fn header(declared_len: usize, id: u16) -> BytesMut {
        let mut bytes = BytesMut::new();
        bytes.extend_from_slice(&(declared_len as FrameLength).to_le_bytes());
        bytes.extend_from_slice(&id.to_le_bytes());
        bytes
    }
}
