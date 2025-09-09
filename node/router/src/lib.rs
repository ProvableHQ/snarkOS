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

#![forbid(unsafe_code)]

#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate tracing;

#[cfg(feature = "metrics")]
extern crate snarkos_node_metrics as metrics;

pub use snarkos_node_router_messages as messages;

mod handshake;

mod heartbeat;
pub use heartbeat::*;

mod helpers;
pub use helpers::*;

mod inbound;
pub use inbound::*;

mod outbound;
pub use outbound::*;

mod routing;
pub use routing::*;

mod writing;

use crate::messages::{Message, MessageCodec, NodeType};

use snarkos_account::Account;
use snarkos_node_bft_ledger_service::LedgerService;
use snarkos_node_tcp::{Config, ConnectionSide, P2P, Tcp, is_bogon_ip, is_unspecified_or_broadcast_ip};

use snarkvm::prelude::{Address, Network, PrivateKey, ViewKey};

use anyhow::{Result, bail};
#[cfg(feature = "locktick")]
use locktick::parking_lot::{Mutex, RwLock};
#[cfg(not(feature = "locktick"))]
use parking_lot::{Mutex, RwLock};
#[cfg(not(any(test)))]
use std::net::IpAddr;
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    net::SocketAddr,
    ops::Deref,
    str::FromStr,
    sync::Arc,
    time::Duration,
};
use tokio::task::JoinHandle;

/// The default port used by the router.
pub const DEFAULT_NODE_PORT: u16 = 4130;

/// The router keeps track of connected and connecting peers.
/// The actual network communication happens in Inbound/Outbound,
/// which is implemented by Validator, Prover, and Client.
#[derive(Clone)]
pub struct Router<N: Network>(Arc<InnerRouter<N>>);

impl<N: Network> Deref for Router<N> {
    type Target = Arc<InnerRouter<N>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct InnerRouter<N: Network> {
    /// The TCP stack.
    tcp: Tcp,
    /// The node type.
    node_type: NodeType,
    /// The account of the node.
    account: Account<N>,
    /// The ledger service.
    ledger: Arc<dyn LedgerService<N>>,
    /// The cache.
    cache: Cache<N>,
    /// The resolver.
    resolver: RwLock<Resolver>,
    /// The collection of both candidate and connected peers.
    peer_pool: RwLock<HashMap<SocketAddr, Peer<N>>>,
    /// The spawned handles.
    handles: Mutex<Vec<JoinHandle<()>>>,
    /// If the flag is set, the node will periodically evict more external peers.
    rotate_external_peers: bool,
    /// If the flag is set, the node will engage in P2P gossip to request more peers.
    allow_external_peers: bool,
    /// The boolean flag for the development mode.
    is_dev: bool,
}

impl<N: Network> Router<N> {
    /// The minimum permitted interval between connection attempts for an IP; anything shorter is considered malicious.
    #[cfg(not(test))]
    const CONNECTION_ATTEMPTS_SINCE_SECS: i64 = 10;
    /// The maximum number of candidate peers permitted to be stored in the node.
    const MAXIMUM_CANDIDATE_PEERS: usize = 10_000;
    /// The maximum amount of connection attempts within a 10 second threshold
    #[cfg(not(test))]
    const MAX_CONNECTION_ATTEMPTS: usize = 10;
    /// The duration after which a connected peer is considered inactive or
    /// disconnected if no message has been received in the meantime.
    const MAX_RADIO_SILENCE: Duration = Duration::from_secs(150); // 2.5 minutes
}

impl<N: Network> Router<N> {
    /// Initializes a new `Router` instance.
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        node_ip: SocketAddr,
        node_type: NodeType,
        account: Account<N>,
        ledger: Arc<dyn LedgerService<N>>,
        trusted_peers: &[SocketAddr],
        max_peers: u16,
        rotate_external_peers: bool,
        allow_external_peers: bool,
        is_dev: bool,
    ) -> Result<Self> {
        // Initialize the TCP stack.
        let tcp = Tcp::new(Config::new(node_ip, max_peers));

        let trusted_peers = trusted_peers
            .iter()
            .copied()
            .map(|addr| (addr, Peer::new_candidate(addr, true)))
            .collect::<HashMap<_, _>>();

        // Initialize the router.
        Ok(Self(Arc::new(InnerRouter {
            tcp,
            node_type,
            account,
            ledger,
            cache: Default::default(),
            resolver: Default::default(),
            peer_pool: RwLock::new(trusted_peers),
            handles: Default::default(),
            rotate_external_peers,
            allow_external_peers,
            is_dev,
        })))
    }
}

