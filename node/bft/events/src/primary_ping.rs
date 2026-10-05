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

use super::*;

/// A validator's claim that it will run `consensus_version` at block `height`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UpgradeSignal {
    pub height: u32,
    /// The raw `ConsensusVersion` discriminant, kept raw so that a version this build does not
    /// define still decodes instead of dropping the connection.
    pub consensus_version: u16,
}

impl ToBytes for UpgradeSignal {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.height.write_le(&mut writer)?;
        self.consensus_version.write_le(&mut writer)
    }
}

impl FromBytes for UpgradeSignal {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let height = u32::read_le(&mut reader)?;
        let consensus_version = u16::read_le(&mut reader)?;
        Ok(Self { height, consensus_version })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PrimaryPing<N: Network> {
    pub version: u32,
    pub block_locators: BlockLocators<N>,
    pub primary_certificate: Data<BatchCertificate<N>>,
    /// Present if and only if `version >= Self::UPGRADE_SIGNAL_VERSION`.
    pub upgrade_signal: Option<UpgradeSignal>,
}

impl<N: Network> PrimaryPing<N> {
    /// The first event version whose pings carry an [`UpgradeSignal`].
    ///
    /// Peers below this version reject pings with trailing bytes, so they must be sent pings
    /// without one; see [`Self::for_peer_version`].
    pub const UPGRADE_SIGNAL_VERSION: u32 = 11;

    /// Initializes a new ping event.
    pub const fn new(
        version: u32,
        block_locators: BlockLocators<N>,
        primary_certificate: Data<BatchCertificate<N>>,
        upgrade_signal: Option<UpgradeSignal>,
    ) -> Self {
        Self { version, block_locators, primary_certificate, upgrade_signal }
    }

    /// Returns this ping in the encoding understood by a peer that handshook with `peer_version`.
    pub fn for_peer_version(mut self, peer_version: u32) -> Self {
        if peer_version < Self::UPGRADE_SIGNAL_VERSION && self.version >= Self::UPGRADE_SIGNAL_VERSION {
            self.version = Self::UPGRADE_SIGNAL_VERSION - 1;
            self.upgrade_signal = None;
        }
        self
    }
}

impl<N: Network> From<(u32, BlockLocators<N>, BatchCertificate<N>, Option<UpgradeSignal>)> for PrimaryPing<N> {
    /// Initializes a new ping event.
    fn from(
        (version, block_locators, primary_certificate, upgrade_signal): (
            u32,
            BlockLocators<N>,
            BatchCertificate<N>,
            Option<UpgradeSignal>,
        ),
    ) -> Self {
        Self::new(version, block_locators, Data::Object(primary_certificate), upgrade_signal)
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
        // Write the version.
        self.version.write_le(&mut writer)?;
        // Write the block locators.
        self.block_locators.write_le(&mut writer)?;
        // Write the primary certificate.
        self.primary_certificate.write_le(&mut writer)?;
        // Write the upgrade signal.
        match (self.version >= Self::UPGRADE_SIGNAL_VERSION, &self.upgrade_signal) {
            (true, Some(upgrade_signal)) => upgrade_signal.write_le(&mut writer)?,
            (false, None) => (),
            _ => return Err(error("A ping carries an upgrade signal if and only if its version supports one")),
        }

        Ok(())
    }
}

impl<N: Network> FromBytes for PrimaryPing<N> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        // Read the version.
        let version = u32::read_le(&mut reader)?;
        // Read the block locators.
        let block_locators = BlockLocators::read_le(&mut reader)?;
        // Read the primary certificate.
        let primary_certificate = Data::read_le(&mut reader)?;
        // Read the upgrade signal.
        let upgrade_signal =
            if version >= Self::UPGRADE_SIGNAL_VERSION { Some(UpgradeSignal::read_le(&mut reader)?) } else { None };

        // Return the ping event.
        Ok(Self::new(version, block_locators, primary_certificate, upgrade_signal))
    }
}

#[cfg(test)]
pub mod prop_tests {
    use crate::{PrimaryPing, UpgradeSignal, certificate_response::prop_tests::any_batch_certificate};
    use snarkos_node_sync_locators::{BlockLocators, test_helpers::sample_block_locators};
    use snarkvm::{
        ledger::narwhal::BatchCertificate,
        utilities::{FromBytes, ToBytes},
    };

    use bytes::{Buf, BufMut, BytesMut};
    use proptest::prelude::{BoxedStrategy, Strategy, any};
    use test_strategy::proptest;

    type CurrentNetwork = snarkvm::prelude::MainnetV0;
    type Ping = PrimaryPing<CurrentNetwork>;

    pub fn any_block_locators() -> BoxedStrategy<BlockLocators<CurrentNetwork>> {
        // `sample_block_locators` inserts a checkpoint every 10_000 heights. An unconstrained
        // `u32` can therefore allocate hundreds of thousands of checkpoints and blow up codec tests.
        (0u32..50_000).prop_map(sample_block_locators).boxed()
    }

