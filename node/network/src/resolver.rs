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

use snarkvm::prelude::{Address, Network};

use std::{collections::HashMap, net::SocketAddr};

/// The resolver contains additional reverse maps for peers which are not available
/// by default to the implementors of PeerPoolHandling (which already contains
/// maps from the peer's listening address to their various components).
#[derive(Debug)]
pub struct Resolver<N: Network> {
    /// The map of peers' connected addresses to the corresponding listener addresses.
    to_listener: HashMap<SocketAddr, SocketAddr>,
    /// A map of peers' Aleo addresses to the corresponding listener addresses.
    /// It is currently only used for the validators.
    address_peers: HashMap<Address<N>, SocketAddr>,
}

impl<N: Network> Default for Resolver<N> {
    /// Initializes a new instance of the resolver.
    fn default() -> Self {
        Self::new()
    }
}

impl<N: Network> Resolver<N> {
    /// Initializes a new instance of the resolver.
    pub fn new() -> Self {
        Self { to_listener: Default::default(), address_peers: Default::default() }
    }
}

impl<N: Network> Resolver<N> {
    /// Returns the listener address for the given connected peer address, if it exists.
    pub fn get_listener(&self, connected_addr: SocketAddr) -> Option<SocketAddr> {
        self.to_listener.get(&connected_addr).copied()
    }

    /// Returns the listener address for the peer with the given Aleo address.
    pub fn get_peer_ip_for_address(&self, aleo_addr: Address<N>) -> Option<SocketAddr> {
        self.address_peers.get(&aleo_addr).copied()
    }

    /// Inserts a mapping of a peer's connected address to its listener address,
    /// alongside an optional mapping of the Aleo address to the listener address.
    pub fn insert_peer(
        &mut self,
        listener_addr: SocketAddr,
        connected_addr: SocketAddr,
        aleo_addr: Option<Address<N>>,
    ) {
        self.to_listener.insert(connected_addr, listener_addr);
        if let Some(addr) = aleo_addr {
            self.address_peers.insert(addr, listener_addr);
        }
    }

    /// Removes the mapping of a peer's connected address to its listener address,
    /// alongside the optional mapping of the Aleo address to the listener address.
    pub fn remove_peer(&mut self, connected_addr: SocketAddr, aleo_addr: Option<Address<N>>) {
        self.to_listener.remove(&connected_addr);
        if let Some(addr) = aleo_addr {
            self.address_peers.remove(&addr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::{prelude::Rng, utilities::TestRng};

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    // Test the basic functionalities of the resolver.
    #[test]
    fn test_resolver() {
        let mut resolver = Resolver::<CurrentNetwork>::new();
        let listener_ip = SocketAddr::from(([127, 0, 0, 1], 1234));
        let peer_addr = SocketAddr::from(([127, 0, 0, 1], 4321));
        let mut rng = TestRng::default();
        let address = Address::<CurrentNetwork>::new(rng.random());

        assert!(resolver.get_listener(peer_addr).is_none());
        assert!(resolver.get_peer_ip_for_address(address).is_none());

        resolver.insert_peer(listener_ip, peer_addr, Some(address));

        assert_eq!(resolver.get_listener(peer_addr).unwrap(), listener_ip);
        assert_eq!(resolver.get_peer_ip_for_address(address).unwrap(), listener_ip);

        resolver.remove_peer(peer_addr, Some(address));

        assert!(resolver.get_listener(peer_addr).is_none());
        assert!(resolver.get_peer_ip_for_address(address).is_none());
    }
}
