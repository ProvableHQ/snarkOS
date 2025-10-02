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

use snarkvm::prelude::{Address, Network};

use std::{collections::HashMap, net::SocketAddr};

/// The resolver contains some reverse maps for peers which are not available
/// by default to the implementors of PeerPoolHandling (who already contain
/// maps from the peer's listening address to their various components).
#[derive(Debug)]
pub struct Resolver<N: Network> {
    /// The map of the (ambiguous) peer address to listener address.
    to_listener: HashMap<SocketAddr, SocketAddr>,
    /// A map of `address` to `peer IP`.
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
    /// Returns the listener address for the given (ambiguous) peer address, if it exists.
    pub fn get_listener(&self, peer_addr: SocketAddr) -> Option<SocketAddr> {
        self.to_listener.get(&peer_addr).copied()
    }

    /// Returns the peer IP for the given address.
    pub fn get_peer_ip_for_address(&self, address: Address<N>) -> Option<SocketAddr> {
        self.address_peers.get(&address).copied()
    }

    /// Inserts a mapping of a peer's connected address to its listener address,
    /// alongside a mapping of the Aleo address to the listener address.
    pub fn insert_peer(&mut self, listener_ip: SocketAddr, peer_addr: SocketAddr, address: Address<N>) {
        self.to_listener.insert(peer_addr, listener_ip);
        self.address_peers.insert(address, listener_ip);
    }

    /// Removes the mapping of a peer's connected address to its listener address,
    /// alongside the mapping of the Aleo address to the listener address.
    pub fn remove_peer(&mut self, connected_addr: SocketAddr, aleo_addr: Address<N>) {
        self.to_listener.remove(&connected_addr);
        self.address_peers.remove(&aleo_addr);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::{prelude::Rng, utilities::TestRng};

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    #[test]
    fn test_resolver() {
        let mut resolver = Resolver::<CurrentNetwork>::new();
        let listener_ip = SocketAddr::from(([127, 0, 0, 1], 1234));
        let peer_addr = SocketAddr::from(([127, 0, 0, 1], 4321));
        let mut rng = TestRng::default();
        let address = Address::<CurrentNetwork>::new(rng.r#gen());

        assert!(resolver.get_listener(peer_addr).is_none());
        assert!(resolver.get_peer_ip_for_address(address).is_none());

        resolver.insert_peer(listener_ip, peer_addr, address);

        assert_eq!(resolver.get_listener(peer_addr).unwrap(), listener_ip);
        assert_eq!(resolver.get_peer_ip_for_address(address).unwrap(), listener_ip);

        resolver.remove_peer(peer_addr, address);

        assert!(resolver.get_listener(peer_addr).is_none());
        assert!(resolver.get_peer_ip_for_address(address).is_none());
    }
}
