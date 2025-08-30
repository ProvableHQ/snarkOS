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

use std::{collections::HashMap, net::SocketAddr};

/// The `Resolver` provides the means to map the connected address (used in the lower-level
/// `tcp` module internals, and provided by the OS) to the listener address (used as the
/// unique peer identifier in the higher-level functions).
#[derive(Debug, Default)]
pub(crate) struct Resolver {
    /// The map of the connected peer address to the correponding listener address.
    to_listener: HashMap<SocketAddr, SocketAddr>,
}

impl Resolver {
    /// Returns the listener address for the given connected address, if it exists.
    pub fn get_listener(&self, connected_addr: &SocketAddr) -> Option<SocketAddr> {
        self.to_listener.get(connected_addr).copied()
    }

    /// Inserts a new mapping of a connected address to the corresponding listener address.
    pub fn insert_peer(&mut self, listener_addr: SocketAddr, connected_addr: SocketAddr) {
        self.to_listener.insert(connected_addr, listener_addr);
    }

    /// Removes the given mapping.
    pub fn remove_peer(&mut self, connected_addr: &SocketAddr) {
        self.to_listener.remove(connected_addr);
    }
}
