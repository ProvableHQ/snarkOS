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

#![forbid(unsafe_code)]

pub mod node_type;
pub use node_type::*;

pub mod peer;
pub use peer::*;

pub mod peering;
pub use peering::*;

pub mod resolver;
pub use resolver::*;

use snarkvm::prelude::Network;

use std::{net::SocketAddr, str::FromStr};
use tracing::*;

// Include the generated build information.
pub mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

/// Returns the list of bootstrap peers.
#[allow(clippy::if_same_then_else)]
pub fn bootstrap_peers<N: Network>(is_dev: bool) -> Vec<SocketAddr> {
    if cfg!(feature = "test") || is_dev {
        // Development testing contains optional bootstrap peers loaded from the environment.
        match std::env::var("TEST_BOOTSTRAP_PEERS") {
            Ok(peers) => peers.split(',').map(|peer| SocketAddr::from_str(peer).unwrap()).collect(),
            Err(err) => {
                warn!("Failed to load bootstrap peers from environment: {err}");
                vec![]
            }
        }
    } else if N::ID == snarkvm::console::network::MainnetV0::ID {
        // Mainnet contains the following bootstrap peers.
        vec![
            SocketAddr::from_str("35.231.67.219:4130").unwrap(),
            SocketAddr::from_str("34.73.195.196:4130").unwrap(),
            SocketAddr::from_str("34.23.225.202:4130").unwrap(),
            SocketAddr::from_str("34.148.16.111:4130").unwrap(),
        ]
    } else if N::ID == snarkvm::console::network::TestnetV0::ID {
        // TestnetV0 contains the following bootstrap peers.
        vec![
            SocketAddr::from_str("34.138.104.159:4130").unwrap(),
            SocketAddr::from_str("35.231.46.237:4130").unwrap(),
            SocketAddr::from_str("34.148.251.155:4130").unwrap(),
            SocketAddr::from_str("35.190.141.234:4130").unwrap(),
        ]
    } else if N::ID == snarkvm::console::network::CanaryV0::ID {
        // CanaryV0 contains the following bootstrap peers.
        vec![
            SocketAddr::from_str("34.139.88.58:4130").unwrap(),
            SocketAddr::from_str("34.139.252.207:4130").unwrap(),
            SocketAddr::from_str("35.185.98.12:4130").unwrap(),
            SocketAddr::from_str("35.231.106.26:4130").unwrap(),
        ]
    } else {
        // Unrecognized networks contain no bootstrap peers.
        vec![]
    }
}

/// Logs the peer's snarkOS repo SHA and how it compares to ours.
pub fn log_repo_sha_comparison(peer_addr: SocketAddr, peer_sha: &str, ctx: &str) {
    let our_sha = built_info::GIT_COMMIT_HASH.unwrap_or_default();
    let sha_cmp = if peer_sha == "unknown" {
        " with an unknown repo SHA".to_owned()
    } else if peer_sha == our_sha {
        format!("@{peer_sha} (same as us)")
    } else if our_sha.is_empty() {
        format!("@{peer_sha} (potentially different than us)")
    } else {
        format!("@{peer_sha} (different than us)")
    };

    debug!("{ctx} Peer '{peer_addr}' uses snarkOS{sha_cmp}");
}