    pub fn any_upgrade_signal() -> BoxedStrategy<UpgradeSignal> {
        (any::<u32>(), any::<u16>())
            .prop_map(|(height, consensus_version)| UpgradeSignal { height, consensus_version })
            .boxed()
    }

    pub fn any_primary_ping() -> BoxedStrategy<PrimaryPing<CurrentNetwork>> {
        (any::<u32>(), any_block_locators(), any_batch_certificate(), any_upgrade_signal())
            .prop_map(|(version, block_locators, batch_certificate, upgrade_signal)| {
                let upgrade_signal = (version >= Ping::UPGRADE_SIGNAL_VERSION).then_some(upgrade_signal);
                PrimaryPing::from((version, block_locators, batch_certificate, upgrade_signal))
            })
            .boxed()
    }

    /// Encodes a ping the way peers below `UPGRADE_SIGNAL_VERSION` do.
    fn legacy_encoding(
        version: u32,
        block_locators: &BlockLocators<CurrentNetwork>,
        certificate: &BatchCertificate<CurrentNetwork>,
    ) -> Vec<u8> {
        let mut bytes = version.to_bytes_le().unwrap();
        bytes.extend(block_locators.to_bytes_le().unwrap());
        bytes.extend(snarkvm::ledger::narwhal::Data::Object(certificate.clone()).to_bytes_le().unwrap());
        bytes
    }

    #[proptest]
    fn primary_ping_for_a_legacy_peer_uses_the_legacy_encoding(
        #[strategy(any_block_locators())] block_locators: BlockLocators<CurrentNetwork>,
        #[strategy(any_batch_certificate())] certificate: BatchCertificate<CurrentNetwork>,
        #[strategy(any_upgrade_signal())] upgrade_signal: UpgradeSignal,
    ) {
        let legacy_version = Ping::UPGRADE_SIGNAL_VERSION - 1;
        let ping = Ping::from((
            Ping::UPGRADE_SIGNAL_VERSION,
            block_locators.clone(),
            certificate.clone(),
            Some(upgrade_signal),
        ))
        .for_peer_version(legacy_version);

        assert_eq!(ping.to_bytes_le().unwrap(), legacy_encoding(legacy_version, &block_locators, &certificate));
        assert_eq!(ping.upgrade_signal, None);
    }

    #[proptest]
    fn primary_ping_from_a_legacy_peer_has_no_upgrade_signal(
        #[strategy(any_block_locators())] block_locators: BlockLocators<CurrentNetwork>,
        #[strategy(any_batch_certificate())] certificate: BatchCertificate<CurrentNetwork>,
    ) {
        let legacy_version = Ping::UPGRADE_SIGNAL_VERSION - 1;
        let bytes = legacy_encoding(legacy_version, &block_locators, &certificate);
        let decoded = Ping::read_le(&bytes[..]).unwrap();
        assert_eq!(decoded.version, legacy_version);
        assert_eq!(decoded.upgrade_signal, None);
    }

    #[proptest]
    fn primary_ping_for_a_current_peer_keeps_its_upgrade_signal(
        #[strategy(any_block_locators())] block_locators: BlockLocators<CurrentNetwork>,
        #[strategy(any_batch_certificate())] certificate: BatchCertificate<CurrentNetwork>,
        #[strategy(any_upgrade_signal())] upgrade_signal: UpgradeSignal,
    ) {
        let version = Ping::UPGRADE_SIGNAL_VERSION;
        let ping = Ping::from((version, block_locators, certificate, Some(upgrade_signal))).for_peer_version(version);
        assert_eq!(ping.version, version);
        assert_eq!(ping.upgrade_signal, Some(upgrade_signal));
    }

    #[proptest]
    fn primary_ping_with_a_mismatched_upgrade_signal_does_not_encode(
        #[strategy(any_block_locators())] block_locators: BlockLocators<CurrentNetwork>,
        #[strategy(any_batch_certificate())] certificate: BatchCertificate<CurrentNetwork>,
        #[strategy(any_upgrade_signal())] upgrade_signal: UpgradeSignal,
    ) {
        let version = Ping::UPGRADE_SIGNAL_VERSION;
        let missing = Ping::from((version, block_locators.clone(), certificate.clone(), None));
        assert!(missing.to_bytes_le().is_err());
        let unexpected = Ping::from((version - 1, block_locators, certificate, Some(upgrade_signal)));
        assert!(unexpected.to_bytes_le().is_err());
    }

    #[proptest]
    fn primary_ping_roundtrip(#[strategy(any_primary_ping())] primary_ping: PrimaryPing<CurrentNetwork>) {
        let mut bytes = BytesMut::default().writer();
        primary_ping.write_le(&mut bytes).unwrap();
        let decoded = Ping::read_le(&mut bytes.into_inner().reader()).unwrap();
        assert_eq!(primary_ping.version, decoded.version);
        assert_eq!(primary_ping.block_locators, decoded.block_locators);
        assert_eq!(primary_ping.upgrade_signal, decoded.upgrade_signal);
        assert_eq!(
            primary_ping.primary_certificate.deserialize_blocking().unwrap(),
            decoded.primary_certificate.deserialize_blocking().unwrap(),
        );
    }
}