impl<N: Network> Router<N> {
    /// Attempts to connect to the given peer IP.
    ///
    /// Returns None if we are already connected to the peer or cannot connect.
    /// Otherwise, it returns a handle to the tokio tasks that sets up the connection.
    pub fn connect(&self, peer_ip: SocketAddr) -> Option<JoinHandle<bool>> {
        // Return early if the attempt is against the protocol rules.
        match self.check_connection_attempt(peer_ip) {
            Ok(true) => return None,
            Ok(false) => {}
            Err(forbidden_message) => {
                warn!("{forbidden_message}");
                return None;
            }
        }

        let router = self.clone();
        Some(tokio::spawn(async move {
            // Attempt to connect to the candidate peer.
            match router.tcp.connect(peer_ip).await {
                // Remove the peer from the candidate peers.
                Ok(()) => true,
                // If the connection was not allowed, log the error.
                Err(error) => {
                    warn!("Unable to connect to '{peer_ip}' - {error}");
                    false
                }
            }
        }))
    }

    /// Checks if we can and are allowed to connect to the given peer.
    ///
    /// # Return Values
    /// - `Ok(true)` if already connected (or connecting) to the peer.
    /// - `Ok(false)` if not connected to the peer but allowed to.
    /// - `Err(err)` if not allowed to connect to the peer.
    fn check_connection_attempt(&self, peer_ip: SocketAddr) -> Result<bool> {
        // Ensure the peer IP is not this node.
        if self.is_local_ip(&peer_ip) {
            bail!("Dropping connection attempt to '{peer_ip}' (attempted to self-connect)")
        }
        // Ensure the node does not surpass the maximum number of peer connections.
        if self.number_of_connected_peers() >= self.max_connected_peers() {
            bail!("Dropping connection attempt to '{peer_ip}' (maximum peers reached)")
        }
        // Ensure the node is not already connecting to this peer.
        if self.is_connecting(&peer_ip) {
            debug!("Dropping connection attempt to '{peer_ip}' (already connecting)");
            return Ok(true);
        }
        // Ensure the node is not already connected to this peer.
        if self.is_connected(&peer_ip) {
            debug!("Dropping connection attempt to '{peer_ip}' (already connected)");
            return Ok(true);
        }

        Ok(false)
    }

    /// Disconnects from the given peer IP, if the peer is connected.
    pub fn disconnect(&self, peer_ip: SocketAddr) -> JoinHandle<bool> {
        let router = self.clone();
        tokio::spawn(async move {
            if let Some(peer) = router.get_connected_peer(&peer_ip) {
                let connected_addr = peer.connected_addr;
                router.tcp.disconnect(connected_addr).await
            } else {
                false
            }
        })
    }

    /// Returns the IP address of this node.
    pub fn local_ip(&self) -> SocketAddr {
        self.tcp.listening_addr().expect("The TCP listener is not enabled")
    }

    /// Returns `true` if the given IP is this node.
    pub fn is_local_ip(&self, ip: &SocketAddr) -> bool {
        *ip == self.local_ip()
            || (ip.ip().is_unspecified() || ip.ip().is_loopback()) && ip.port() == self.local_ip().port()
    }

    /// Returns `true` if the given IP is not this node, is not a bogon address, and is not unspecified.
    pub fn is_valid_peer_ip(&self, ip: &SocketAddr) -> bool {
        !self.is_local_ip(ip) && !is_bogon_ip(ip.ip()) && !is_unspecified_or_broadcast_ip(ip.ip())
    }

    /// Returns `true` if the message version is valid.
    pub fn is_valid_message_version(&self, message_version: u32) -> bool {
        // Determine the minimum message version this node will accept, based on its role.
        // - Provers always operate at the latest message version.
        // - Validators and clients may accept older versions, depending on their current block height.
        let lowest_accepted_message_version = match self.node_type {
            // Provers should always use the latest version.
            NodeType::Prover => Message::<N>::latest_message_version(),
            // Validators and clients accept messages from lower version based on the migration height.
            NodeType::Validator | NodeType::Client => {
                Message::<N>::lowest_accepted_message_version(self.ledger.latest_block_height())
            }
        };

        // Check if the incoming message version is valid.
        message_version >= lowest_accepted_message_version
    }

    /// Returns the node type.
    pub fn node_type(&self) -> NodeType {
        self.node_type
    }

    /// Returns the account private key of the node.
    pub fn private_key(&self) -> &PrivateKey<N> {
        self.account.private_key()
    }

    /// Returns the account view key of the node.
    pub fn view_key(&self) -> &ViewKey<N> {
        self.account.view_key()
    }

    /// Returns the account address of the node.
    pub fn address(&self) -> Address<N> {
        self.account.address()
    }

    /// Returns `true` if the node is in development mode.
    pub fn is_dev(&self) -> bool {
        self.is_dev
    }

    /// Returns `true` if the node is periodically evicting more external peers.
    pub fn rotate_external_peers(&self) -> bool {
        self.rotate_external_peers
    }

