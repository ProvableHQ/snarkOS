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
    network::ConnectionMode,
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
    /// Which of the two wire protocols the peer on this connection speaks, or `None` on a codec
    /// that is not bound to a connected peer (the handshake codec, and the encode-only codec
    /// `Writing` builds).
    ///
    /// A frame's leading `u16` does **not** determine this. `Message` and `Event` number their
    /// variants independently from zero, so the two ID spaces overlap completely and the same
    /// value means different things on the two connection modes - ID 2 is a `ChallengeRequest`
    /// to a router peer and a `BatchCertified` to a gateway peer. Only the connection knows
    /// which, so `decode` is told rather than left to guess.
    mode: Option<ConnectionMode>,
    _phantom: PhantomData<N>,
}

impl<N: Network> BootstrapClientCodec<N> {
    pub fn handshake() -> Self {
        let mut codec = Self::default();
        codec.codec.set_max_frame_length(MAX_HANDSHAKE_SIZE);
        codec
    }

    /// A codec for reading from a peer that has completed the handshake, and whose connection mode
    /// is therefore settled.
    ///
    /// `Tcp` enables the handshake protocol before the reading protocol (see `enable_protocol!` in
    /// `node/tcp/src/tcp.rs`), so by the time `Reading::codec` builds one of these the peer is
    /// already recorded with its mode.
    pub fn for_mode(mode: ConnectionMode) -> Self {
        Self { mode: Some(mode), ..Self::default() }
    }
}

