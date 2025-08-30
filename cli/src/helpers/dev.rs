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

use snarkvm::{console::network::Network, prelude::PrivateKey};

use anyhow::Result;
use rand::SeedableRng;
use rand_chacha::ChaChaRng;

/// The development mode RNG seed.
pub const DEVELOPMENT_MODE_RNG_SEED: u64 = 1234567890u64;

/// The development mode number of genesis committee members.
pub const DEVELOPMENT_MODE_NUM_GENESIS_COMMITTEE_MEMBERS: u16 = 4;

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
