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

impl<N: Network> Decoder for MessageCodec<N> {
    type Error = std::io::Error;
    type Item = Message<N>;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Decode a frame containing bytes belonging to a message.
        let bytes = match self.codec.decode(source)? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        Self::Item::check_size(&bytes)?;

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
        MAX_PING_MESSAGE_SIZE,
        Transaction,
        UnconfirmedTransaction,
        ping::prop_tests::largest_possible_ping,
        unconfirmed_transaction::prop_tests::{
            any_large_unconfirmed_transaction,
            any_max_size_unconfirmed_transaction,
            any_transaction,
            any_unconfirmed_transaction,
        },
    };

    use proptest::prelude::ProptestConfig;
    use test_strategy::proptest;

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    #[proptest]
    fn unconfirmed_transaction(#[strategy(any_unconfirmed_transaction())] tx: UnconfirmedTransaction<CurrentNetwork>) {
        let mut bytes = BytesMut::new();
        let mut codec = MessageCodec::<CurrentNetwork>::default();
        assert!(codec.encode(Message::UnconfirmedTransaction(tx), &mut bytes).is_ok());
        // `Ok(None)` ("frame incomplete") would also pass a bare `.is_ok()` check, so this must
        // pin down that a message was actually decoded, not merely that decoding didn't error.
        assert!(matches!(codec.decode(&mut bytes), Ok(Some(Message::UnconfirmedTransaction(_)))));
    }

    /// Pins `UnconfirmedTransaction::OVERHEAD` against what `Message::write_le` actually emits,
    /// rather than trusting the arithmetic in its doc comment: encodes a real transaction inside
    /// an `UnconfirmedTransaction` frame and checks the frame is exactly that much larger than
    /// the transaction's own serialized size, for every size a transaction actually takes.
    #[proptest]
    fn unconfirmed_transaction_overhead_matches_the_real_encoding(
        #[strategy(any_transaction())] transaction: Transaction<CurrentNetwork>,
    ) {
        let mut transaction_bytes = BytesMut::default().writer();
        transaction.write_le(&mut transaction_bytes).unwrap();
        let transaction_len = transaction_bytes.into_inner().len();

        let mut frame = BytesMut::default().writer();
        Message::UnconfirmedTransaction(transaction.into()).write_le(&mut frame).unwrap();
        let frame_len = frame.into_inner().len();

        assert_eq!(frame_len, transaction_len + UnconfirmedTransaction::<CurrentNetwork>::OVERHEAD);
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

    /// A transaction of exactly `LATEST_MAX_TRANSACTION_SIZE` is valid - both the ledger service
    /// and the REST endpoint accept one - so the message carrying it must be accepted too, even
    /// though the frame is 39 bytes larger than the transaction itself.
    #[proptest(ProptestConfig { cases : 10, ..ProptestConfig::default() })]
    fn max_size_unconfirmed_transaction_is_accepted(
        #[strategy(any_max_size_unconfirmed_transaction())] tx: UnconfirmedTransaction<CurrentNetwork>,
    ) {
        let mut bytes = BytesMut::new();
        let mut codec = MessageCodec::<CurrentNetwork>::default();
        assert!(codec.encode(Message::UnconfirmedTransaction(tx), &mut bytes).is_ok());
        assert!(matches!(codec.decode(&mut bytes), Ok(Some(Message::UnconfirmedTransaction(_)))));
    }

    /// The largest `Ping` the wire format permits must still be accepted - the failure mode a too
    /// tight cap causes is disconnecting honest peers, which is worse than the DoS this exists to
    /// prevent.
    #[test]
    fn largest_possible_ping_is_accepted() {
        let mut bytes = BytesMut::new();
        let mut codec = MessageCodec::<CurrentNetwork>::default();
        assert!(codec.encode(Message::Ping(largest_possible_ping()), &mut bytes).is_ok());
        assert!(matches!(codec.decode(&mut bytes), Ok(Some(Message::Ping(_)))));
    }

    /// `check_size` is exercised directly here, rather than through a constructed `Ping`: nothing
    /// past a `Ping`'s own wire-format ceiling (`MAX_CHECKPOINTS` checkpoints) can actually reach
    /// `MAX_PING_MESSAGE_SIZE` - there's real headroom between the two - so a frame large enough
    /// to violate the cap isn't representable as a legitimately-shaped `Ping` at all. The content
    /// after the ID is irrelevant to `check_size`, which only inspects the ID and the length.
    #[test]
    fn oversized_ping_is_rejected() {
        let id: u16 = 7; // Ping
        let mut bytes = vec![0u8; MAX_PING_MESSAGE_SIZE + 1];
        bytes[..2].copy_from_slice(&id.to_le_bytes());
        assert!(Message::<CurrentNetwork>::check_size(&bytes).is_err());
    }

    /// The boundary itself: exactly `MAX_PING_MESSAGE_SIZE` must still be accepted by `check_size`.
    #[test]
    fn max_size_ping_frame_is_accepted_by_check_size() {
        let id: u16 = 7; // Ping
        let mut bytes = vec![0u8; MAX_PING_MESSAGE_SIZE];
        bytes[..2].copy_from_slice(&id.to_le_bytes());
        assert!(Message::<CurrentNetwork>::check_size(&bytes).is_ok());
    }
}
