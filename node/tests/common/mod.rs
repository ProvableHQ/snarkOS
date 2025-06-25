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

pub mod node;
pub mod test_peer;

use std::str::FromStr;

use snarkos_account::Account;
use snarkvm::prelude::{FromBytes, MainnetV0 as CurrentNetwork, Network, block::Block};

/// Returns a fixed account.
pub fn sample_account() -> Account<CurrentNetwork> {
    Account::<CurrentNetwork>::from_str("APrivateKey1zkp2oVPTci9kKcUprnbzMwq95Di1MQERpYBhEeqvkrDirK1").unwrap()
}

/// Loads the current network's genesis block.
pub fn sample_genesis_block() -> Block<CurrentNetwork> {
    Block::<CurrentNetwork>::from_bytes_le(CurrentNetwork::genesis_bytes()).unwrap()
}

/// Enables logging in tests.
pub fn initialize_logger(verbosity: u8) {
    let verbosity_str = match verbosity {
        0 => "info",
        1 => "debug",
        2..=4 => "trace",
        _ => "info",
    };

    // Filter out undesirable logs.
    let filter = tracing_subscriber::EnvFilter::from_str(verbosity_str)
        .unwrap()
        .add_directive("snarkos=off".parse().unwrap())
        .add_directive("tokio_util=off".parse().unwrap())
        .add_directive("mio=off".parse().unwrap());

    // Initialize tracing.
    let _ = tracing_subscriber::fmt().with_env_filter(filter).with_target(verbosity > 2).try_init();
}
