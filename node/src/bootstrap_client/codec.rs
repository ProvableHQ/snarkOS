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

use crate::{bft::events::Event, bootstrap_client::network::MessageOrEvent, router::messages::Message};
use snarkvm::prelude::{FromBytes, Network, ToBytes};

use bytes::{BufMut, BytesMut};
use core::marker::PhantomData;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

/// The maximum size of a message that can be transmitted during the handshake.
const MAX_HANDSHAKE_SIZE: usize = 1024 * 1024; // 1 MiB
/// The maximum size of a post-handshake message that can be obtained from the network.
const MAX_POST_HANDSHAKE_SIZE: usize = 2 * 1024 * 1024; // 2 MiB

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
                .max_frame_length(MAX_POST_HANDSHAKE_SIZE)
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
            2..=5 => match Message::read_le(&bytes[..]) {
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
