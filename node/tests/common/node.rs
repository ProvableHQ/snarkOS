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

use crate::common::test_peer::sample_genesis_block;

use snarkos_account::Account;
use snarkos_node::{Client, Prover, Validator};
use snarkos_utilities::SignalHandler;

use snarkvm::prelude::{MainnetV0 as CurrentNetwork, store::helpers::memory::ConsensusMemory};

use aleo_std::StorageMode;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

/// Bind to a random port to avoid conflicts during testing.
const ANY_ADDR: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0);

pub async fn client() -> Client<CurrentNetwork, ConsensusMemory<CurrentNetwork>> {
    Client::new(
        ANY_ADDR,
        Some(ANY_ADDR),
        10,
        Account::<CurrentNetwork>::from_str("APrivateKey1zkp2oVPTci9kKcUprnbzMwq95Di1MQERpYBhEeqvkrDirK1").unwrap(),
        &[],
        sample_genesis_block(),
        None, // No CDN.
        StorageMode::new_test(None),
        false, // Connect to untrusted peers.
        None,
        SignalHandler::new(),
    )
    .await
    .expect("couldn't create client instance")
}

pub async fn prover() -> Prover<CurrentNetwork, ConsensusMemory<CurrentNetwork>> {
    Prover::new(
        ANY_ADDR,
        Account::<CurrentNetwork>::from_str("APrivateKey1zkp2oVPTci9kKcUprnbzMwq95Di1MQERpYBhEeqvkrDirK1").unwrap(),
        &[],
        sample_genesis_block(),
        StorageMode::new_test(None),
        false,
        None,
        SignalHandler::new(),
    )
    .await
    .expect("couldn't create prover instance")
}

pub async fn validator() -> Validator<CurrentNetwork, ConsensusMemory<CurrentNetwork>> {
    Validator::new(
        ANY_ADDR,
        Some(ANY_ADDR),
        Some(ANY_ADDR),
        10,
        Account::<CurrentNetwork>::from_str("APrivateKey1zkp2oVPTci9kKcUprnbzMwq95Di1MQERpYBhEeqvkrDirK1").unwrap(),
        &[],
        &[],
        sample_genesis_block(), // Should load the current network's genesis block.
        None,                   // No CDN.
        StorageMode::new_test(None),
        false, // This test requires validators to connect to peers.
        false, // No dev traffic in production mode.
        None,
        SignalHandler::new(),
    )
    .await
    .expect("couldn't create validator instance")
}
