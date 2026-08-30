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

use crate::{
    bft::events::Event,
    bootstrap_client::network::MessageOrEvent,
    router::messages::{Message, MessageId},
};
use snarkvm::prelude::{FromBytes, Network, ToBytes};

use bytes::{BufMut, BytesMut};
use core::marker::PhantomData;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

/// The maximum size of a message that can be transmitted during the handshake.
const MAX_HANDSHAKE_SIZE: usize = 1024 * 1024; // 1 MiB

/// The message types a bootstrap client never has a legitimate reason to receive, and so does not
/// size its frame ceiling for.
///
/// `BlockResponse`: it never sends a `BlockRequest` - it does not sync blocks - so no well-behaved
/// peer sends it the response.
///
/// `Ping`: unreachable here twice over. No node sends one to a `BootstrapClient` peer -
/// `Client::on_connect` sends only a `PeerRequest`, and the validator and prover routers skip
/// `ping.on_peer_connected` for this node type, so a bootstrap peer never enters the ping
/// rotation. And `decode` below routes wire ID 7 to `Event::read_le` rather than
/// `Message::read_le`, so a `Message::Ping` frame would be rejected even if one arrived. Leaving
/// it in would set the ceiling to `MAX_PING_MESSAGE_SIZE` (16 MiB) - eight times what is needed -
/// and `LengthDelimitedCodec` reserves the declared frame length up front, so a peer sending only
/// a four-byte length prefix would reserve all of it.
const EXCLUDED_IDS: &[MessageId] = &[MessageId::BlockResponse, MessageId::Ping];

/// The maximum size of a post-handshake message that can be obtained from the network.
///
/// `decode` below discards the message types a bootstrap client isn't interested in, but it can
/// only do so *after* the length-delimited codec has produced the frame - reading the ID means
/// reading the frame. So this ceiling has to cover every type a well-behaved peer may send,
/// including the ones that are then discarded, or the frame errors out and the connection is
/// dropped instead. `UnconfirmedTransaction` is the type that makes this concrete: transaction
/// gossip reaches a bootstrap client (`Routing::propagate` sends to every connected peer, not
/// just validators), and a maximum-size deployment is far larger than the fixed 2 MiB ceiling
/// this used to carry.
///
/// This codec frames `Event`s as well as `Message`s, and `Event` has no per-type caps to derive
/// from - hence the floor below, which the message-derived ceiling is only ever raised above.
fn max_post_handshake_size<N: Network>() -> usize {
    MessageId::max_size_over::<N>(MessageId::all().filter(|id| !EXCLUDED_IDS.contains(id))).max(MIN_EVENT_FRAME_SIZE)
}

/// The floor the ceiling above may not drop below, because `Event`s share this codec and are not
/// part of its derivation.
///
/// The `Event` that could rival a message is `PrimaryPing`, which embeds the same `BlockLocators`
/// that motivates `MAX_PING_MESSAGE_SIZE`. There is no `EventId::max_size` to ask, so this is the
/// fixed ceiling this codec carried before - under which `PrimaryPing` has been fine in production
/// - restated as a floor rather than left to coincidence.
///
/// Without it the derivation is one edit away from breaking: `decode` discards transaction gossip,
/// so adding `UnconfirmedTransaction` to `EXCLUDED_IDS` looks like a tidy-up, and would drop the
/// ceiling to `MAX_SMALL_MESSAGE_SIZE` (8 KiB) - erroring every `PrimaryPing` and dropping every
/// validator connection.
const MIN_EVENT_FRAME_SIZE: usize = 2 * 1024 * 1024; // 2 MiB

/// The codec used to decode and encode network messages.
pub struct BootstrapClientCodec<N: Network> {
    codec: LengthDelimitedCodec,
    _phantom: PhantomData<N>,
}

impl<N: Network> BootstrapClientCodec<N> {
    pub fn handshake() -> Self {
        let mut codec = Self::default();
        codec.codec.set_max_frame_length(MAX_HANDSHAKE_SIZE);
        codec
    }
}

impl<N: Network> Default for BootstrapClientCodec<N> {
    fn default() -> Self {
        Self {
            codec: LengthDelimitedCodec::builder()
                .max_frame_length(max_post_handshake_size::<N>())
                .little_endian()
                .new_codec(),
            _phantom: Default::default(),
        }
    }
}

impl<N: Network> Encoder<Message<N>> for BootstrapClientCodec<N> {
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

impl<N: Network> Encoder<Event<N>> for BootstrapClientCodec<N> {
    type Error = std::io::Error;

    fn encode(&mut self, event: Event<N>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Serialize the payload directly into dst.
        event
            .write_le(&mut dst.writer())
            // This error should never happen, the conversion is for greater compatibility.
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "serialization error"))?;

        let serialized_event = dst.split_to(dst.len()).freeze();

