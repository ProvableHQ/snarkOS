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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimaryPing<N: Network> {
    pub version: u32,
    pub current_block_height: u32,
    pub primary_certificate: Data<BatchCertificate<N>>,
}

impl<N: Network> PrimaryPing<N> {
    /// Initializes a new ping event.
    pub const fn new(version: u32, current_block_height: u32, primary_certificate: Data<BatchCertificate<N>>) -> Self {
        Self { version, current_block_height, primary_certificate }
    }
}

impl<N: Network> From<(u32, u32, BatchCertificate<N>)> for PrimaryPing<N> {
    /// Initializes a new ping event.
    fn from((version, current_block_height, primary_certificate): (u32, u32, BatchCertificate<N>)) -> Self {
        Self::new(version, current_block_height, Data::Object(primary_certificate))
    }
}

impl<N: Network> EventTrait for PrimaryPing<N> {
    /// Returns the event name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        "PrimaryPing".into()
    }
}

impl<N: Network> ToBytes for PrimaryPing<N> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.version.write_le(&mut writer)?;
        self.current_block_height.write_le(&mut writer)?;
        self.primary_certificate.write_le(&mut writer)?;

        Ok(())
    }
}

impl<N: Network> FromBytes for PrimaryPing<N> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        // Read the version.
        let version = u32::read_le(&mut reader)?;
        // Read the block locators.
        let current_block_height = u32::read_le(&mut reader)?;
        // Read the primary certificate.
        let primary_certificate = Data::read_le(&mut reader)?;

        // Return the ping event.
        Ok(Self::new(version, current_block_height, primary_certificate))
    }
}

#[cfg(test)]
pub mod prop_tests {
    use crate::{PrimaryPing, certificate_response::prop_tests::any_batch_certificate};
    use snarkvm::utilities::{FromBytes, ToBytes};

    use bytes::{Buf, BufMut, BytesMut};
    use proptest::prelude::{BoxedStrategy, Strategy, any};
    use test_strategy::proptest;

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    pub fn any_primary_ping() -> BoxedStrategy<PrimaryPing<CurrentNetwork>> {
        (any::<u32>(), any::<u32>(), any_batch_certificate())
            .prop_map(|(version, current_block_height, batch_certificate)| {
                PrimaryPing::from((version, current_block_height, batch_certificate.clone()))
            })
            .boxed()
    }

    #[proptest]
    fn primary_ping_roundtrip(#[strategy(any_primary_ping())] primary_ping: PrimaryPing<CurrentNetwork>) {
        let mut bytes = BytesMut::default().writer();
        primary_ping.write_le(&mut bytes).unwrap();
        let decoded = PrimaryPing::<CurrentNetwork>::read_le(&mut bytes.into_inner().reader()).unwrap();
        assert_eq!(primary_ping.version, decoded.version);
        assert_eq!(primary_ping.current_block_height, decoded.current_block_height);
        assert_eq!(
            primary_ping.primary_certificate.deserialize_blocking().unwrap(),
            decoded.primary_certificate.deserialize_blocking().unwrap(),
        );
    }
}