    /// Returns `true` if the node is engaging in P2P gossip to request more peers.
    pub fn allow_external_peers(&self) -> bool {
        self.allow_external_peers
    }

    /// Returns the listener IP address from the (ambiguous) peer address.
    pub fn resolve_to_listener(&self, connected_addr: &SocketAddr) -> Option<SocketAddr> {
        self.resolver.read().get_listener(connected_addr)
    }

    /// Returns the (ambiguous) peer address from the listener IP address.
    pub fn resolve_to_ambiguous(&self, listener_addr: &SocketAddr) -> Option<SocketAddr> {
        if let Some(Peer::Connected(peer)) = self.peer_pool.read().get(listener_addr) {
            Some(peer.connected_addr)
        } else {
            None
        }
    }

    /// Returns `true` if the node is connecting to the given peer's listener address.
    pub fn is_connecting(&self, listener_addr: &SocketAddr) -> bool {
        self.peer_pool.read().get(listener_addr).is_some_and(|peer| peer.is_connecting())
    }

    /// Returns `true` if the node is connected to the given peer listener address.
    pub fn is_connected(&self, listener_addr: &SocketAddr) -> bool {
        self.peer_pool.read().get(listener_addr).is_some_and(|peer| peer.is_connected())
    }

    /// Returns `true` if the given listener address is trusted.
    pub fn is_trusted(&self, listener_addr: &SocketAddr) -> bool {
        self.peer_pool.read().get(listener_addr).is_some_and(|peer| peer.is_trusted())
    }

    /// Returns the maximum number of connected peers.
    pub fn max_connected_peers(&self) -> usize {
        self.tcp.config().max_connections as usize
    }

    /// Returns the number of connected peers.
    pub fn number_of_connected_peers(&self) -> usize {
        self.peer_pool.read().iter().filter(|(_, peer)| peer.is_connected()).count()
    }

    /// Returns the number of candidate peers.
    pub fn number_of_candidate_peers(&self) -> usize {
        self.peer_pool.read().values().filter(|peer| matches!(peer, Peer::Candidate(_))).count()
    }

    /// Returns the connected peer given the peer IP, if it exists.
    pub fn get_connected_peer(&self, listener_addr: &SocketAddr) -> Option<ConnectedPeer<N>> {
        if let Some(Peer::Connected(peer)) = self.peer_pool.read().get(listener_addr) {
            Some(peer.clone())
        } else {
            None
        }
    }

    /// Returns the list of all peers (connected, connecting, and candidate).
    pub fn get_peers(&self) -> Vec<Peer<N>> {
        self.peer_pool.read().values().cloned().collect()
    }

    /// Returns all connected peers.
    pub fn get_connected_peers(&self) -> Vec<ConnectedPeer<N>> {
        self.filter_connected_peers(|_| true)
    }

    /// Returns all connected peers that satisify the given predicate.
    pub fn filter_connected_peers<P: FnMut(&ConnectedPeer<N>) -> bool>(
        &self,
        mut predicate: P,
    ) -> Vec<ConnectedPeer<N>> {
        self.peer_pool
            .read()
            .values()
            .filter_map(|p| {
                if let Peer::Connected(peer) = p
                    && predicate(peer)
                {
                    Some(peer)
                } else {
                    None
                }
            })
            .cloned()
            .collect()
    }

    /// Returns the list of connected peers.
    pub fn connected_peers(&self) -> Vec<SocketAddr> {
        self.peer_pool.read().iter().filter_map(|(addr, peer)| peer.is_connected().then_some(*addr)).collect()
    }

    /// Returns the list of candidate peers.
    pub fn candidate_peers(&self) -> HashSet<SocketAddr> {
        let banned_ips = self.tcp().banned_peers().get_banned_ips();
        self.peer_pool
            .read()
            .iter()
            .filter_map(|(addr, peer)| {
                (matches!(peer, Peer::Candidate(_)) && !banned_ips.contains(&addr.ip())).then_some(*addr)
            })
            .collect()
    }

    /// Returns the list of unconnected trusted peers.
    pub fn unconnected_trusted_peers(&self) -> HashSet<SocketAddr> {
        self.peer_pool
            .read()
            .iter()
            .filter_map(
                |(addr, peer)| if let Peer::Candidate(peer) = peer { peer.trusted.then_some(*addr) } else { None },
            )
            .collect()
    }

