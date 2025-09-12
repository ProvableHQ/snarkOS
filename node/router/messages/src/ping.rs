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

use super::*;

use snarkvm::prelude::{FromBytes, ToBytes};

use std::borrow::Cow;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ping {
    pub version: u32,
    pub node_type: NodeType,
    pub block_height: Option<u32>,
}

impl MessageTrait for Ping {
    /// Returns the message name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        "Ping".into()
    }
}

impl ToBytes for Ping {
    fn write_le<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.version.write_le(&mut writer)?;
        self.node_type.write_le(&mut writer)?;
        if let Some(height) = &self.block_height {
            1u8.write_le(&mut writer)?;
            height.write_le(&mut writer)?;
        } else {
            0u8.write_le(&mut writer)?;
        }

        Ok(())
    }
}

impl FromBytes for Ping {
    fn read_le<R: io::Read>(mut reader: R) -> io::Result<Self> {
        let version = u32::read_le(&mut reader)?;
        let node_type = NodeType::read_le(&mut reader)?;

        let selector = u8::read_le(&mut reader)?;
        let block_height = match selector {
            0 => None,
            1 => Some(u32::read_le(&mut reader)?),
            _ => return Err(error("Invalid block locators marker")),
        };

        Ok(Self { version, node_type, block_height })
    }
}

impl Ping {
    pub fn new<N: Network>(node_type: NodeType, block_height: Option<u32>) -> Self {
        Self { version: <Message<N>>::latest_message_version(), node_type, block_height }
    }
}

#[cfg(test)]
pub mod prop_tests {
    use crate::{Ping, challenge_request::prop_tests::any_node_type};
    use snarkvm::utilities::{FromBytes, ToBytes};

    use bytes::{Buf, BufMut, BytesMut};
    use proptest::prelude::{BoxedStrategy, Strategy, any};
    use test_strategy::proptest;

    pub fn any_ping() -> BoxedStrategy<Ping> {
        (any::<u32>(), any::<u32>(), any_node_type())
            .prop_map(|(version, height, node_type)| Ping { version, block_height: Some(height), node_type })
            .boxed()
    }

    #[proptest]
    fn ping_roundtrip(#[strategy(any_ping())] ping: Ping) {
        let mut bytes = BytesMut::default().writer();
        ping.write_le(&mut bytes).unwrap();
        let decoded = Ping::read_le(&mut bytes.into_inner().reader()).unwrap();
        assert_eq!(ping, decoded);
    }
}
