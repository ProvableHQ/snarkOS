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

use crate::NodeType;
use snarkvm::prelude::{Address, Network};

use std::{net::SocketAddr, time::Instant};

/// The class of a peer, indicating its trust level and origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeClass {
    /// A peer that was discovered through the network protocol.
    Discovered,
    /// A peer that was explicitly configured as trusted.
    Trusted,
    /// A peer that is a hardcoded bootstrap node for the network.
    Bootstrap,
}

/// A peer of any connection status.
#[derive(Clone)]
pub enum Peer<N: Network> {
    /// A candidate peer that's currently not connected to.
    Candidate(CandidatePeer),
    /// A peer that's currently being connected to (the handshake is in progress).
    Connecting(ConnectingPeer),
    /// A fully connected (post-handshake) peer.
    Connected(ConnectedPeer<N>),
}

/// A candidate peer.
#[derive(Clone)]
pub struct ConnectingPeer {
    /// The listening address of a connecting peer.
    pub listener_addr: SocketAddr,
    /// The class of the peer (trusted, bootstrap, or discovered).
    pub class: NodeClass,
}

/// A candidate peer.
#[derive(Clone)]
pub struct CandidatePeer {
    /// The listening address of a candidate peer.
    pub listener_addr: SocketAddr,
    /// The class of the peer (trusted, bootstrap, or discovered).
    pub class: NodeClass,
}

/// A fully connected peer.
#[derive(Clone)]
pub struct ConnectedPeer<N: Network> {
    /// The listener address of the peer.
    pub listener_addr: SocketAddr,
    /// The connected address of the peer.
    pub connected_addr: SocketAddr,
    /// The class of the peer (trusted, bootstrap, or discovered).
    pub class: NodeClass,
    /// The Aleo address of the peer.
    pub aleo_addr: Address<N>,
    /// The node type of the peer.
    pub node_type: NodeType,
    /// The message version of the peer.
    pub version: u32,
    /// The latest block height known to be associated with the peer.
    pub last_height_seen: Option<u32>,
    /// The timestamp of the first message received from the peer.
    pub first_seen: Instant,
    /// The timestamp of the last message received from this peer.
    pub last_seen: Instant,
}

impl CandidatePeer {
    /// Returns `true` if the peer is considered trusted.
    pub fn is_trusted(&self) -> bool {
        matches!(self.class, NodeClass::Trusted)
    }

    /// Returns `true` if the peer is a bootstrap peer.
    pub fn is_bootstrap(&self) -> bool {
        matches!(self.class, NodeClass::Bootstrap)
    }
}

impl<N: Network> ConnectedPeer<N> {
    /// Returns `true` if the peer is considered trusted.
    pub fn is_trusted(&self) -> bool {
        matches!(self.class, NodeClass::Trusted)
    }

    /// Returns `true` if the peer is a bootstrap peer.
    pub fn is_bootstrap(&self) -> bool {
        matches!(self.class, NodeClass::Bootstrap)
    }
}

impl<N: Network> Peer<N> {
    /// Create a candidate peer.
    pub const fn new_candidate(listener_addr: SocketAddr, class: NodeClass) -> Self {
        Self::Candidate(CandidatePeer { listener_addr, class })
    }

    /// Create a connecting peer.
    pub const fn new_connecting(listener_addr: SocketAddr, class: NodeClass) -> Self {
        Self::Connecting(ConnectingPeer { listener_addr, class })
    }

    /// Promote a connecting peer to a fully connected one.
    pub fn upgrade_to_connected(
        &mut self,
        connected_addr: SocketAddr,
        listener_port: u16,
        aleo_address: Address<N>,
        node_type: NodeType,
        node_version: u32,
    ) {
        // Logic check: this can only happen during the handshake.
        assert!(matches!(self, Self::Connecting(_)));

        let timestamp = Instant::now();
        let listener_addr = SocketAddr::from((connected_addr.ip(), listener_port));

        *self = Self::Connected(ConnectedPeer {
            listener_addr,
            connected_addr,
            aleo_addr: aleo_address,
            node_type,
            class: self.class(),
            version: node_version,
            last_height_seen: None,
            first_seen: timestamp,
            last_seen: timestamp,
        });
    }

    /// Demote a peer to candidate status, marking it as disconnected.
    pub fn downgrade_to_candidate(&mut self, listener_addr: SocketAddr) {
        *self = Self::new_candidate(listener_addr, self.class());
    }

    /// Returns the type of the node (only applicable to connected peers).
    pub fn node_type(&self) -> Option<NodeType> {
        match self {
            Self::Candidate(_) => None,
            Self::Connecting(_) => None,
            Self::Connected(peer) => Some(peer.node_type),
        }
    }

    /// The listener (public) address of this peer.
    pub fn listener_addr(&self) -> SocketAddr {
        match self {
            Self::Candidate(p) => p.listener_addr,
            Self::Connecting(p) => p.listener_addr,
            Self::Connected(p) => p.listener_addr,
        }
    }

    /// The listener (public) address of this peer.
    pub fn last_height_seen(&self) -> Option<u32> {
        match self {
            Self::Candidate(_) => None,
            Self::Connecting(_) => None,
            Self::Connected(peer) => peer.last_height_seen,
        }
    }

    /// Returns `true` if the peer is not connected or connecting.
    pub fn is_candidate(&self) -> bool {
        matches!(self, Peer::Candidate(_))
    }

    /// Returns `true` if the peer is currently undergoing the network handshake.
    pub fn is_connecting(&self) -> bool {
        matches!(self, Peer::Connecting(_))
    }

    /// Returns `true` if the peer has concluded the network handshake.
    pub fn is_connected(&self) -> bool {
        matches!(self, Peer::Connected(_))
    }

    /// Returns the class of the peer (either trusted, bootstrap, or discovered).
    pub fn class(&self) -> NodeClass {
        match self {
            Self::Candidate(peer) => peer.class,
            Self::Connecting(peer) => peer.class,
            Self::Connected(peer) => peer.class,
        }
    }

    /// Returns `true` if the peer is considered trusted.
    pub fn is_trusted(&self) -> bool {
        matches!(self.class(), NodeClass::Trusted)
    }

    /// Returns `true` if the peer is a bootstrap peer.
    pub fn is_bootstrap(&self) -> bool {
        matches!(self.class(), NodeClass::Bootstrap)
    }

    /// Updates the peer's `last_seen` timestamp.
    pub fn update_last_seen(&mut self) {
        if let Self::Connected(ConnectedPeer { last_seen, .. }) = self {
            *last_seen = Instant::now();
        }
    }
}