        self.codec.encode(serialized_event, dst)
    }
}

impl<N: Network> Encoder<MessageOrEvent<N>> for BootstrapClientCodec<N> {
    type Error = std::io::Error;

    fn encode(&mut self, item: MessageOrEvent<N>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Serialize the payload directly into dst.
        match item {
            MessageOrEvent::Message(message) => self.encode(message, dst),
            MessageOrEvent::Event(event) => self.encode(event, dst),
        }
    }
}

impl<N: Network> Decoder for BootstrapClientCodec<N> {
    type Error = std::io::Error;
    type Item = MessageOrEvent<N>;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Decode a frame containing bytes belonging to a message.
        let bytes = match self.codec.decode(source)? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        // Reject invalid/truncated messages.
        if bytes.len() < 2 {
            warn!("Failed to deserialize a message: too short");
            return Err(std::io::ErrorKind::InvalidData.into());
        }

        // Check the ID of the serialized Message or Event.
        let message_id = u16::from_le_bytes(bytes[..2].try_into().unwrap());

        // Discard messages that aren't of interest to a bootstrapper node.
        match message_id {
            2..=5 => match Message::<N>::check_size(&bytes).and_then(|()| Message::read_le(&bytes[..])) {
                Ok(message) => Ok(Some(MessageOrEvent::Message(message))),
                Err(error) => {
                    warn!("Failed to deserialize a message: {error}");
                    Err(std::io::ErrorKind::InvalidData.into())
                }
            },
            7..=9 | 13 => match Event::read_le(&bytes[..]) {
                Ok(event) => Ok(Some(MessageOrEvent::Event(event))),
                Err(error) => {
                    warn!("Failed to deserialize a message: {error}");
                    Err(std::io::ErrorKind::InvalidData.into())
                }
            },
            id => {
                trace!("Ignoring an unhandled message (ID {id})");
                Ok(None)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    /// Wraps a payload in the length prefix `LengthDelimitedCodec` expects.
    fn frame(payload: &[u8]) -> BytesMut {
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buffer.extend_from_slice(payload);
        buffer
    }

    /// The ceiling is exactly `UnconfirmedTransaction`'s cap - the largest type a bootstrap client
    /// may actually be sent, and the one the old fixed 2 MiB ceiling was below.
    ///
    /// This is pinned as an equality rather than as a pair of inequalities on purpose. `∀ id: id
    /// fits the maximum over those ids` is the definition of a maximum and holds for any exclusion
    /// list at all; it would pass just as happily with `Ping` left in and the ceiling eight times
    /// higher than it needs to be. `LengthDelimitedCodec` reserves the declared frame length
    /// before any of it arrives, so the value, not just the ordering, is what matters here.
    #[test]
    fn ceiling_is_the_largest_type_a_bootstrap_client_can_be_sent() {
        assert_eq!(
            max_post_handshake_size::<CurrentNetwork>(),
            MessageId::UnconfirmedTransaction.max_size::<CurrentNetwork>()
        );
    }

    /// `MIN_EVENT_FRAME_SIZE` is load-bearing, not belt-and-braces: with the message-derived
    /// ceiling collapsed - which one plausible edit to `EXCLUDED_IDS` would do, since `decode`
    /// discards transaction gossip anyway - the floor is the only thing left holding the ceiling
    /// above what an `Event` needs.
    #[test]
    fn the_event_floor_holds_when_the_message_derivation_collapses() {
        let collapsed = MessageId::max_size_over::<CurrentNetwork>(std::iter::empty());
        assert!(collapsed < MIN_EVENT_FRAME_SIZE, "this test no longer exercises the floor");
        assert!(collapsed.max(MIN_EVENT_FRAME_SIZE) >= MIN_EVENT_FRAME_SIZE);
        assert!(max_post_handshake_size::<CurrentNetwork>() >= MIN_EVENT_FRAME_SIZE);
    }

    /// An oversized frame is rejected by the type's own cap, not merely by the connection's
    /// ceiling. The ceiling admits the largest type any peer may send, so without this a peer
    /// could tag 2.3 MB as a `Disconnect` (cap 8 KiB) and have all of it buffered before
    /// `read_le` noticed. `MessageCodec::decode` has always called `check_size`; this codec did
    /// not, which left the per-type caps derived but unenforced.
    #[test]
    fn an_oversized_frame_is_rejected_by_its_own_type_cap() {
        let oversized = vec![0u8; MessageId::Disconnect.max_size::<CurrentNetwork>() + 1];
        let mut source = frame(&{
            let mut payload = MessageId::Disconnect.as_u16().to_le_bytes().to_vec();
            payload.extend_from_slice(&oversized);
            payload
        });

        let mut codec = BootstrapClientCodec::<CurrentNetwork>::default();
        assert!(codec.decode(&mut source).is_err(), "a frame far over its type's cap was accepted");
    }
}
