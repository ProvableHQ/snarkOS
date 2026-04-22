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

use snarkos_node_network::BlockResponse;
use snarkvm::prelude::{FromBytes, Network, ToBytes};

use bytes::{Buf, BufMut, BytesMut};
use core::marker::PhantomData;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
use tracing::*;

/// The maximum size of a message that can be transmitted in the network.
const MAX_MSG_SIZE: usize = 256 * 1024 * 1024; // 256 MiB

/// The codec used to decode and encode network messages.
pub struct SyncCodec<N: Network> {
    codec: LengthDelimitedCodec,
    _phantom: PhantomData<N>,
}

impl<N: Network> Default for SyncCodec<N> {
    fn default() -> Self {
        Self {
            codec: LengthDelimitedCodec::builder().max_frame_length(MAX_MSG_SIZE).little_endian().new_codec(),
            _phantom: Default::default(),
        }
    }
}

impl<N: Network> Encoder<BlockResponse<N>> for SyncCodec<N> {
    type Error = std::io::Error;

    fn encode(&mut self, msg: BlockResponse<N>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Serialize the payload directly into dst.
        msg
            .write_le(&mut dst.writer())
            // This error should never happen, the conversion is for greater compatibility.
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "serialization error"))?;

        let serialized_event = dst.split_to(dst.len()).freeze();

        self.codec.encode(serialized_event, dst)
    }
}

impl<N: Network> Decoder for SyncCodec<N> {
    type Error = std::io::Error;
    type Item = BlockResponse<N>;

    fn decode(&mut self, source: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Decode a frame.
        let bytes = match self.codec.decode(source)? {
            Some(bytes) => bytes,
            None => return Ok(None),
        };

        let reader = bytes.reader();
        match BlockResponse::<N>::read_le(reader) {
            Ok(resp) => Ok(Some(resp)),
            Err(error) => {
                error!("Failed to deserialize a BlockResponse: {}", error);
                Err(std::io::ErrorKind::InvalidData.into())
            }
        }
    }
}
