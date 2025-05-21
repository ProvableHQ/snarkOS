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

use std::{collections::HashMap, net::IpAddr, time::Instant};

#[cfg(feature = "locktick")]
use locktick::parking_lot::RwLock;
#[cfg(not(feature = "locktick"))]
use parking_lot::RwLock;

/// Contains the ban details for a banned peer.
#[derive(Clone)]
pub struct BanDetails {
    /// The time when the ban was created.
    banned_at: Instant,
}

impl BanDetails {
    /// Creates a new ban at the given time.
    pub fn new() -> Self {
        Self { banned_at: Instant::now() }
    }
}

impl Default for BanDetails {
    fn default() -> Self {
        Self::new()
    }
}

/// Contains the set of peers currently banned by IP.
#[derive(Default)]
pub struct BannedPeers(RwLock<HashMap<IpAddr, BanDetails>>);

impl BannedPeers {
    /// Check whether the given IP address is currently banned.
    pub fn is_ip_banned(&self, ip: &IpAddr) -> bool {
        self.0.read().contains_key(ip)
    }

    /// Get all banned IPs.
    pub fn get_banned_ips(&self) -> Vec<IpAddr> {
        self.0.read().keys().cloned().collect()
    }

    /// Get ban config for the given IP address.
    pub fn get_ban_config(&self, ip: IpAddr) -> Option<BanDetails> {
        self.0.read().get(&ip).cloned()
    }

    /// Insert or update a banned IP.
    pub fn update_ip_ban(&self, ip: IpAddr) {
        self.0.write().insert(ip, BanDetails::default());
    }

    /// Remove the expired entries
    pub fn remove_old_bans(&self, ban_time_in_secs: u64) {
        self.0.write().retain(|_, ban_config| ban_config.banned_at.elapsed().as_secs() < ban_time_in_secs);
    }
}
