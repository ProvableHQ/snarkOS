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

use std::path::{Path, PathBuf};

/// The filename of the gateway peer cache.
pub const GATWAY_PEER_CACHE_FILE: &str = "gateway-peer-cache";
/// The old filename of the gateway peer cache.
pub const LEGACY_GATWAY_PEER_CACHE_FILE: &str = "cached_gateway_peers";

/// The filename of the router peer cache.
pub const ROUTER_PEER_CACHE_FILE: &str = "router-peer-cache";
/// The old filename of the router peer cache.
pub const LEGACY_ROUTER_PEER_CACHE_FILE: &str = "cached_router_peers";

/// The filename of the proposal cache.
pub const CURRENT_PROPOSAL_CACHE_FILE: &str = "current-proposal-cache";

/// The old filename of the JWT secret for a given address.
pub fn jwt_secret_file<D: std::fmt::Display>(address: &D) -> PathBuf {
    PathBuf::from(format!("jwt_secret_{address}.txt"))
}

/// The old filename of the current proposal cache.
pub fn legacy_current_proposal_cache_file(network: u16, dev: Option<u16>) -> PathBuf {
    if let Some(dev) = dev {
        PathBuf::from(format!(".current-proposal-cache-{network}-{dev}"))
    } else {
        PathBuf::from(format!("current-proposal-cache-{network}"))
    }
}

/// Tracks information about where the node-specfic configuration files are stored.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NodeDataDir {
    path: PathBuf,

    /// Is the node a development node?
    /// This will eventually be removed when the development storage format is no longer supported.
    dev: Option<u16>,
}

impl NodeDataDir {
    pub fn new(path: PathBuf) -> Self {
        Self { path, dev: None }
    }

    pub fn new_test(dev: Option<u16>) -> Self {
        if let Some(dev) = dev {
            Self { path: PathBuf::from(format!(".node-config-test-{dev}")), dev: Some(dev) }
        } else {
            Self { path: PathBuf::from(".node-config-test"), dev: None }
        }
    }

    pub fn new_development(dev: u16) -> Self {
        Self { path: PathBuf::from(format!(".node-config-{dev}")), dev: Some(dev) }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn is_dev(&self) -> bool {
        self.dev.is_some()
    }

    pub fn dev(&self) -> Option<u16> {
        self.dev
    }

    /// The location to store the previous peer cache.
    pub fn router_peer_cache_path(&self) -> PathBuf {
        self.path.join(ROUTER_PEER_CACHE_FILE)
    }

    pub fn gateway_peer_cache_path(&self) -> PathBuf {
        self.path.join(GATWAY_PEER_CACHE_FILE)
    }

    /// The location to store the current proposal cache.
    pub fn current_proposal_cache_path(&self) -> PathBuf {
        self.path.join(CURRENT_PROPOSAL_CACHE_FILE)
    }

    /// The location to store the JWT secret for a given address.
    pub fn jwt_secret_path<D: std::fmt::Display>(&self, address: &D) -> PathBuf {
        self.path.join(jwt_secret_file(address))
    }
}
