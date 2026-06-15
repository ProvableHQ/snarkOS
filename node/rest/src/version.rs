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
use serde::Serialize;
use std::sync::OnceLock;

use snarkvm::prelude::ConsensusVersion;

// Include the generated build information
mod built_info {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

// Cache for version info to avoid repeated string allocations
static VERSION_INFO: OnceLock<VersionInfo> = OnceLock::new();

#[derive(Clone, Debug, Serialize)]
pub struct VersionInfo {
    /// The version from Cargo.toml
    pub version: String,
    /// Git commit hash
    pub git_commit: String,
    /// Git branch name
    pub git_branch: String,
    /// The latest consensus version in the moment
    pub latest_consensus_version: u16,
    /// A list of all the consensus heights
    pub consensus_heights: Vec<u32>,
}

impl VersionInfo {
    /// Get the cached version information
    pub fn get<N: Network>() -> &'static VersionInfo {
        let latest = ConsensusVersion::latest();
        let latest_num: u16 = latest as u16;

        let consensus_heights: Vec<u32> = N::CONSENSUS_VERSION_HEIGHTS().iter().map(|(_, height)| *height).collect();

        VERSION_INFO.get_or_init(|| VersionInfo {
            version: env!("SNARKOS_VERSION").to_string(),
            git_commit: built_info::GIT_COMMIT_HASH.unwrap_or("unknown").to_string(),
            git_branch: built_info::GIT_HEAD_REF.unwrap_or("unknown").to_string(),
            latest_consensus_version: latest_num,
            consensus_heights,
        })
    }
}
