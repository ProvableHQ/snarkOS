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

use crate::NodeType;
use snarkvm::prelude::{Address, Network};
use tracing::*;

use std::{fmt, net::SocketAddr, time::Instant};

/// The class of a peer, indicating its trust level and origin.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeClass {
    /// A peer that was explicitly configured as trusted.
    Trusted,
    /// A peer that is a hardcoded bootstrap node for the network.
    Bootstrap,
    /// A peer that was discovered through the network protocol.
    Discovered,
}

impl std::fmt::Display for NodeClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Trusted => write!(f, "trusted"),
            Self::Bootstrap => write!(f, "bootstrap"),
            Self::Discovered => write!(f, "discovered"),
        }
    }
}

/// A peer of any connection status.
#[derive(Clone, Debug)]
pub enum Peer<N: Network> {
    /// A candidate peer that's currently not connected to.
    Candidate(CandidatePeer),
    /// A peer that's currently being connected to (the handshake is in progress).
    Connecting(ConnectingPeer),
    /// A fully connected (post-handshake) peer.
    Connected(ConnectedPeer<N>),
}

/// A connecting peer.
#[derive(Clone, Debug)]
pub struct ConnectingPeer {
    /// The listening address of a connecting peer.
    pub listener_addr: SocketAddr,
    /// The class of the peer (trusted, bootstrap, or discovered).
    pub class: NodeClass,
}

/// A candidate peer.
#[derive(Clone, Debug)]
pub struct CandidatePeer {
    /// The listening address of a candidate peer.
    pub listener_addr: SocketAddr,
    /// The latest block height known to be associated with the peer.
    pub last_height_seen: Option<u32>,
    /// The last time we attempted to connect to the peer.
    /// `None` if there was no attempt to connect since the peer was last connected, or no attempt at all.
    pub last_connection_attempt: Option<Instant>,
    /// The total number of connection attempts, since the peer was last connected.
    pub total_connection_attempts: u32,
    /// The class of the peer (trusted, bootstrap, or discovered).
    pub class: NodeClass,
}

/// A fully connected peer.
#[derive(Clone, Debug)]
pub struct ConnectedPeer<N: Network> {
    /// The listener address of the peer.
    pub listener_addr: SocketAddr,
    /// The connected address of the peer.
    pub connected_addr: SocketAddr,
    /// Indicates whether this is a Router or a Gateway connection for the peer.
    pub connection_mode: ConnectionMode,
    /// The class of the peer (trusted, bootstrap, or discovered).
    pub class: NodeClass,
    /// The Aleo address of the peer.
    pub aleo_addr: Address<N>,
    /// The node type of the peer.
    pub node_type: NodeType,
    /// The message version of the peer.
    pub version: u32,
    /// The snarkOS commit hash of the peer.
    pub snarkos_sha: Option<[u8; 40]>,
    /// The latest block height known to be associated with the peer.
    pub last_height_seen: Option<u32>,
    /// The timestamp of the first message received from the peer.
    pub first_seen: Instant,
    /// The timestamp of the last message received from this peer.
    pub last_seen: Instant,
}

/// Indicates whether a peer is connected via the Gateway or the Router.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionMode {
    Router,
    Gateway,
}

impl CandidatePeer {
    /// Returns `true` if the peer is considered trusted.
    pub fn is_trusted(&self) -> bool {
        self.class == NodeClass::Trusted
    }
}

impl<N: Network> ConnectedPeer<N> {
    /// Returns `true` if the peer is considered trusted.
    pub fn is_trusted(&self) -> bool {
        self.class == NodeClass::Trusted
    }

    /// Returns `true` if the peer is a bootstrap peer.
    pub fn is_bootstrap(&self) -> bool {
        self.class == NodeClass::Bootstrap
    }
}

impl fmt::Display for ConnectionMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConnectionMode::Gateway => write!(f, "Gateway"),
            ConnectionMode::Router => write!(f, "Router"),
        }
    }
}

impl<N: Network> Peer<N> {
    /// Create a candidate peer.
    pub const fn new_candidate(listener_addr: SocketAddr, class: NodeClass) -> Self {
        Self::Candidate(CandidatePeer {
            listener_addr,
            class,
            last_height_seen: None,
            last_connection_attempt: None,
            total_connection_attempts: 0,
        })
    }

    /// Create a connecting peer.
    pub const fn new_connecting(listener_addr: SocketAddr, class: NodeClass) -> Self {
        Self::Connecting(ConnectingPeer { class, listener_addr })
    }

    /// Promote a connecting peer to a fully connected one.
    #[allow(clippy::too_many_arguments)]
    pub fn upgrade_to_connected(
        &mut self,
        connected_addr: SocketAddr,
        listener_port: u16,
        aleo_address: Address<N>,
        node_type: NodeType,
        node_version: u32,
        snarkos_sha: Option<[u8; 40]>,
        connection_mode: ConnectionMode,
    ) {
        let timestamp = Instant::now();
        let listener_addr = SocketAddr::from((connected_addr.ip(), listener_port));

        // Logic check: this can only happen during the handshake. This isn't a fatal
        // error, but should not be triggered.
        if !matches!(self, Self::Connecting(_)) {
            warn!("Peer '{listener_addr}' is being upgraded to Connected, but isn't Connecting");
        }

        *self = Self::Connected(ConnectedPeer {
            listener_addr,
            connected_addr,
            connection_mode,
            aleo_addr: aleo_address,
            node_type,
            class: self.class(),
            version: node_version,
            snarkos_sha,
            last_height_seen: None,
            first_seen: timestamp,
            last_seen: timestamp,
        });
    }

    /// Demote a peer to candidate status, marking it as disconnected.
    pub fn downgrade_to_candidate(&mut self, listener_addr: SocketAddr) {
        *self = Self::Candidate(CandidatePeer {
            listener_addr,
            class: self.class(),
            last_height_seen: self.last_height_seen(),
            last_connection_attempt: None,
            total_connection_attempts: 0,
        });
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

    /// Returns a reference to the underlying `ConnectedPeer` if it is connedcted,
    /// otherwise `None`.
    pub fn as_connected(&self) -> Option<&ConnectedPeer<N>> {
        match self {
            Self::Connected(peer) => Some(peer),
            _ => None,
        }
    }
}

impl<N: Network> ConnectedPeer<N> {
    /// Returns `true` if this peer is validator.
    pub fn is_validator(&self) -> bool {
        self.node_type == NodeType::Validator
    }
}