    /// Returns the list of bootstrap peers.
    #[allow(clippy::if_same_then_else)]
    pub fn bootstrap_peers(&self) -> Vec<SocketAddr> {
        if cfg!(feature = "test") || self.is_dev {
            // Development testing contains no bootstrap peers.
            vec![]
        } else if N::ID == snarkvm::console::network::MainnetV0::ID {
            // Mainnet contains the following bootstrap peers.
            vec![
                SocketAddr::from_str("35.231.67.219:4130").unwrap(),
                SocketAddr::from_str("34.73.195.196:4130").unwrap(),
                SocketAddr::from_str("34.23.225.202:4130").unwrap(),
                SocketAddr::from_str("34.148.16.111:4130").unwrap(),
            ]
        } else if N::ID == snarkvm::console::network::TestnetV0::ID {
            // TestnetV0 contains the following bootstrap peers.
            vec![
                SocketAddr::from_str("34.138.104.159:4130").unwrap(),
                SocketAddr::from_str("35.231.46.237:4130").unwrap(),
                SocketAddr::from_str("34.148.251.155:4130").unwrap(),
                SocketAddr::from_str("35.190.141.234:4130").unwrap(),
            ]
        } else if N::ID == snarkvm::console::network::CanaryV0::ID {
            // CanaryV0 contains the following bootstrap peers.
            vec![
                SocketAddr::from_str("34.139.88.58:4130").unwrap(),
                SocketAddr::from_str("34.139.252.207:4130").unwrap(),
                SocketAddr::from_str("35.185.98.12:4130").unwrap(),
                SocketAddr::from_str("35.231.106.26:4130").unwrap(),
            ]
        } else {
            // Unrecognized networks contain no bootstrap peers.
            vec![]
        }
    }

    /// Check whether the given IP address is currently banned.
    #[cfg(not(any(test)))]
    fn is_ip_banned(&self, ip: IpAddr) -> bool {
        self.tcp.banned_peers().is_ip_banned(&ip)
    }

    /// Insert or update a banned IP.
    #[cfg(not(any(test)))]
    fn update_ip_ban(&self, ip: IpAddr) {
        self.tcp.banned_peers().update_ip_ban(ip);
    }

    /// Returns the list of metrics for the connected peers.
    pub fn connected_metrics(&self) -> Vec<(SocketAddr, NodeType)> {
        self.get_connected_peers().iter().map(|peer| (peer.listener_addr, peer.node_type)).collect()
    }

    #[cfg(feature = "metrics")]
    fn update_metrics(&self) {
        metrics::gauge(metrics::router::CONNECTED, self.number_of_connected_peers() as f64);
        metrics::gauge(metrics::router::CANDIDATE, self.number_of_candidate_peers() as f64);
    }

    /// Inserts the given peer IPs to the set of candidate peers.
    ///
    /// This method skips adding any given peers if the combined size exceeds the threshold,
    /// as the peer providing this list could be subverting the protocol.
    pub fn insert_candidate_peers(&self, peers: &[SocketAddr]) {
        // Compute the maximum number of candidate peers.
        let max_candidate_peers = Self::MAXIMUM_CANDIDATE_PEERS.saturating_sub(self.number_of_candidate_peers());
        {
            let mut peer_pool = self.peer_pool.write();
            // Ensure the combined number of peers does not surpass the threshold.
            let eligible_peers = peers
                .iter()
                .filter(|peer_ip| {
                    // Ensure the peer is not itself, and is not already known.
                    !self.is_local_ip(peer_ip) && !peer_pool.contains_key(peer_ip)
                })
                .take(max_candidate_peers)
                .map(|addr| (*addr, Peer::new_candidate(*addr, false)))
                .collect::<Vec<_>>();

            // Proceed to insert the eligible candidate peer IPs.
            peer_pool.extend(eligible_peers);
        }
        #[cfg(feature = "metrics")]
        self.update_metrics();
    }

    pub fn update_last_seen_for_connected_peer(&self, peer_ip: SocketAddr) {
        if let Some(peer) = self.peer_pool.write().get_mut(&peer_ip) {
            peer.update_last_seen();
        }
    }

    /// Removes the connected peer and adds them to the candidate peers.
    pub fn remove_connected_peer(&self, peer_ip: SocketAddr) {
        if let Some(peer) = self.peer_pool.write().get_mut(&peer_ip) {
            peer.downgrade_to_candidate(peer_ip);
        }
        // Clear cached entries applicable to the peer.
        self.cache.clear_peer_entries(peer_ip);
        #[cfg(feature = "metrics")]
        self.update_metrics();
    }

    /// Spawns a task with the given future; it should only be used for long-running tasks.
    pub fn spawn<T: Future<Output = ()> + Send + 'static>(&self, future: T) {
        self.handles.lock().push(tokio::spawn(future));
    }

    /// Shuts down the router.
    pub async fn shut_down(&self) {
        info!("Shutting down the router...");
        // Abort the tasks.
        self.handles.lock().iter().for_each(|handle| handle.abort());
        // Close the listener.
        self.tcp.shut_down().await;
    }
}