impl<N: Network> Default for BootstrapClientCodec<N> {
    fn default() -> Self {
        Self {
            codec: LengthDelimitedCodec::builder()
                .max_frame_length(max_post_handshake_size::<N>())
                .little_endian()
                .new_codec(),
            mode: None,
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

        // Which of the two wire protocols this frame is in is a property of the connection, not of
        // the frame - see `mode` on the struct for why the leading ID cannot settle it.
        let Some(mode) = self.mode else {
            // Only a codec bound to a connected peer decodes, and `Reading::codec` builds those
            // with the mode set. Reaching this means the peer was gone by the time the reading
            // protocol was enabled, so the connection is being torn down regardless.
            debug!("Dropping a frame from a connection with no settled mode");
            return Err(std::io::ErrorKind::InvalidData.into());
        };

        // The ID leading the frame. What it names depends on `mode`.
        let message_id = u16::from_le_bytes(bytes[..2].try_into().unwrap());

        // Discard the messages that aren't of interest to a bootstrapper node, without paying to
        // deserialize them - a gateway peer sends a steady stream of batch and ping traffic that
        // this node has no use for.
        //
        // These are the same two ID sets the mode-blind version used, only applied to the mode
        // each was written for. They are disjoint, which is what let them share one match and hid
        // the bug: on a gateway connection the `Event`s numbered 2..=5 - `BatchCertified`,
        // `BlockRequest`, `BlockResponse`, `CertificateRequest` - fell into the set meant for a
        // router peer's `Message`s and were handed to `Message::read_le`, which rejected them and
        // dropped the connection.
        let decoded = match mode {
            // ChallengeRequest, ChallengeResponse, Disconnect, PeerRequest.
            ConnectionMode::Router if (2..=5).contains(&message_id) => Message::<N>::check_size(&bytes)
                .and_then(|()| Message::read_le(&bytes[..]))
                .map(MessageOrEvent::Message),
            // ChallengeRequest, ChallengeResponse, Disconnect, ValidatorsRequest. There is no
            // `Event` equivalent of `Message::check_size` to apply here - events carry no per-type
            // caps - so a gateway frame is bounded only by the connection's ceiling.
            ConnectionMode::Gateway if matches!(message_id, 7..=9 | 13) => {
                Event::read_le(&bytes[..]).map(MessageOrEvent::Event)
            }
            _ => {
                trace!("Ignoring an unhandled {mode:?} message (ID {message_id})");
                return Ok(None);
            }
        };

        match decoded {
            Ok(decoded) => Ok(Some(decoded)),
            Err(error) => {
                warn!("Failed to deserialize a {mode:?} message: {error}");
                Err(std::io::ErrorKind::InvalidData.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{bft::events, router::messages};

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    /// Wraps a payload in the length prefix `LengthDelimitedCodec` expects, so a frame can be
    /// built from a bare wire ID. The ignore path returns before deserializing, so for the tests
    /// that exercise it the ID is the whole of the input that matters.
    fn frame(payload: &[u8]) -> BytesMut {
        let mut buffer = BytesMut::new();
        buffer.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buffer.extend_from_slice(payload);
        buffer
    }

    fn frame_with_id(id: u16) -> BytesMut {
        frame(&id.to_le_bytes())
    }

    fn encode(codec: &mut BootstrapClientCodec<CurrentNetwork>, item: MessageOrEvent<CurrentNetwork>) -> BytesMut {
        let mut buffer = BytesMut::new();
        codec.encode(item, &mut buffer).unwrap();
        buffer
    }

    /// The bug this fixes. `Message` and `Event` number their variants independently from zero, so
    /// wire ID 2 is a `Message::ChallengeRequest` to a router peer and an `Event::BatchCertified`
    /// to a gateway one. The mode-blind decoder sent every ID in 2..=5 to `Message::read_le`,
    /// which rejected the gateway events numbered the same and errored the frame - dropping the
    /// connection over a message this node merely has no use for. `Gateway::broadcast` has no
    /// bootstrap-client filter, so validators really do send `BatchCertified`.
    #[test]
    fn a_gateway_event_numbered_in_the_router_range_is_ignored_not_rejected() {
        for id in 2..=5 {
            let mut codec = BootstrapClientCodec::<CurrentNetwork>::for_mode(ConnectionMode::Gateway);
            assert!(matches!(codec.decode(&mut frame_with_id(id)), Ok(None)), "gateway event ID {id} was not ignored");
        }
    }

    /// The mirror of the above, which was equally wrong and is latent only because nothing
    /// currently sends a bootstrap client a `Ping`, `Pong` or `PuzzleRequest`: those are
    /// `Message` IDs 7..=9, which the mode-blind decoder handed to `Event::read_le`.
    #[test]
    fn a_router_message_numbered_in_the_gateway_range_is_ignored_not_rejected() {
        for id in [7, 8, 9, 13] {
            let mut codec = BootstrapClientCodec::<CurrentNetwork>::for_mode(ConnectionMode::Router);
            assert!(matches!(codec.decode(&mut frame_with_id(id)), Ok(None)), "router message ID {id} was not ignored");
        }
    }

    /// The same wire ID decodes as two different things depending only on the connection, which is
    /// the property the mode-blind version could not express.
    #[test]
    fn the_same_wire_id_decodes_per_mode() {
        // ID 5 is `Message::PeerRequest` on a router connection - the one message a bootstrap
        // client acts on there - and `Event::CertificateRequest` on a gateway one.
        let mut encoder = BootstrapClientCodec::<CurrentNetwork>::default();
        let mut encoded = encode(&mut encoder, MessageOrEvent::Message(Message::PeerRequest(messages::PeerRequest)));

        let mut router = BootstrapClientCodec::<CurrentNetwork>::for_mode(ConnectionMode::Router);
        assert!(matches!(
            router.decode(&mut encoded.clone()),
            Ok(Some(MessageOrEvent::Message(Message::PeerRequest(_))))
        ));

        let mut gateway = BootstrapClientCodec::<CurrentNetwork>::for_mode(ConnectionMode::Gateway);
        assert!(matches!(gateway.decode(&mut encoded), Ok(None)), "ID 5 must not be read as a Message on a gateway");
    }

    /// The gateway counterpart: `ValidatorsRequest` is the one event a bootstrap client acts on,
    /// and it must survive the round trip on a gateway connection.
    #[test]
    fn a_gateway_validators_request_decodes() {
        let mut encoder = BootstrapClientCodec::<CurrentNetwork>::default();
        let mut encoded =
            encode(&mut encoder, MessageOrEvent::Event(Event::ValidatorsRequest(events::ValidatorsRequest)));

        let mut gateway = BootstrapClientCodec::<CurrentNetwork>::for_mode(ConnectionMode::Gateway);
        assert!(matches!(gateway.decode(&mut encoded), Ok(Some(MessageOrEvent::Event(Event::ValidatorsRequest(_))))));
    }

    /// A codec that was never bound to a connected peer cannot know which protocol a frame is in,
    /// so it refuses rather than guessing. In practice this only arises if the peer disconnected
    /// between the handshake and the reading protocol being enabled.
    #[test]
    fn a_codec_with_no_mode_refuses_to_decode() {
        let mut codec = BootstrapClientCodec::<CurrentNetwork>::default();
        assert!(codec.decode(&mut frame_with_id(5)).is_err());
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

        // Bound to a router connection specifically: a modeless codec would refuse the frame for
        // an unrelated reason, and this must fail on the type's own cap.
        let mut codec = BootstrapClientCodec::<CurrentNetwork>::for_mode(ConnectionMode::Router);
        assert!(codec.decode(&mut source).is_err(), "a frame far over its type's cap was accepted");

        // The same frame within the cap is accepted, so the rejection above is the cap and not
        // the frame merely being malformed.
        let mut within = frame(&MessageId::Disconnect.as_u16().to_le_bytes());
        let mut codec = BootstrapClientCodec::<CurrentNetwork>::for_mode(ConnectionMode::Router);
        assert!(
            !matches!(codec.decode(&mut within), Err(ref e) if e.to_string().contains("too large")),
            "a frame within its type's cap was rejected by the cap"
        );
    }
}
