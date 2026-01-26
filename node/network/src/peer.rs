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

use std::{net::SocketAddr, time::Instant};

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
    /// Indicates whether the peer is considered trusted.
    pub trusted: bool,
}

/// A candidate peer.
#[derive(Clone, Debug)]
pub struct CandidatePeer {
    /// The listening address of a candidate peer.
    pub listener_addr: SocketAddr,
    /// Indicates whether the peer is considered trusted.
    pub trusted: bool,
    /// The latest block height known to be associated with the peer.
    pub last_height_seen: Option<u32>,
    /// The last time we attempted to connect to the peer.
    /// `None` if there was no attempt to connect since the peer was last connected, or no attempt at all.
    pub last_connection_attempt: Option<Instant>,
    /// The total number of connection attempts, since the peer was last connected.
    pub total_connection_attempts: u32,
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
    /// Indicates whether the peer is considered trusted.
    pub trusted: bool,
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

/// Indicates whether a peer is connected via the Gateway or the Router.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionMode {
    Gateway,
    Router,
}

impl<N: Network> Peer<N> {
    /// Create a candidate peer.
    pub const fn new_candidate(listener_addr: SocketAddr, trusted: bool) -> Self {
        Self::Candidate(CandidatePeer {
            listener_addr,
            trusted,
            last_height_seen: None,
            last_connection_attempt: None,
            total_connection_attempts: 0,
        })
    }

    /// Create a connecting peer.
    pub const fn new_connecting(listener_addr: SocketAddr, trusted: bool) -> Self {
        Self::Connecting(ConnectingPeer { listener_addr, trusted })
    }

    /// Promote a connecting peer to a fully connected one.
    pub fn upgrade_to_connected(
        &mut self,
        connected_addr: SocketAddr,
        listener_port: u16,
        aleo_address: Address<N>,
        node_type: NodeType,
        node_version: u32,
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
            trusted: self.is_trusted(),
            version: node_version,
            last_height_seen: None,
            first_seen: timestamp,
            last_seen: timestamp,
        });
    }

    /// Demote a peer to candidate status, marking it as disconnected.
    pub fn downgrade_to_candidate(&mut self, listener_addr: SocketAddr) {
        *self = Self::Candidate(CandidatePeer {
            listener_addr,
            trusted: self.is_trusted(),
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

    /// Returns `true` if the peer is considered trusted.
    pub fn is_trusted(&self) -> bool {
        match self {
            Self::Candidate(peer) => peer.trusted,
            Self::Connecting(peer) => peer.trusted,
            Self::Connected(peer) => peer.trusted,
        }
    }

    /// Updates the peer's `last_seen` timestamp.
    pub fn update_last_seen(&mut self) {
        if let Self::Connected(ConnectedPeer { last_seen, .. }) = self {
            *last_seen = Instant::now();
        }
    }
}
