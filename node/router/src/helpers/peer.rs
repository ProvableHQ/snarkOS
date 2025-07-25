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

use crate::{NodeType, Router, messages::ChallengeRequest};
use snarkvm::prelude::{Address, Network};

use std::{net::SocketAddr, time::Instant};

/// A peer of any connection status.
pub enum Peer<N: Network> {
    /// A candidate peer that's currently not connected to.
    Candidate(CandidatePeer),
    /// A peer that's currently being connected to (the handshake is in progress).
    Connecting,
    /// A fully connected (post-handshake) peer.
    Connected(ConnectedPeer<N>),
}

/// A candidate peer.
#[derive(Clone)]
pub struct CandidatePeer {
    /// The listening address of a candidate peer.
    pub listener_addr: SocketAddr,
    /// A restricted peer is one that has recently exceeded the `MAXIMUM_CONNECTION_FAILURES`,
    /// and is ineligible for connection attempts for `RADIO_SILENCE_IN_SECS`.
    // TODO: consider removing in favor of the ban feature for simplicity.
    pub restricted: Option<Instant>,
}

impl CandidatePeer {
    /// Restrict a candidate peer with the current timestamp.
    pub fn restrict(&mut self) {
        self.restricted = Some(Instant::now());
    }
}

/// A fully connected peer.
#[derive(Clone)]
pub struct ConnectedPeer<N: Network> {
    /// The listener address of the peer.
    pub listener_addr: SocketAddr,
    /// The connected address of the peer.
    pub connected_addr: SocketAddr,
    /// The Aleo address of the peer.
    pub aleo_addr: Address<N>,
    /// The node type of the peer.
    pub node_type: NodeType,
    /// The message version of the peer.
    pub version: u32,
    /// The timestamp of the first message received from the peer.
    pub first_seen: Instant,
    /// The timestamp of the last message received from this peer.
    pub last_seen: Instant,
    /// A reference to the associated `Router` object.
    pub router: Router<N>,
}

impl<N: Network> Peer<N> {
    /// Create a candidate peer.
    pub const fn new_candidate(listener_addr: SocketAddr) -> Self {
        Self::Candidate(CandidatePeer { listener_addr, restricted: None })
    }

    /// Promote a connecting peer to a fully connected one.
    pub fn upgrade_to_connected(&mut self, connected_addr: SocketAddr, cr: &ChallengeRequest<N>, router: Router<N>) {
        // Logic check: this can only happen during the handshake.
        assert!(matches!(self, Self::Connecting));

        let timestamp = Instant::now();
        let listener_addr = SocketAddr::from((connected_addr.ip(), cr.listener_port));

        // Introduce the peer in the resolver.
        router.resolver.write().insert_peer(listener_addr, connected_addr);

        *self = Self::Connected(ConnectedPeer {
            listener_addr,
            connected_addr,
            aleo_addr: cr.address,
            node_type: cr.node_type,
            version: cr.version,
            first_seen: timestamp,
            last_seen: timestamp,
            router,
        });
    }

    /// Demote a peer to candidate status, marking it as disconnected.
    pub fn downgrade_to_candidate(&mut self, listener_addr: SocketAddr, restrict: bool) {
        // Connecting peers are not in the resolver.
        if let Self::Connected(peer) = self {
            // Remove the peer from the resolver.
            peer.router.resolver.write().remove_peer(&peer.connected_addr);
        };

        let restricted = if restrict { Some(Instant::now()) } else { None };
        *self = Self::Candidate(CandidatePeer { listener_addr, restricted });
    }

    /// Returns the type of the node (only applicable to connected peers).
    pub fn node_type(&self) -> Option<NodeType> {
        match self {
            Self::Candidate(_) => None,
            Self::Connecting => None,
            Self::Connected(peer) => Some(peer.node_type),
        }
    }

    /// Returns `true` if the peer is currently undergoing the network handshake.
    pub fn is_connecting(&self) -> bool {
        matches!(self, Peer::Connecting)
    }

    /// Returns `true` if the peer has concluded the network handshake.
    pub fn is_connected(&self) -> bool {
        matches!(self, Peer::Connected(_))
    }

    /// Updates the peer's `last_seen` timestamp.
    pub fn update_last_seen(&mut self) {
        if let Self::Connected(ConnectedPeer { last_seen, .. }) = self {
            *last_seen = Instant::now();
        }
    }

    /// Updates the peer's version.
    pub fn update_version(&mut self, new_version: u32) {
        if let Self::Connected(ConnectedPeer { version, .. }) = self {
            *version = new_version;
        }
    }
}
