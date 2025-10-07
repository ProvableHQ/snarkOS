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

pub use crate::messages::NodeType;
use crate::messages::{BlockRequest, Message, MessageCodec};

use snarkos_account::Account;
use snarkos_node_bft_ledger_service::LedgerService;
use snarkos_node_sync_communication_service::CommunicationService;
use snarkos_node_tcp::{Config, ConnectionSide, P2P, Tcp, is_bogon_ip, is_unspecified_or_broadcast_ip};

use snarkvm::prelude::{Address, Network, PrivateKey, ViewKey};

use aleo_std::{StorageMode, aleo_ledger_dir};
use anyhow::{Result, bail};
#[cfg(feature = "locktick")]
use locktick::parking_lot::{Mutex, RwLock};
#[cfg(not(feature = "locktick"))]
use parking_lot::{Mutex, RwLock};
use std::{
    cmp,
    collections::{HashMap, HashSet, hash_map::Entry},
    fs,
    future::Future,
    io::{self, Write},
    net::{IpAddr, SocketAddr},
    ops::Deref,
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::task::JoinHandle;

/// The default port used by the router.
pub const DEFAULT_NODE_PORT: u16 = 4130;

/// The name of the file containing cached peers.
const PEER_CACHE_FILENAME: &str = "cached_router_peers";

pub trait PeerPoolHandling<N: Network>: P2P {
    const OWNER: &str;

    /// The maximum number of peers permitted to be stored in the peer pool.
    const MAXIMUM_POOL_SIZE: usize;

    /// The number of candidate peers to be removed from the pool once `MAXIMUM_POOL_SIZE` is reached.
    /// It must be lower than `MAXIMUM_POOL_SIZE`.
    const PEER_SLASHING_COUNT: usize;

    fn peer_pool(&self) -> &RwLock<HashMap<SocketAddr, Peer<N>>>;

    fn resolver(&self) -> &RwLock<Resolver<N>>;

    /// Returns `true` if the owning node is in development mode.
    fn is_dev(&self) -> bool;

    /// Returns the listener address of this node.
    fn local_ip(&self) -> SocketAddr {
        self.tcp().listening_addr().expect("The TCP listener is not enabled")
    }

    /// Returns `true` if the given IP is this node.
    fn is_local_ip(&self, addr: SocketAddr) -> bool {
        addr == self.local_ip()
            || (addr.ip().is_unspecified() || addr.ip().is_loopback()) && addr.port() == self.local_ip().port()
    }

    /// Returns `true` if the given IP is not this node, is not a bogon address, and is not unspecified.
    fn is_valid_peer_ip(&self, ip: SocketAddr) -> bool {
        !self.is_local_ip(ip) && !is_bogon_ip(ip.ip()) && !is_unspecified_or_broadcast_ip(ip.ip())
    }

    /// Returns the maximum number of connected peers.
    fn max_connected_peers(&self) -> usize {
        self.tcp().config().max_connections as usize
    }

    /// Ensure we are allowed to connect to the given listener address of a peer.
    ///
    /// # Return Values
    /// - `Ok(true)` if already connected (or connecting) to the peer.
    /// - `Ok(false)` if not connected to the peer but allowed to.
    /// - `Err(err)` if not allowed to connect to the peer.
    fn check_connection_attempt(&self, listener_addr: SocketAddr) -> Result<bool> {
        // Ensure the peer IP is not this node.
        if self.is_local_ip(listener_addr) {
            bail!("{} Dropping connection attempt to '{listener_addr}' (attempted to self-connect)", Self::OWNER);
        }
        // Ensure the node does not surpass the maximum number of peer connections.
        if self.number_of_connected_peers() >= self.max_connected_peers() {
            bail!("{} Dropping connection attempt to '{listener_addr}' (maximum peers reached)", Self::OWNER);
        }
        // Ensure the node is not already connected to this peer.
        if self.is_connected(listener_addr) {
            debug!("{} Dropping connection attempt to '{listener_addr}' (already connected)", Self::OWNER);
            return Ok(true);
        }
        // Ensure the node is not already connecting to this peer.
        if self.is_connecting(listener_addr) {
            debug!("{} Dropping connection attempt to '{listener_addr}' (already connecting)", Self::OWNER);
            return Ok(true);
        }
        // If the IP is already banned, reject the attempt.
        if self.is_ip_banned(listener_addr.ip()) {
            bail!("{} Rejected a connection attempt to a banned IP '{}'", Self::OWNER, listener_addr.ip());
        }
        Ok(false)
    }

    /// Attempts to connect to the given peer's listener address.
    ///
    /// Returns None if we are already connected to the peer or cannot connect.
    /// Otherwise, it returns a handle to the tokio tasks that sets up the connection.
    fn connect(&self, listener_addr: SocketAddr) -> Option<JoinHandle<bool>> {
        // Return early if the attempt is against the protocol rules.
        match self.check_connection_attempt(listener_addr) {
            Ok(true) => return None,
            Ok(false) => {}
            Err(error) => {
                warn!("{} {error}", Self::OWNER);
                return None;
            }
        }

        // Determine whether the peer is trusted or a bootstrap node in order to decide
        // how problematic any potential connection issues are.
        let is_trusted_or_bootstrap =
            self.is_trusted(listener_addr) || bootstrap_peers::<N>(false).contains(&listener_addr);

        let tcp = self.tcp().clone();
        Some(tokio::spawn(async move {
            debug!("{} Connecting to {listener_addr}...", Self::OWNER);
            // Attempt to connect to the peer.
            match tcp.connect(listener_addr).await {
                Ok(_) => true,
                Err(error) => {
                    if is_trusted_or_bootstrap {
                        warn!("{} Unable to connect to '{listener_addr}' - {error}", Self::OWNER);
                    } else {
                        debug!("{} Unable to connect to '{listener_addr}' - {error}", Self::OWNER);
                    }
                    false
                }
            }
        }))
    }

    /// Disconnects from the given peer IP, if the peer is connected. The returned boolean
    /// indicates whether the peer was actually disconnected from, or if this was a noop.
    fn disconnect(&self, listener_addr: SocketAddr) -> JoinHandle<bool> {
        if let Some(connected_addr) = self.resolve_to_ambiguous(listener_addr) {
            let tcp = self.tcp().clone();
            tokio::spawn(async move { tcp.disconnect(connected_addr).await })
        } else {
            tokio::spawn(async { false })
        }
    }

    /// Downgrades a connected peer to candidate status.
    fn downgrade_peer_to_candidate(&self, listener_addr: SocketAddr) {
        if let Some(peer) = self.peer_pool().write().get_mut(&listener_addr) {
            if let Peer::Connected(peer) = peer {
                // Only validators get their aleo address registered with the resolver.
                let aleo_addr = if peer.node_type == NodeType::Validator { Some(peer.aleo_addr) } else { None };
                self.resolver().write().remove_peer(peer.connected_addr, aleo_addr);
            }
            peer.downgrade_to_candidate(listener_addr);
        }
    }

    /// Adds new candidate peers to the peer pool, ensuring their validity and following the
    /// limit on the number of peers in the pool. The listener addresses may be paired with
    /// the last known block height of the associated peer.
    fn insert_candidate_peers(&self, mut listener_addrs: Vec<(SocketAddr, Option<u32>)>) {
        // Hold a write guard from now on, so as not to accidentally slash multiple times
        // based on multiple batches of candidate peers, and to not overwrite any entries.
        let mut peer_pool = self.peer_pool().write();

        // Perform filtering to ensure candidate validity. Also count how many entries are updates.
        let mut num_updates: usize = 0;
        listener_addrs.retain(|&(addr, height)| {
            !self.is_ip_banned(addr.ip())
                && if self.is_dev() { !is_bogon_ip(addr.ip()) } else { self.is_valid_peer_ip(addr) }
                && peer_pool
                    .get(&addr)
                    .map(|peer| peer.is_candidate() && height.is_some())
                    .inspect(|is_valid_update| {
                        if *is_valid_update {
                            num_updates += 1
                        }
                    })
                    .unwrap_or(true)
        });

        // If we've managed to filter out every entry, there's nothing to do.
        if listener_addrs.is_empty() {
            return;
        }

        // If we're about to exceed the peer pool size limit, apply candidate slashing.
        if self.number_of_peers() + listener_addrs.len() - num_updates >= Self::MAXIMUM_POOL_SIZE
            && Self::PEER_SLASHING_COUNT != 0
        {
            // Collect the addresses of prospect peers.
            let mut peers_to_slash = peer_pool
                .iter()
                .filter_map(|(addr, peer)| (matches!(peer, Peer::Candidate(_))).then_some(*addr))
                .collect::<Vec<_>>();

            // Get the low-level peer stats.
            let known_peers = self.tcp().known_peers().snapshot();

            // Sort the list of candidate peers by failure count (descending) and timestamp (ascending).
            let default_value = (0, Instant::now());
            peers_to_slash.sort_unstable_by_key(|addr| {
                let (num_failures, last_seen) = known_peers
                    .get(&addr.ip())
                    .map(|stats| (stats.failures(), stats.timestamp()))
                    .unwrap_or(default_value);
                (cmp::Reverse(num_failures), last_seen)
            });

            // Retain the candidate peers with the most failures and oldest timestamps.
            peers_to_slash.truncate(Self::PEER_SLASHING_COUNT);

            // Remove the peers to slash from the pool.
            peer_pool.retain(|addr, _| !peers_to_slash.contains(addr));
        }

        // Make sure that we won't breach the pool size limit in case the slashing didn't suffice.
        listener_addrs.truncate(Self::MAXIMUM_POOL_SIZE.saturating_sub(self.number_of_peers()));

        // If we've managed to truncate to 0, exit.
        if listener_addrs.is_empty() {
            return;
        }

        // Insert or update the applicable candidate peers.
        for (addr, height) in listener_addrs {
            match peer_pool.entry(addr) {
                Entry::Vacant(entry) => {
                    entry.insert(Peer::new_candidate(addr, false));
                }
                Entry::Occupied(mut entry) => {
                    if let Peer::Candidate(peer) = entry.get_mut() {
                        peer.last_height_seen = height;
                    }
                }
            }
        }
    }

    /// Completely removes an entry from the peer pool.
    fn remove_peer(&self, listener_addr: SocketAddr) {
        self.peer_pool().write().remove(&listener_addr);
    }

    /// Returns the connected peer address from the listener IP address.
    fn resolve_to_ambiguous(&self, listener_addr: SocketAddr) -> Option<SocketAddr> {
        if let Some(Peer::Connected(peer)) = self.peer_pool().read().get(&listener_addr) {
            Some(peer.connected_addr)
        } else {
            None
        }
    }

    /// Returns the connected peer aleo address from the listener IP address.
    fn resolve_to_aleo_addr(&self, listener_addr: SocketAddr) -> Option<Address<N>> {
        if let Some(Peer::Connected(peer)) = self.peer_pool().read().get(&listener_addr) {
            Some(peer.aleo_addr)
        } else {
            None
        }
    }

    /// Returns `true` if the node is connecting to the given peer's listener address.
    fn is_connecting(&self, listener_addr: SocketAddr) -> bool {
        self.peer_pool().read().get(&listener_addr).is_some_and(|peer| peer.is_connecting())
    }

    /// Returns `true` if the node is connected to the given peer listener address.
    fn is_connected(&self, listener_addr: SocketAddr) -> bool {
        self.peer_pool().read().get(&listener_addr).is_some_and(|peer| peer.is_connected())
    }

    /// Returns `true` if the given listener address is trusted.
    fn is_trusted(&self, listener_addr: SocketAddr) -> bool {
        self.peer_pool().read().get(&listener_addr).is_some_and(|peer| peer.is_trusted())
    }

    /// Returns the number of all peers.
    fn number_of_peers(&self) -> usize {
        self.peer_pool().read().len()
    }

    /// Returns the number of connected peers.
    fn number_of_connected_peers(&self) -> usize {
        self.peer_pool().read().iter().filter(|(_, peer)| peer.is_connected()).count()
    }

    /// Returns the number of connecting peers.
    fn number_of_connecting_peers(&self) -> usize {
        self.peer_pool().read().iter().filter(|(_, peer)| peer.is_connecting()).count()
    }

    /// Returns the number of candidate peers.
    fn number_of_candidate_peers(&self) -> usize {
        self.peer_pool().read().values().filter(|peer| matches!(peer, Peer::Candidate(_))).count()
    }

    /// Returns the connected peer given the peer IP, if it exists.
    fn get_connected_peer(&self, listener_addr: SocketAddr) -> Option<ConnectedPeer<N>> {
        if let Some(Peer::Connected(peer)) = self.peer_pool().read().get(&listener_addr) {
            Some(peer.clone())
        } else {
            None
        }
    }

    /// Updates the connected peer - if it exists -  given the peer IP and a closure.
    /// The returned status indicates whether the update was successful, i.e. the peer had existed.
    fn update_connected_peer<F: FnMut(&mut ConnectedPeer<N>)>(
        &self,
        listener_addr: &SocketAddr,
        mut update_fn: F,
    ) -> bool {
        if let Some(Peer::Connected(peer)) = self.peer_pool().write().get_mut(listener_addr) {
            update_fn(peer);
            true
        } else {
            false
        }
    }

    /// Returns the list of all peers (connected, connecting, and candidate).
    fn get_peers(&self) -> Vec<Peer<N>> {
        self.peer_pool().read().values().cloned().collect()
    }

    /// Returns all connected peers.
    fn get_connected_peers(&self) -> Vec<ConnectedPeer<N>> {
        self.filter_connected_peers(|_| true)
    }

    /// Returns all connected peers that satisify the given predicate.
    fn filter_connected_peers<P: FnMut(&ConnectedPeer<N>) -> bool>(&self, mut predicate: P) -> Vec<ConnectedPeer<N>> {
        self.peer_pool()
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
    fn connected_peers(&self) -> Vec<SocketAddr> {
        self.peer_pool().read().iter().filter_map(|(addr, peer)| peer.is_connected().then_some(*addr)).collect()
    }

    /// Returns the list of trusted peers.
    fn trusted_peers(&self) -> Vec<SocketAddr> {
        self.peer_pool().read().iter().filter_map(|(addr, peer)| peer.is_trusted().then_some(*addr)).collect()
    }

    /// Returns the list of candidate peers.
    fn get_candidate_peers(&self) -> Vec<CandidatePeer> {
        self.peer_pool()
            .read()
            .values()
            .filter_map(|peer| if let Peer::Candidate(peer) = peer { Some(peer.clone()) } else { None })
            .collect()
    }

    /// Returns the list of unconnected trusted peers.
    fn unconnected_trusted_peers(&self) -> HashSet<SocketAddr> {
        self.peer_pool()
            .read()
            .iter()
            .filter_map(
                |(addr, peer)| if let Peer::Candidate(peer) = peer { peer.trusted.then_some(*addr) } else { None },
            )
            .collect()
    }

    /// Loads any previously cached peer addresses so they can be introduced as initial
    /// candidate peers to connect to.
    fn load_cached_peers(storage_mode: &StorageMode, filename: &str) -> Result<Vec<SocketAddr>> {
        let mut peer_cache_path = aleo_ledger_dir(N::ID, storage_mode);
        peer_cache_path.push(filename);

        let peers = match fs::read_to_string(&peer_cache_path) {
            Ok(cached_peers_str) => {
                let mut cached_peers = Vec::new();
                for peer_addr_str in cached_peers_str.lines() {
                    match SocketAddr::from_str(peer_addr_str) {
                        Ok(addr) => cached_peers.push(addr),
                        Err(error) => warn!("Couldn't parse the cached peer address '{peer_addr_str}': {error}"),
                    }
                }
                cached_peers
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                // Not an issue - the cache may not exist yet.
                Vec::new()
            }
            Err(error) => {
                warn!("{} Couldn't load cached peers at {}: {error}", Self::OWNER, peer_cache_path.display());
                Vec::new()
            }
        };

        Ok(peers)
    }

    /// Preserve the peers who have the greatest known block heights, and the lowest
    /// number of registered network failures.
    fn save_best_peers(&self, storage_mode: &StorageMode, filename: &str, max_entries: Option<usize>) -> Result<()> {
        // Collect all prospect peers.
        let mut peers = self.get_peers();

        // Get the low-level peer stats.
        let known_peers = self.tcp().known_peers().snapshot();

        // Sort the list of peers.
        peers.sort_unstable_by_key(|peer| {
            if let Some(peer_stats) = known_peers.get(&peer.listener_addr().ip()) {
                // Prioritize greatest height, then lowest failure count.
                (cmp::Reverse(peer.last_height_seen()), peer_stats.failures())
            } else {
                // Unreachable; use an else-compatible dummy.
                (cmp::Reverse(peer.last_height_seen()), 0)
            }
        });
        if let Some(max) = max_entries {
            peers.truncate(max);
        }

        // Dump the connected peers to a file.
        let mut path = aleo_ledger_dir(N::ID, storage_mode);
        path.push(filename);
        let mut file = fs::File::create(path)?;
        for peer in peers {
            writeln!(file, "{}", peer.listener_addr())?;
        }

        Ok(())
    }

    // Introduces a new connecting peer into the peer pool if unknown, or promotes
    // a known candidate peer to a connecting one. The returned boolean indicates
    // whether the peer has been added/promoted, or rejected due to already being
    // shaken hands with or connected.
    fn add_connecting_peer(&self, listener_addr: SocketAddr) -> bool {
        match self.peer_pool().write().entry(listener_addr) {
            Entry::Vacant(entry) => {
                entry.insert(Peer::new_connecting(listener_addr, false));
                true
            }
            Entry::Occupied(mut entry) if matches!(entry.get(), Peer::Candidate(_)) => {
                entry.insert(Peer::new_connecting(listener_addr, entry.get().is_trusted()));
                true
            }
            Entry::Occupied(_) => false,
        }
    }

    /// Temporarily IP-ban and disconnect from the peer with the given listener address and an
    /// optional reason for the ban. This also removes the peer from the candidate pool.
    fn ip_ban_peer(&self, listener_addr: SocketAddr, reason: Option<&str>) {
        let ip = listener_addr.ip();
        debug!("IP-banning {ip}{}", reason.map(|r| format!(" reason: {r}")).unwrap_or_default());

        // Insert/update the low-level IP ban list.
        self.tcp().banned_peers().update_ip_ban(ip);

        // Disconnect from the peer.
        self.disconnect(listener_addr);
        // Remove the peer from the pool.
        self.remove_peer(listener_addr);
    }

    /// Check whether the given IP address is currently banned.
    fn is_ip_banned(&self, ip: IpAddr) -> bool {
        self.tcp().banned_peers().is_ip_banned(&ip)
    }

    /// Insert or update a banned IP.
    fn update_ip_ban(&self, ip: IpAddr) {
        self.tcp().banned_peers().update_ip_ban(ip);
    }
}

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

impl<N: Network> PeerPoolHandling<N> for Router<N> {
    const MAXIMUM_POOL_SIZE: usize = 10_000;
    const OWNER: &str = "[Router]";
    const PEER_SLASHING_COUNT: usize = 200;

    fn peer_pool(&self) -> &RwLock<HashMap<SocketAddr, Peer<N>>> {
        &self.peer_pool
    }

    fn resolver(&self) -> &RwLock<Resolver<N>> {
        &self.resolver
    }

    fn is_dev(&self) -> bool {
        self.is_dev
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
    resolver: RwLock<Resolver<N>>,
    /// The collection of both candidate and connected peers.
    peer_pool: RwLock<HashMap<SocketAddr, Peer<N>>>,
    /// The spawned handles.
    handles: Mutex<Vec<JoinHandle<()>>>,
    /// If the flag is set, the node will periodically evict more external peers.
    rotate_external_peers: bool,
    /// If the flag is set, the node will engage in P2P gossip to request more peers.
    allow_external_peers: bool,
    /// The storage mode.
    storage_mode: StorageMode,
    /// The boolean flag for the development mode.
    is_dev: bool,
}

impl<N: Network> Router<N> {
    /// The minimum permitted interval between connection attempts for an IP; anything shorter is considered malicious.
    #[cfg(not(feature = "test"))]
    const CONNECTION_ATTEMPTS_SINCE_SECS: i64 = 10;
    /// The maximum amount of connection attempts within a 10 second threshold
    #[cfg(not(feature = "test"))]
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
        storage_mode: StorageMode,
        is_dev: bool,
    ) -> Result<Self> {
        // Initialize the TCP stack.
        let tcp = Tcp::new(Config::new(node_ip, max_peers));

        // Prepare the collection of the initial peers.
        let mut initial_peers = HashMap::new();

        // Load entries from the peer cache (if present).
        let cached_peers = Self::load_cached_peers(&storage_mode, PEER_CACHE_FILENAME)?;
        for addr in cached_peers {
            initial_peers.insert(addr, Peer::new_candidate(addr, false));
        }

        // Add the trusted peers to the list of the initial peers; this may promote
        // some of the cached peers to trusted ones.
        initial_peers.extend(trusted_peers.iter().copied().map(|addr| (addr, Peer::new_candidate(addr, true))));

        // Initialize the router.
        Ok(Self(Arc::new(InnerRouter {
            tcp,
            node_type,
            account,
            ledger,
            cache: Default::default(),
            resolver: Default::default(),
            peer_pool: RwLock::new(initial_peers),
            handles: Default::default(),
            rotate_external_peers,
            allow_external_peers,
            storage_mode,
            is_dev,
        })))
    }
}

impl<N: Network> Router<N> {
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

    /// Returns a reference to the cache.
    pub fn cache(&self) -> &Cache<N> {
        &self.cache
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
    pub fn resolve_to_listener(&self, connected_addr: SocketAddr) -> Option<SocketAddr> {
        self.resolver.read().get_listener(connected_addr)
    }

    /// Returns the list of metrics for the connected peers.
    pub fn connected_metrics(&self) -> Vec<(SocketAddr, NodeType)> {
        self.get_connected_peers().iter().map(|peer| (peer.listener_addr, peer.node_type)).collect()
    }

    #[cfg(feature = "metrics")]
    pub fn update_metrics(&self) {
        metrics::gauge(metrics::router::CONNECTED, self.number_of_connected_peers() as f64);
        metrics::gauge(metrics::router::CANDIDATE, self.number_of_candidate_peers() as f64);
    }

    pub fn update_last_seen_for_connected_peer(&self, peer_ip: SocketAddr) {
        if let Some(peer) = self.peer_pool.write().get_mut(&peer_ip) {
            peer.update_last_seen();
        }
    }

    /// Spawns a task with the given future; it should only be used for long-running tasks.
    pub fn spawn<T: Future<Output = ()> + Send + 'static>(&self, future: T) {
        self.handles.lock().push(tokio::spawn(future));
    }

    /// Shuts down the router.
    pub async fn shut_down(&self) {
        info!("Shutting down the router...");
        // Save the best peers for future use.
        if let Err(e) = self.save_best_peers(&self.storage_mode, PEER_CACHE_FILENAME, Some(MAX_PEERS_TO_SEND)) {
            warn!("Failed to persist best peers to disk: {e}");
        }
        // Abort the tasks.
        self.handles.lock().iter().for_each(|handle| handle.abort());
        // Close the listener.
        self.tcp.shut_down().await;
    }
}

#[async_trait]
impl<N: Network> CommunicationService for Router<N> {
    /// The message type.
    type Message = Message<N>;

    /// Prepares a block request to be sent.
    fn prepare_block_request(start_height: u32, end_height: u32) -> Self::Message {
        debug_assert!(start_height < end_height, "Invalid block request format");
        Message::BlockRequest(BlockRequest { start_height, end_height })
    }

    /// Sends the given message to specified peer.
    ///
    /// This function returns as soon as the message is queued to be sent,
    /// without waiting for the actual delivery; instead, the caller is provided with a [`oneshot::Receiver`]
    /// which can be used to determine when and whether the message has been delivered.
    async fn send(
        &self,
        peer_ip: SocketAddr,
        message: Self::Message,
    ) -> Option<tokio::sync::oneshot::Receiver<io::Result<()>>> {
        self.send(peer_ip, message)
    }
}

/// Returns the list of bootstrap peers.
#[allow(clippy::if_same_then_else)]
pub fn bootstrap_peers<N: Network>(is_dev: bool) -> Vec<SocketAddr> {
    if cfg!(feature = "test") || is_dev {
        // Development testing contains optional bootstrap peers loaded from the environment.
        match std::env::var("TEST_BOOTSTRAP_PEERS") {
            Ok(peers) => peers.split(',').map(|peer| SocketAddr::from_str(peer).unwrap()).collect(),
            Err(err) => {
                warn!("Failed to load bootstrap peers from environment: {err}");
                vec![]
            }
        }
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
