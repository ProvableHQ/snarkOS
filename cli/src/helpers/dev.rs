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

use snarkos_node::{bft::MEMORY_POOL_PORT, router::DEFAULT_NODE_PORT};

use snarkvm::{console::network::Network, prelude::PrivateKey};

use anyhow::Result;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;
pub use snarkos_utilities::DEVELOPMENT_MODE_RNG_SEED;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

/// The development mode number of genesis committee members.
pub const DEVELOPMENT_MODE_NUM_GENESIS_COMMITTEE_MEMBERS: u16 = 4;

/// The number of validators a devnet client connects to by default.
pub const DEVNET_NUM_VALIDATORS_PER_CLIENT: u16 = 2;

/// Get the private key for a validator in development mode.
pub fn get_development_key<N: Network>(index: u16) -> Result<PrivateKey<N>> {
    // Sample the private key of this node.
    // Initialize the (fixed) RNG.
    let mut rng = ChaChaRng::seed_from_u64(DEVELOPMENT_MODE_RNG_SEED);
    // Iterate through 'dev' address instances to match the account.
    for _ in 0..index {
        let _ = PrivateKey::<N>::new(&mut rng)?;
    }

    PrivateKey::<N>::new(&mut rng)
}

/// Returns the indicies of validators a particular devnet client will connect to.
pub fn get_devnet_validators_for_client(dev: u16, num_validators: u16) -> Vec<u16> {
    (0..DEVNET_NUM_VALIDATORS_PER_CLIENT).map(|i| (dev + i) % num_validators).collect()
}

/// Returns the gateway address a particular devnet validator will listen on.
pub fn get_devnet_gateway_address_for_validator(dev: u16) -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, MEMORY_POOL_PORT + dev))
}

/// Returns the router address a particular devnet validator will list on.
pub fn get_devnet_router_address_for_node(dev: u16) -> SocketAddr {
    SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, DEFAULT_NODE_PORT + dev))
}
