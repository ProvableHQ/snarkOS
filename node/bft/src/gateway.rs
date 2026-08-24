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

#[cfg(feature = "telemetry")]
use crate::helpers::{Telemetry, TelemetryWorker};
use crate::{
    CONTEXT,
    MAX_BATCH_DELAY,
    MAX_FETCH_TIMEOUT,
    MEMORY_POOL_PORT,
    Worker,
    events::{DisconnectReason, EventCodec, PrimaryPing},
    helpers::{Cache, PrimarySender, Storage, SyncSender, WorkerSender, assign_to_worker},
    spawn_blocking,
};
use smol_str::SmolStr;
use snarkos_account::Account;
use snarkos_node_bft_events::{
    BlockRequest,
    BlockResponse,
    CertificateRequest,
    CertificateResponse,
    ChallengeRequest,
    ChallengeResponse,
    DataBlocks,
    Event,
    EventTrait,
    HANDSHAKE_DOMAIN,
    HandshakeHint,
    InitiatorInfo,
    PeerInfo,
    ResponderProof,
    TransmissionRequest,
    TransmissionResponse,
    ValidatorsRequest,
    ValidatorsResponse,
};
use snarkos_node_bft_ledger_service::LedgerService;
use snarkos_node_network::{
    ConnectionMode,
    NodeType,
    Peer,
    PeerPoolHandling,
    Resolver,
    bootstrap_peers,
    get_repo_commit_hash,
    harden_socket,
    log_repo_sha_comparison,
    noise::{
        HandshakeProtocol,
        NoiseSession,
        PendingSession,
        Role,
        binding_message,
        detect_handshake_protocol,
        prepare_framed,
        write_noise_magic,
    },
    shorten_snarkos_sha,
};
use snarkos_node_sync::{MAX_BLOCKS_BEHIND, communication_service::CommunicationService};
use snarkos_node_tcp::{
    Config,
    ConnectError,
    Connection,
    ConnectionSide,
    P2P,
    Tcp,
    connections::DisconnectOrigin,
    protocols::{Disconnect, Handshake, OnConnect, Reading, Writing},
};
use snarkos_utilities::NodeDataDir;
use snarkvm::{
    console::prelude::*,
    ledger::{
        committee::Committee,
        narwhal::{BatchHeader, Data},
    },
    prelude::{Address, Field, Signature},
    utilities::flatten_error,
};

use colored::Colorize;
use futures::{SinkExt, future::join_all};
use indexmap::IndexMap;
#[cfg(feature = "locktick")]
use locktick::parking_lot::{Mutex, RwLock};
#[cfg(not(feature = "locktick"))]
use parking_lot::{Mutex, RwLock};
use rand::seq::{IteratorRandom, SliceRandom};
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4},
    sync::Arc,
    time::Duration,
};
use tokio::{
    net::TcpStream,
    sync::{OnceCell, oneshot},
    task::{self, JoinHandle},
};
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

/// The maximum interval of events to cache.
const CACHE_EVENTS_INTERVAL: i64 = (MAX_BATCH_DELAY.as_secs()) as i64; // seconds
/// The maximum interval of requests to cache.
const CACHE_REQUESTS_INTERVAL: i64 = (MAX_BATCH_DELAY.as_secs()) as i64; // seconds

/// The number of consensus rounds of traffic that a per-connection message queue must absorb.
///
/// A message that has not made it onto (or off of) the wire within `MAX_FETCH_TIMEOUT` is already
/// useless to the peer: a requester has given up on it by then (see
/// `Worker::send_transmission_request`), and any consensus event it carried is stale. Sizing the
/// per-connection queues to this window therefore bounds them to the traffic that can still be
/// delivered in time, rather than to the entire garbage-collection window (`MAX_GC_ROUNDS`, which
/// is ~100 rounds, i.e. several minutes of traffic).
const QUEUE_WINDOW_ROUNDS: usize = (MAX_FETCH_TIMEOUT.as_millis() / MAX_BATCH_DELAY.as_millis()) as usize;

/// Computes the depth of the per-connection inbound and outbound message queues.
///
/// These queues are transient send/receive buffers, not backlogs: they only need to hold the
/// traffic a peer can legitimately exchange with us over `QUEUE_WINDOW_ROUNDS` (see above).
/// Per round, the worst case is every certificate in the round plus every transmission each of
/// those certificates contains — that is, a peer that is missing an entire round and fetches all
/// of it from us. The leading factor of 2 is headroom for the remaining, far smaller, event
/// traffic (batch proposals and signatures, primary and worker pings, block and validator
/// requests) and for requests that straddle a round boundary.
///
/// Note that each slot can hold a full `Transmission`, so this value is a direct multiplier on the
/// heap a single peer can pin. It must stay small enough that `depth * max transmission size` is
/// survivable on a commodity validator.
fn per_connection_queue_depth<N: Network>() -> usize {
    2 * QUEUE_WINDOW_ROUNDS
        * N::LATEST_MAX_CERTIFICATES() as usize
        * (BatchHeader::<N>::MAX_TRANSMISSIONS_PER_BATCH + 1)
}

/// The maximum number of connection attempts in an interval.
#[cfg(not(test))]
const MAX_CONNECTION_ATTEMPTS: usize = 10;

/// The maximum number of validators to send in a validators response event.
pub const MAX_VALIDATORS_TO_SEND: usize = 200;

/// The minimum permitted interval between connection attempts for an IP; anything shorter is considered malicious.
#[cfg(not(test))]
const CONNECTION_ATTEMPTS_SINCE_SECS: i64 = 10;

/// The amount of time an IP address is prohibited from connecting.
const IP_BAN_TIME_IN_SECS: u64 = 300;

/// The consensus version at which this node starts *initiating* Noise handshakes, if one is
/// scheduled.
///
/// Only the initiator's choice is gated: a responder accepts either protocol as soon as this code
/// ships, which is what allows validators to be upgraded one at a time. `None` means no switchover
/// has been scheduled yet - except in development, where the Noise path is always taken so that
/// devnets exercise it, and in tests, which pin the choice explicitly.
///
/// Setting this is not the end of the migration, only the middle of it; see
/// [`LEGACY_HANDSHAKE_EXPIRY`].
const NOISE_HANDSHAKE_ACTIVATION: Option<ConsensusVersion> = Some(ConsensusVersion::V20);

/// The consensus version at which this node stops *accepting* the legacy handshake, if one is
/// scheduled.
///
/// This is the step that actually collects what the conversion is for. For as long as the responder
/// still accepts the legacy handshake, two things remain reachable through it: the relay that the
/// handshake binding exists to prevent, and the legacy handshake codec's 1 MiB frame limit - sixteen
/// times what a Noise message may be, on a buffer an unauthenticated peer gets to size. Preferring
/// the new path does not close either; refusing the old one does.
///
/// It must trail [`NOISE_HANDSHAKE_ACTIVATION`] by enough for every peer to have switched, as a
/// validator that has not yet reached the activation still dials with the legacy handshake and would
/// be shut out. Note also that the same relay is reachable through the router's handshake, which
/// signs a byte-identical message with the same account key, so the gateway cannot be the last part
/// of this to be converted.
const LEGACY_HANDSHAKE_EXPIRY: Option<ConsensusVersion> = Some(ConsensusVersion::V21);

/// Part of the Gateway API that deals with networking.
/// This is a separate trait to allow for easier testing/mocking.
#[async_trait]
pub trait Transport<N: Network>: Send + Sync {
    async fn send(&self, peer_ip: SocketAddr, event: Event<N>) -> Option<oneshot::Receiver<io::Result<()>>>;
    fn broadcast(&self, event: Event<N>);
}

/// The gateway maintains connections to other validators.
/// For connections with clients and provers, the Router logic is used.
#[derive(Clone)]
pub struct Gateway<N: Network>(Arc<InnerGateway<N>>);

impl<N: Network> Deref for Gateway<N> {
    type Target = Arc<InnerGateway<N>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct InnerGateway<N: Network> {
    /// The account of the node.
    account: Account<N>,
    /// The storage.
    storage: Storage<N>,
    /// The ledger service.
    ledger: Arc<dyn LedgerService<N>>,
    /// The TCP stack.
    tcp: Tcp,
    /// The cache.
    cache: Cache<N>,
    /// The resolver.
    resolver: RwLock<Resolver<N>>,
    /// The collection of both candidate and connected peers.
    peer_pool: RwLock<HashMap<SocketAddr, Peer<N>>>,
    /// The handle to the validator telemetry tracker.
    #[cfg(feature = "telemetry")]
    validator_telemetry: Telemetry<N>,
    /// The telemetry worker, taken and spawned by `run`.
    ///
    /// This is a one-shot handoff from `new` to `run`, not a hot-path lock: the telemetry
    /// state itself lives in the worker task, and is never shared.
    #[cfg(feature = "telemetry")]
    telemetry_worker: Mutex<Option<TelemetryWorker<N>>>,
    /// The primary sender.
    primary_sender: OnceCell<PrimarySender<N>>,
    /// The worker senders.
    worker_senders: OnceCell<IndexMap<u8, WorkerSender<N>>>,
    /// The sync sender.
    sync_sender: OnceCell<SyncSender<N>>,
    /// The spawned handles.
    handles: Mutex<Vec<JoinHandle<()>>>,
    /// The storage mode.
    node_data_dir: NodeDataDir,
    /// If the flag is set, the node will only connect to trusted peers.
    trusted_peers_only: bool,
    /// The development mode.
    dev: Option<u16>,
    /// Pins which handshake this node offers when it dials, bypassing
    /// [`NOISE_HANDSHAKE_ACTIVATION`].
    ///
    /// Tests default to the Noise handshake, since that is the one under test, and set this to
    /// `false` to cover the other half of the transition: a converted node has to keep talking to
    /// unconverted ones, which means the legacy path must stay exercised for as long as it exists.
    #[cfg(any(test, feature = "test"))]
    initiates_noise_handshake: std::sync::atomic::AtomicBool,
}

impl<N: Network> PeerPoolHandling<N> for Gateway<N> {
    const MAXIMUM_POOL_SIZE: usize = 200;
    const OWNER: &str = CONTEXT;
    const PEER_SLASHING_COUNT: usize = 20;

    fn peer_pool(&self) -> &RwLock<HashMap<SocketAddr, Peer<N>>> {
        &self.peer_pool
    }

    fn resolver(&self) -> &RwLock<Resolver<N>> {
        &self.resolver
    }

    fn is_dev(&self) -> bool {
        self.dev.is_some()
    }

    fn trusted_peers_only(&self) -> bool {
        self.trusted_peers_only
    }

    fn node_type(&self) -> NodeType {
        NodeType::Validator
    }
}

impl<N: Network> Gateway<N> {
    /// Initializes a new gateway.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        account: Account<N>,
        storage: Storage<N>,
        ledger: Arc<dyn LedgerService<N>>,
        ip: Option<SocketAddr>,
        trusted_validators: &[SocketAddr],
        trusted_peers_only: bool,
        node_data_dir: NodeDataDir,
        dev: Option<u16>,
    ) -> Result<Self> {
        // Initialize the gateway IP.
        let ip = match (ip, dev) {
            (None, Some(dev)) => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, MEMORY_POOL_PORT + dev)),
            (None, None) => SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, MEMORY_POOL_PORT)),
            (Some(ip), _) => ip,
        };
        // Initialize the TCP stack.
        //
        // The 10x multiplier allows for more TCP connections than the maximum
        // committee size to prevent "connection refused" errors when two nodes
        // simultaneous attempt to connect to each other. Note, that later,
        // during handshake, the Gateway applies its own limit to the number of
        // active connections and removes duplicates.
        let tcp = Tcp::new(Config::new(ip, Committee::<N>::max_committee_size() * 10));

        // Prepare the collection of the initial peers.
        let mut initial_peers = HashMap::new();

        // Load entries from the validator cache (if present and if we are not in trusted peers only mode).
        if !trusted_peers_only {
            let cached_peers = Self::load_cached_peers(&node_data_dir.gateway_peer_cache_path())?;
            for addr in cached_peers {
                initial_peers.insert(addr, Peer::new_candidate(addr, false));
            }
        }

        // Add the trusted peers to the list of the initial peers; this may promote
        // some of the cached validators to trusted ones.
        initial_peers.extend(trusted_validators.iter().copied().map(|addr| (addr, Peer::new_candidate(addr, true))));

        // Initialize the validator telemetry. The worker is spawned later, by `run`.
        #[cfg(feature = "telemetry")]
        let (validator_telemetry, telemetry_worker) = Telemetry::new();

        // Return the gateway.
        Ok(Self(Arc::new(InnerGateway {
            account,
            storage,
            ledger,
            tcp,
            cache: Default::default(),
            resolver: Default::default(),
            peer_pool: RwLock::new(initial_peers),
            #[cfg(feature = "telemetry")]
            validator_telemetry,
            #[cfg(feature = "telemetry")]
            telemetry_worker: Mutex::new(Some(telemetry_worker)),
            primary_sender: Default::default(),
            worker_senders: Default::default(),
            sync_sender: Default::default(),
            handles: Default::default(),
            node_data_dir,
            trusted_peers_only,
            dev,
            // See the field's documentation for why the tests start out on the Noise handshake.
            #[cfg(any(test, feature = "test"))]
            initiates_noise_handshake: std::sync::atomic::AtomicBool::new(true),
        })))
    }

    /// Run the gateway.
    pub async fn run(
        &self,
        primary_sender: PrimarySender<N>,
        worker_senders: IndexMap<u8, WorkerSender<N>>,
        sync_sender: Option<SyncSender<N>>,
    ) {
        debug!("Starting the gateway for the memory pool...");

        // Set the primary sender.
        self.primary_sender.set(primary_sender).expect("Primary sender already set in gateway");

        // Set the worker senders.
        self.worker_senders.set(worker_senders).expect("The worker senders are already set");

        // If the sync sender was provided, set the sync sender.
        if let Some(sync_sender) = sync_sender {
            self.sync_sender.set(sync_sender).expect("Sync sender already set in gateway");
        }

        // Spawn the validator telemetry worker, which owns all telemetry state.
        // It is registered in `handles`, so `shut_down` aborts it along with everything else.
        #[cfg(feature = "telemetry")]
        if let Some(telemetry_worker) = self.telemetry_worker.lock().take() {
            self.spawn(telemetry_worker.run());
        }

        // Enable the TCP protocols.
        self.enable_handshake().await;
        self.enable_reading().await;
        self.enable_writing().await;
        self.enable_disconnect().await;
        self.enable_on_connect().await;

        // Spawn a loop for periodic metrics.
        #[cfg(feature = "metrics")]
        {
            let gateway = self.clone();
            self.spawn(async move {
                loop {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    gateway.update_metrics();
                }
            });
        }

        // Enable the TCP listener. Note: This must be called after the above protocols.
        let listen_addr = self.tcp.enable_listener().await.expect("Failed to enable the TCP listener");
        debug!("Listening for validator connections at address {listen_addr:?}");

        // Initialize the heartbeat.
        self.initialize_heartbeat();

        info!("Started the gateway for the memory pool at '{}'", self.local_ip());
    }
}

// Dynamic rate limiting.
impl<N: Network> Gateway<N> {
    /// The current maximum committee size.
    fn max_committee_size(&self) -> usize {
        self.ledger
            .current_committee()
            .map_or_else(|_e| Committee::<N>::max_committee_size() as usize, |committee| committee.num_members())
    }

    /// The maximum number of events to cache.
    fn max_cache_events(&self) -> usize {
        self.max_cache_transmissions()
    }

    /// The maximum number of certificate requests to cache.
    fn max_cache_certificates(&self) -> usize {
        2 * BatchHeader::<N>::MAX_GC_ROUNDS * self.max_committee_size()
    }

    /// The maximum number of transmission requests to cache.
    fn max_cache_transmissions(&self) -> usize {
        self.max_cache_certificates() * BatchHeader::<N>::MAX_TRANSMISSIONS_PER_BATCH
    }

    /// The maximum number of duplicates for any particular request.
    fn max_cache_duplicates(&self) -> usize {
        self.max_committee_size().pow(2)
    }
}

#[async_trait]
impl<N: Network> CommunicationService for Gateway<N> {
    /// The message type.
    type Message = Event<N>;

    /// Prepares a block request to be sent.
    fn prepare_block_request(start_height: u32, end_height: u32) -> Self::Message {
        debug_assert!(start_height < end_height, "Invalid block request format");
        Event::BlockRequest(BlockRequest { start_height, end_height })
    }

    /// Sends the given message to specified peer.
    ///
    /// This function returns as soon as the message is queued to be sent,
    /// without waiting for the actual delivery; instead, the caller is provided with a [`oneshot::Receiver`]
    /// which can be used to determine when and whether the message has been delivered.
    async fn send(&self, peer_ip: SocketAddr, message: Self::Message) -> Option<oneshot::Receiver<io::Result<()>>> {
        Transport::send(self, peer_ip, message).await
    }
}

impl<N: Network> Gateway<N> {
    /// Returns the account of the node.
    pub fn account(&self) -> &Account<N> {
        &self.account
    }

    /// Returns the dev identifier of the node.
    pub fn dev(&self) -> Option<u16> {
        self.dev
    }

    /// Returns a reference to the ledger.
    pub fn ledger(&self) -> &Arc<dyn LedgerService<N>> {
        &self.ledger
    }

    /// Returns the resolver.
    pub fn resolver(&self) -> &RwLock<Resolver<N>> {
        &self.resolver
    }

    /// Returns the listener IP address from the (ambiguous) peer address.
    pub fn resolve_to_listener(&self, connected_addr: &SocketAddr) -> Option<SocketAddr> {
        self.resolver.read().get_listener(*connected_addr)
    }

    /// Returns the validator telemetry.
    #[cfg(feature = "telemetry")]
    pub fn validator_telemetry(&self) -> &Telemetry<N> {
        &self.validator_telemetry
    }

    /// Returns the primary sender.
    pub fn primary_sender(&self) -> &PrimarySender<N> {
        self.primary_sender.get().expect("Primary sender not set in gateway")
    }

    /// Returns the number of workers.
    pub fn num_workers(&self) -> u8 {
        u8::try_from(self.worker_senders.get().expect("Missing worker senders in gateway").len())
            .expect("Too many workers")
    }

    /// Returns the worker sender for the given worker ID.
    pub fn get_worker_sender(&self, worker_id: u8) -> Option<&WorkerSender<N>> {
        self.worker_senders.get().and_then(|senders| senders.get(&worker_id))
    }

    /// Returns `true` if the given peer IP is an authorized validator.
    pub fn is_authorized_validator_ip(&self, ip: SocketAddr) -> bool {
        // If the peer IP is in the trusted validators, return early.
        if self.trusted_peers().contains(&ip) {
            return true;
        }
        // Retrieve the Aleo address of the peer IP.
        match self.resolve_to_aleo_addr(ip) {
            // Determine if the peer IP is an authorized validator.
            Some(address) => self.is_authorized_validator_address(address),
            None => {
                warn!("{CONTEXT} Could not resolve the Aleo address for '{ip}'");
                false
            }
        }
    }

    /// Returns `true` if the given address is an authorized validator.
    pub fn is_authorized_validator_address(&self, validator_address: Address<N>) -> bool {
        // Determine if the validator address is a member of the committee lookback,
        // the current committee, or the previous committee lookbacks.
        // We allow leniency in this validation check in order to accommodate these two scenarios:
        //  1. New validators should be able to connect immediately once bonded as a committee member.
        //  2. Existing validators must remain connected until they are no longer bonded as a committee member.
        //     (i.e. meaning they must stay online until the next block has been produced)

        // Determine if the validator is in the current committee with lookback.
        if self
            .ledger
            .get_committee_lookback_for_round(self.storage.current_round())
            .is_ok_and(|committee| committee.is_committee_member(validator_address))
        {
            return true;
        }

        // Determine if the validator is in the latest committee on the ledger.
        if self.ledger.current_committee().is_ok_and(|committee| committee.is_committee_member(validator_address)) {
            return true;
        }

        // Retrieve the previous block height to consider from the sync tolerance.
        let previous_block_height = self.ledger.latest_block_height().saturating_sub(MAX_BLOCKS_BEHIND);
        // Determine if the validator is in any of the previous committee lookbacks.
        match self.ledger.get_block_round(previous_block_height) {
            Ok(block_round) => (block_round..self.storage.current_round()).step_by(2).any(|round| {
                self.ledger
                    .get_committee_lookback_for_round(round)
                    .is_ok_and(|committee| committee.is_committee_member(validator_address))
            }),
            Err(_) => false,
        }
    }

    /// Returns the list of connected addresses.
    pub fn connected_addresses(&self) -> HashSet<Address<N>> {
        self.get_connected_peers().into_iter().map(|peer| peer.aleo_addr).collect()
    }

    /// Ensure the peer is allowed to connect.
    fn ensure_peer_is_allowed(&self, listener_addr: SocketAddr) -> Result<(), DisconnectReason> {
        // Ensure the peer IP is not this node.
        if self.is_local_ip(listener_addr) {
            return Err(DisconnectReason::SelfConnect);
        }

        Ok(())
    }

    /// Updates the connection metrics for the gateway. Ignores the bootstrap clients.
    #[cfg(feature = "metrics")]
    fn update_metrics(&self) {
        if let Some(count) = self.number_of_connected_validators() {
            metrics::gauge(metrics::bft::CONNECTED, count as f64);
        }
        if let Some(count) = self.number_of_connecting_peers() {
            metrics::gauge(metrics::bft::CONNECTING, count as f64);
        }
    }

    /// Inserts the given peer into the connected peers. This is only used in testing.
    #[cfg(test)]
    pub fn insert_connected_peer(&self, peer_ip: SocketAddr, peer_addr: SocketAddr, address: Address<N>) {
        // Adds a bidirectional map between the listener address and (ambiguous) peer address.
        self.resolver.write().insert_peer(peer_ip, peer_addr, Some(address));
        // Add a transmission for this peer in the connected peers.
        self.peer_pool.write().insert(peer_ip, Peer::new_connecting(peer_ip, false));
        if let Some(peer) = self.peer_pool.write().get_mut(&peer_ip) {
            peer.upgrade_to_connected(
                peer_addr,
                peer_ip.port(),
                address,
                NodeType::Validator,
                0,
                get_repo_commit_hash(),
                ConnectionMode::Gateway,
            );
        }
    }

    /// Sends the given event to specified peer.
    ///
    /// This function returns as soon as the event is queued to be sent,
    /// without waiting for the actual delivery; instead, the caller is provided with a [`oneshot::Receiver`]
    /// which can be used to determine when and whether the event has been delivered.
    fn send_inner(&self, peer_ip: SocketAddr, event: Event<N>) -> Option<oneshot::Receiver<io::Result<()>>> {
        // Resolve the listener IP to the (ambiguous) peer address.
        let Some(peer_addr) = self.resolve_to_ambiguous(peer_ip) else {
            warn!("Unable to resolve the listener IP address '{peer_ip}'");
            return None;
        };
        // Retrieve the event name.
        let name = event.name();
        // Send the event to the peer.
        trace!("{CONTEXT} Sending '{name}' to '{peer_ip}'");
        let result = self.unicast(peer_addr, event);
        // If the event was unable to be sent, disconnect.
        if let Err(err) = &result {
            warn!("{CONTEXT} Failed to send '{name}' to '{peer_ip}': {err:?}");
            debug!("{CONTEXT} Disconnecting from '{peer_ip}' (unable to send)");
            self.disconnect(peer_ip);
        }
        result.ok()
    }

    /// Handles the inbound event from the peer. The returned value indicates whether
    /// the connection is still active, and errors cause a disconnect once they are
    /// propagated to the caller.
    async fn inbound(&self, peer_addr: SocketAddr, event: Event<N>) -> Result<bool> {
        // Retrieve the listener IP for the peer.
        let Some(peer_ip) = self.resolver.read().get_listener(peer_addr) else {
            // No longer connected to the peer.
            trace!("Dropping a {} from {peer_addr} - no longer connected.", event.name());
            return Ok(false);
        };
        // Ensure that the peer is an authorized committee member or a bootstrapper.
        if !(self.is_authorized_validator_ip(peer_ip)
            || self
                .get_connected_peer(peer_ip)
                .map(|peer| peer.node_type == NodeType::BootstrapClient)
                .unwrap_or(false))
        {
            bail!("{CONTEXT} Dropping '{}' from '{peer_ip}' (not authorized)", event.name())
        }
        // Drop the peer, if they have exceeded the rate limit (i.e. they are requesting too much from us).
        let num_events = self.cache.insert_inbound_event(peer_ip, CACHE_EVENTS_INTERVAL);
        if num_events >= self.max_cache_events() {
            bail!("Dropping '{peer_ip}' for spamming events (num_events = {num_events})")
        }
        // Rate limit for duplicate requests.
        match event {
            Event::CertificateRequest(_) | Event::CertificateResponse(_) => {
                // Retrieve the certificate ID.
                let certificate_id = match &event {
                    Event::CertificateRequest(CertificateRequest { certificate_id }) => *certificate_id,
                    Event::CertificateResponse(CertificateResponse { certificate }) => certificate.id(),
                    _ => unreachable!(),
                };
                // Skip processing this certificate if the rate limit was exceed (i.e. someone is spamming a specific certificate).
                let num_events = self.cache.insert_inbound_certificate(certificate_id, CACHE_REQUESTS_INTERVAL);
                if num_events >= self.max_cache_duplicates() {
                    return Ok(true);
                }
            }
            Event::TransmissionRequest(TransmissionRequest { transmission_id })
            | Event::TransmissionResponse(TransmissionResponse { transmission_id, .. }) => {
                // Skip processing this certificate if the rate limit was exceeded (i.e. someone is spamming a specific certificate).
                let num_events = self.cache.insert_inbound_transmission(transmission_id, CACHE_REQUESTS_INTERVAL);
                if num_events >= self.max_cache_duplicates() {
                    return Ok(true);
                }
            }
            Event::BlockRequest(_) => {
                let num_events = self.cache.insert_inbound_block_request(peer_ip, CACHE_REQUESTS_INTERVAL);
                if num_events >= self.max_cache_duplicates() {
                    return Ok(true);
                }
            }
            _ => {}
        }
        trace!("{CONTEXT} Received '{}' from '{peer_ip}'", event.name());

        // This match statement handles the inbound event by deserializing the event,
        // checking the event is valid, and then calling the appropriate (trait) handler.
        match event {
            Event::BatchPropose(batch_propose) => {
                // Send the batch propose to the primary.
                let _ = self.primary_sender().tx_batch_propose.send((peer_ip, batch_propose)).await;
                Ok(true)
            }
            Event::BatchSignature(batch_signature) => {
                // Send the batch signature to the primary.
                let _ = self.primary_sender().tx_batch_signature.send((peer_ip, batch_signature)).await;
                Ok(true)
            }
            Event::BatchCertified(batch_certified) => {
                // Send the batch certificate to the primary.
                let _ = self.primary_sender().tx_batch_certified.send((peer_ip, batch_certified.certificate)).await;
                Ok(true)
            }
            Event::BlockRequest(block_request) => {
                let BlockRequest { start_height, end_height } = block_request;

                // Ensure the block request is well-formed.
                if start_height >= end_height {
                    bail!("Block request from '{peer_ip}' has an invalid range ({start_height}..{end_height})")
                }
                // Ensure that the block request is within the allowed bounds.
                if end_height - start_height > DataBlocks::<N>::MAXIMUM_NUMBER_OF_BLOCKS as u32 {
                    bail!("Block request from '{peer_ip}' has an excessive range ({start_height}..{end_height})")
                }

                // End height is exclusive.
                let latest_consensus_version = N::CONSENSUS_VERSION(end_height - 1)?;

                let self_ = self.clone();
                let blocks = match task::spawn_blocking(move || {
                    // Retrieve the blocks within the requested range.
                    match self_.ledger.get_blocks(start_height..end_height) {
                        Ok(blocks) => Ok(DataBlocks(blocks)),
                        Err(error) => bail!("Missing blocks {start_height} to {end_height} from ledger - {error}"),
                    }
                })
                .await
                {
                    Ok(Ok(blocks)) => blocks,
                    Ok(Err(error)) => return Err(error),
                    Err(error) => return Err(anyhow!("[BlockRequest] {error}")),
                };

                let self_ = self.clone();
                tokio::spawn(async move {
                    // Send the `BlockResponse` message to the peer.
                    let event =
                        Event::BlockResponse(BlockResponse::new(block_request, blocks, latest_consensus_version));
                    Transport::send(&self_, peer_ip, event).await;
                });
                Ok(true)
            }
            Event::BlockResponse(BlockResponse { request, latest_consensus_version, blocks, .. }) => {
                // Process the block response. Except for some tests, there is always a sync sender.
                if let Some(sync_sender) = self.sync_sender.get() {
                    // Check the response corresponds to a request.
                    if !self.cache.remove_outbound_block_request(peer_ip, &request) {
                        bail!("Unsolicited block response from '{peer_ip}'")
                    }

                    // Perform the deferred non-blocking deserialization of the blocks.
                    // The deserialization can take a long time (minutes). We should not be running
                    // this on a blocking task, but on a rayon thread pool.
                    let (send, recv) = tokio::sync::oneshot::channel();
                    rayon::spawn_fifo(move || {
                        let blocks = blocks.deserialize_blocking().map_err(|error| anyhow!("[BlockResponse] {error}"));
                        let _ = send.send(blocks);
                    });
                    let blocks = match recv.await {
                        Ok(Ok(blocks)) => blocks,
                        Ok(Err(error)) => bail!("Peer '{peer_ip}' sent an invalid block response - {error}"),
                        Err(error) => bail!("Peer '{peer_ip}' sent an invalid block response - {error}"),
                    };

                    // Ensure the block response is well-formed.
                    blocks.ensure_response_is_well_formed(peer_ip, request.start_height, request.end_height)?;
                    // Send the blocks to the sync module.
                    match sync_sender.insert_block_response(peer_ip, blocks.0, latest_consensus_version).await {
                        Ok(_) => Ok(true),
                        Err(err) if err.is_benign() => {
                            let err: anyhow::Error = err.into();
                            let err = err.context(format!("Ignoring block response from peer '{peer_ip}'"));
                            debug!("{}", flatten_error(err));
                            Ok(true)
                        }
                        Err(err) if err.is_consensus_version_ahead() => {
                            let err: anyhow::Error = err.into();
                            let err = err.context(format!(
                                "Peer sent a block response with a newer consensus version '{peer_ip}'"
                            ));
                            warn!("{}", flatten_error(&err));
                            Ok(true)
                        }
                        Err(err) if err.is_consensus_version_behind() => {
                            let err: anyhow::Error = err.into();
                            let err = err.context(format!("Peer sent an invalid block response '{peer_ip}'"));

                            let msg = flatten_error(&err);
                            error!("{msg}");
                            self.ip_ban_peer(peer_ip, Some(&msg));
                            Err(err)
                        }
                        Err(err) => {
                            let err: anyhow::Error = err.into();
                            let err = err.context(format!("Peer '{peer_ip}' sent an invalid block response"));
                            warn!("{}", flatten_error(err));

                            // TODO(kaimast): This needs more testing to ensure disconnect is the correct action.
                            Ok(true)
                        }
                    }
                } else {
                    debug!("Ignoring block response from '{peer_ip}' - no sync sender");
                    Ok(true)
                }
            }
            Event::CertificateRequest(certificate_request) => {
                // Send the certificate request to the sync module.
                // Except for some tests, there is always a sync sender.
                if let Some(sync_sender) = self.sync_sender.get() {
                    // Send the certificate request to the sync module.
                    let _ = sync_sender.tx_certificate_request.send((peer_ip, certificate_request)).await;
                }
                Ok(true)
            }
            Event::CertificateResponse(certificate_response) => {
                // Send the certificate response to the sync module.
                // Except for some tests, there is always a sync sender.
                if let Some(sync_sender) = self.sync_sender.get() {
                    // Send the certificate response to the sync module.
                    let _ = sync_sender.tx_certificate_response.send((peer_ip, certificate_response)).await;
                }
                Ok(true)
            }
            Event::ChallengeRequest(..) | Event::ChallengeResponse(..) => {
                // Disconnect as the peer is not following the protocol.
                bail!("{CONTEXT} Peer '{peer_ip}' is not following the protocol")
            }
            Event::Disconnect(message) => {
                // The peer informs us that they had disconnected. Disconnect from them too.
                debug!("Peer '{peer_ip}' decided to disconnect due to '{}'", message.reason);
                self.disconnect(peer_ip);
                Ok(false)
            }
            Event::PrimaryPing(ping) => {
                let PrimaryPing { version, block_locators, primary_certificate } = ping;

                // Ensure the event version is not outdated.
                if version < Event::<N>::VERSION {
                    bail!("Dropping '{peer_ip}' on event version {version} (outdated)");
                }

                // Log the validator's height.
                debug!("Validator '{peer_ip}' is at height {}", block_locators.latest_locator_height());

                // Update the peer locators. Except for some tests, there is always a sync sender.
                if let Some(sync_sender) = self.sync_sender.get() {
                    // Check the block locators are valid, and update the validators in the sync module.
                    if let Err(error) = sync_sender.update_peer_locators(peer_ip, block_locators).await {
                        bail!("Validator '{peer_ip}' sent invalid block locators - {error}");
                    }
                }

                // Send the batch certificates to the primary.
                let _ = self.primary_sender().tx_primary_ping.send((peer_ip, primary_certificate)).await;
                Ok(true)
            }
            Event::TransmissionRequest(request) => {
                // TODO (howardwu): Add rate limiting checks on this event, on a per-peer basis.
                // Determine the worker ID.
                let Ok(worker_id) = assign_to_worker(request.transmission_id, self.num_workers()) else {
                    warn!("{CONTEXT} Unable to assign transmission ID '{}' to a worker", request.transmission_id);
                    return Ok(true);
                };
                // Send the transmission request to the worker.
                if let Some(sender) = self.get_worker_sender(worker_id) {
                    // Send the transmission request to the worker.
                    let _ = sender.tx_transmission_request.send((peer_ip, request)).await;
                }
                Ok(true)
            }
            Event::TransmissionResponse(response) => {
                // Determine the worker ID.
                let Ok(worker_id) = assign_to_worker(response.transmission_id, self.num_workers()) else {
                    warn!("{CONTEXT} Unable to assign transmission ID '{}' to a worker", response.transmission_id);
                    return Ok(true);
                };
                // Send the transmission response to the worker.
                if let Some(sender) = self.get_worker_sender(worker_id) {
                    // Send the transmission response to the worker.
                    let _ = sender.tx_transmission_response.send((peer_ip, response)).await;
                }
                Ok(true)
            }
            Event::ValidatorsRequest(_) => {
                let mut connected_peers = self.get_best_connected_peers(Some(MAX_VALIDATORS_TO_SEND));
                connected_peers.shuffle(&mut rand::rng());

                let self_ = self.clone();
                tokio::spawn(async move {
                    // Initialize the validators.
                    let mut validators = IndexMap::with_capacity(MAX_VALIDATORS_TO_SEND);
                    // Iterate over the validators.
                    for validator in connected_peers.into_iter() {
                        // Add the validator to the list of validators.
                        validators.insert(validator.listener_addr, validator.aleo_addr);
                    }
                    // Send the validators response to the peer.
                    let event = Event::ValidatorsResponse(ValidatorsResponse { validators });
                    Transport::send(&self_, peer_ip, event).await;
                });
                Ok(true)
            }
            Event::ValidatorsResponse(response) => {
                if self.trusted_peers_only {
                    bail!("{CONTEXT} Not accepting validators response from '{peer_ip}' (trusted peers only)");
                }
                let ValidatorsResponse { validators } = response;
                // Ensure the number of validators is not too large.
                ensure!(validators.len() <= MAX_VALIDATORS_TO_SEND, "{CONTEXT} Received too many validators");
                // Ensure the cache contains a validators request for this peer.
                if !self.cache.contains_outbound_validators_request(peer_ip) {
                    bail!("{CONTEXT} Received validators response from '{peer_ip}' without a validators request")
                }
                // Decrement the number of validators requests for this peer.
                self.cache.decrement_outbound_validators_requests(peer_ip);

                // Add valid validators as candidates to the peer pool; only validator-related
                // filters need to be applied, the rest is handled by `PeerPoolHandling`.
                let valid_addrs = validators
                    .into_iter()
                    .filter_map(|(listener_addr, aleo_addr)| {
                        (self.account.address() != aleo_addr
                            && !self.is_connected_address(aleo_addr)
                            && self.is_authorized_validator_address(aleo_addr))
                        .then_some((listener_addr, None))
                    })
                    .collect::<Vec<_>>();
                if !valid_addrs.is_empty() {
                    self.insert_candidate_peers(valid_addrs);
                }

                Ok(true)
            }
            Event::WorkerPing(ping) => {
                // Ensure the number of transmissions is not too large.
                ensure!(
                    ping.transmission_ids.len() <= Worker::<N>::MAX_TRANSMISSIONS_PER_WORKER_PING,
                    "{CONTEXT} Received too many transmissions"
                );
                // Retrieve the number of workers.
                let num_workers = self.num_workers();
                // Iterate over the transmission IDs.
                for transmission_id in ping.transmission_ids.into_iter() {
                    // Determine the worker ID.
                    let Ok(worker_id) = assign_to_worker(transmission_id, num_workers) else {
                        warn!("{CONTEXT} Unable to assign transmission ID '{transmission_id}' to a worker");
                        continue;
                    };
                    // Send the transmission ID to the worker.
                    if let Some(sender) = self.get_worker_sender(worker_id) {
                        // Send the transmission ID to the worker.
                        let _ = sender.tx_worker_ping.send((peer_ip, transmission_id)).await;
                    }
                }
                Ok(true)
            }
        }
    }

    /// Initialize a new instance of the heartbeat.
    fn initialize_heartbeat(&self) {
        let self_clone = self.clone();
        self.spawn(async move {
            // Sleep briefly to ensure the other nodes are ready to connect.
            tokio::time::sleep(Duration::from_millis(1000)).await;
            info!("Starting the heartbeat of the gateway...");
            loop {
                // Process a heartbeat in the gateway.
                self_clone.heartbeat().await;
                // Sleep for the heartbeat interval.
                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }

    /// Spawns a task with the given future; it should only be used for long-running tasks.
    #[allow(dead_code)]
    fn spawn<T: Future<Output = ()> + Send + 'static>(&self, future: T) {
        self.handles.lock().push(tokio::spawn(future));
    }

    /// Shuts down the gateway.
    pub async fn shut_down(&self) {
        info!("Shutting down the gateway...");
        // Save the best peers for future use.
        if let Err(e) = self.save_best_peers(&self.node_data_dir.gateway_peer_cache_path(), None, true) {
            warn!("Failed to persist best validators to disk: {e}");
        }
        // Abort the tasks.
        self.handles.lock().iter().for_each(|handle| handle.abort());
        // Close the listener.
        self.tcp.shut_down().await;
    }
}

impl<N: Network> Gateway<N> {
    /// The minimum time between connection attempts to a peer.
    const MINIMUM_TIME_BETWEEN_CONNECTION_ATTEMPTS: Duration = Duration::from_secs(10);
    /// The uptime after which nodes log a warning about missing validator connections.
    const MISSING_VALIDATOR_CONNECTIONS_GRACE_PERIOD: Duration = Duration::from_secs(60);

    /// Handles the heartbeat request.
    async fn heartbeat(&self) {
        // Log the connected validators.
        self.log_connected_validators();
        // Log the validator participation scores.
        #[cfg(feature = "telemetry")]
        self.log_participation_scores();
        // Keep the trusted validators connected.
        self.handle_trusted_validators();
        // Keep the bootstrap peers within the allowed range.
        self.handle_bootstrap_peers().await;
        // Removes any validators that not in the current committee.
        self.handle_unauthorized_validators();
        // If the number of connected validators is less than the minimum, send a `ValidatorsRequest`.
        self.handle_min_connected_validators().await;
        // Unban any addresses whose ban time has expired.
        self.handle_banned_ips();
    }

    /// Logs the connected validators.
    fn log_connected_validators(&self) {
        // Retrieve the connected validators and current committee.
        // The gatway may also be connected to bootstrap clients, which we should not log as connected validators.
        let connected_validators = self.filter_connected_peers(|peer| peer.node_type == NodeType::Validator);

        let committee = match self.ledger.current_committee() {
            Ok(c) => c,
            Err(err) => {
                error!("Failed to get current committee: {err}");
                return;
            }
        };

        // Resolve the total number of connectable validators.
        let validators_total = committee.num_members().saturating_sub(1);
        // Format the total validators message.
        let total_validators = format!("(of {validators_total} bonded validators)").dimmed();
        // Construct the connections message.
        let connections_msg = match connected_validators.len() {
            0 => "No connected validators".to_string(),
            num_connected => format!("Connected to {num_connected} validators {total_validators}"),
        };
        info!("{connections_msg}");

        // Collect the connected validator addresses and stake.
        let mut connected_validator_addresses = HashSet::with_capacity(connected_validators.len());
        let mut connected_validator_shas: HashMap<SmolStr, u64> = HashMap::with_capacity(connected_validators.len());
        // Insert our sha.
        let our_sha = shorten_snarkos_sha(&get_repo_commit_hash());
        let our_stake = committee.get_stake(self.account.address());
        connected_validator_shas.insert(our_sha.clone(), our_stake);
        // Include our own address.
        connected_validator_addresses.insert(self.account.address());
        // Include and log the connected validators.
        for peer in &connected_validators {
            // Register the Aleo address.
            let address = peer.aleo_addr;
            connected_validator_addresses.insert(address);
            // Register the snarkOS commit SHA and the associated stake.
            let address_stake = committee.get_stake(address);
            let short_peer_sha = shorten_snarkos_sha(&peer.snarkos_sha);
            *connected_validator_shas.entry(short_peer_sha.clone()).or_default() += address_stake;

            debug!(
                "{}",
                format!(
                    "  Connected to: {} - {} (connection age {:?})",
                    peer.listener_addr,
                    peer.aleo_addr,
                    peer.first_seen.elapsed()
                )
                .dimmed()
            );
        }

        // Log how much of the stake uses our git commit hash.
        if let Some(combined_stake) = connected_validator_shas.get(&our_sha) {
            let percentage = *combined_stake as f64 / committee.total_stake() as f64 * 100.0;
            debug!("{}", format!("  Combined stake @ {our_sha}: {percentage:.2}%").dimmed());
            #[cfg(feature = "metrics")]
            metrics::gauge(metrics::bft::CONNECTED_STAKE_WITH_MATCHING_SHA, percentage);
        }

        // Log the validators that are not connected.
        let num_not_connected = validators_total.saturating_sub(connected_validators.len());
        if num_not_connected > 0 && self.tcp().uptime() > Self::MISSING_VALIDATOR_CONNECTIONS_GRACE_PERIOD {
            // Cache the total stake for computing percentages.
            let total_stake = committee.total_stake();
            let total_stake_f64 = total_stake as f64;

            // Collect the committee members.
            let committee_members: HashSet<_> =
                self.ledger.current_committee().map(|c| c.members().keys().copied().collect()).unwrap_or_default();

            let not_connected_stake: u64 = committee_members
                .difference(&connected_validator_addresses)
                .map(|address| {
                    let address_stake = committee.get_stake(*address);
                    let address_stake_as_percentage =
                        if total_stake == 0 { 0.0 } else { address_stake as f64 / total_stake_f64 * 100.0 };
                    debug!(
                        "{}",
                        format!("  Not connected to {address} ({address_stake_as_percentage:.2}% of total stake)")
                            .dimmed()
                    );
                    address_stake
                })
                .sum();

            let not_connected_stake_as_percentage =
                if total_stake == 0 { 0.0 } else { not_connected_stake as f64 / total_stake_f64 * 100.0 };
            warn!(
                "Not connected to {num_not_connected} validators {total_validators} ({not_connected_stake_as_percentage:.2}% of total stake not connected)"
            );
            #[cfg(feature = "metrics")]
            {
                let connected_stake_as_percentage = 100.0 - not_connected_stake_as_percentage;
                metrics::gauge(metrics::bft::CONNECTED_STAKE, connected_stake_as_percentage);
            }
        } else {
            #[cfg(feature = "metrics")]
            metrics::gauge(metrics::bft::CONNECTED_STAKE, 100.0);
        };

        if !committee.is_quorum_threshold_reached(&connected_validator_addresses) {
            // Not being connected to a quorum of validators is begning during startup.
            if self.tcp().uptime() > Self::MISSING_VALIDATOR_CONNECTIONS_GRACE_PERIOD {
                error!("Not connected to a quorum of validators");
            } else {
                debug!("Not connected to a quorum of validators");
            }
        }
    }

    // Logs the validator participation scores.
    #[cfg(feature = "telemetry")]
    fn log_participation_scores(&self) {
        if let Ok(committee_lookback) = self.ledger.get_committee_lookback_for_round(self.storage.current_round()) {
            // Retrieve the participation scores.
            let participation_scores = self.validator_telemetry().get_participation_scores(&committee_lookback);

            // Log the participation scores.
            debug!("Participation Scores (in the last {} rounds):", self.storage.max_gc_rounds());
            for (address, (cert_score, sig_score)) in participation_scores {
                debug!(
                    "{}",
                    format!("  {address} - certificates: {cert_score:.2}%  signatures: {sig_score:.2}%").dimmed()
                );
            }
        }
    }

    /// This function attempts to connect to any disconnected trusted validators.
    fn handle_trusted_validators(&self) {
        let trusted_peers = self.trusted_peers();

        // Attempt to re-establish connections with any trusted peer that is not connected already.
        let handles: Vec<JoinHandle<_>> = trusted_peers
            .iter()
            .filter_map(|validator_ip| {
                // Attempt to connect to the trusted validator.
                match self.connect(*validator_ip) {
                    Ok(hdl) => Some(hdl),
                    Err(ConnectError::SelfConnect { .. })
                    | Err(ConnectError::AlreadyConnected { .. })
                    | Err(ConnectError::AlreadyConnecting { .. }) => None,
                    Err(err) => {
                        warn!("Could not initiate connection to trusted validator at '{validator_ip}' - {err}");
                        None
                    }
                }
            })
            .collect();

        if !handles.is_empty() {
            info!("Reconnecting to {} out of {} trusted validators", handles.len(), trusted_peers.len());
        }
    }

    /// This function keeps the number of bootstrap peers within the allowed range.
    async fn handle_bootstrap_peers(&self) {
        // Return early if we are in trusted peers only mode.
        if self.trusted_peers_only {
            return;
        }
        // Split the bootstrap peers into connected and candidate lists.
        let mut candidate_bootstrap = Vec::new();
        let connected_bootstrap = self.filter_connected_peers(|peer| peer.node_type == NodeType::BootstrapClient);
        for bootstrap_ip in bootstrap_peers::<N>(self.is_dev()) {
            if !connected_bootstrap.iter().any(|peer| peer.listener_addr == bootstrap_ip) {
                candidate_bootstrap.push(bootstrap_ip);
            }
        }
        // If there are not enough connected bootstrap peers, connect to more.
        if connected_bootstrap.is_empty() {
            // Sample a random bootstrap peer to connect to (drop rng before any await).
            let peer_to_connect = candidate_bootstrap.into_iter().choose(&mut rand::rng());
            if let Some(peer_ip) = peer_to_connect {
                match self.connect(peer_ip) {
                    Ok(hdl) => {
                        debug!("{CONTEXT} (Re-)connecting to bootstrap peer at '{peer_ip}'");
                        let result = hdl.await;
                        if let Err(err) = result {
                            warn!("{CONTEXT} Failed to connect to bootstrap peer at '{peer_ip}' - {err}");
                        }
                    }
                    Err(ConnectError::AlreadyConnected { .. }) | Err(ConnectError::AlreadyConnecting { .. }) => {}
                    Err(err) => {
                        warn!("{CONTEXT} Could not initiate connection to bootstrap peer at '{peer_ip}' - {err}")
                    }
                }
            }
        }
        // Determine if the node is connected to more bootstrap peers than allowed.
        let num_surplus = connected_bootstrap.len().saturating_sub(1);
        if num_surplus > 0 {
            // Sample peers to disconnect (drop rng before any await).
            let peers_to_disconnect = connected_bootstrap.into_iter().sample(&mut rand::rng(), num_surplus);
            for peer in peers_to_disconnect {
                info!("{CONTEXT} Disconnecting from '{}' (exceeded maximum bootstrap)", peer.listener_addr);
                <Self as Transport<N>>::send(
                    self,
                    peer.listener_addr,
                    Event::Disconnect(DisconnectReason::NoReasonGiven.into()),
                )
                .await;
                // Disconnect from this peer.
                self.disconnect(peer.listener_addr);
            }
        }
    }

    /// This function attempts to disconnect any validators that are not in the current committee.
    fn handle_unauthorized_validators(&self) {
        let self_ = self.clone();
        tokio::spawn(async move {
            // Retrieve the connected validators.
            let validators = self_.get_connected_peers();
            // Iterate over the validator IPs.
            for peer in validators {
                // Skip bootstrapper peers.
                if peer.node_type == NodeType::BootstrapClient {
                    continue;
                }
                // Disconnect any validator that is not in the current committee.
                if !self_.is_authorized_validator_ip(peer.listener_addr) {
                    warn!(
                        "{CONTEXT} Disconnecting from '{}' - Validator is not in the current committee",
                        peer.listener_addr
                    );
                    Transport::send(&self_, peer.listener_addr, DisconnectReason::ProtocolViolation.into()).await;
                    // Disconnect from this peer.
                    self_.disconnect(peer.listener_addr);
                }
            }
        });
    }

    /// This function sends a `ValidatorsRequest` to a random validator,
    /// if the number of connected validators is less than the minimum.
    /// It also attempts to connect to known unconnected validators.
    async fn handle_min_connected_validators(&self) {
        // Attempt to connect to untrusted validators we're not connected to yet.
        // The trusted ones are already handled by `handle_trusted_validators`.
        let trusted_validators = self.trusted_peers();
        if self.number_of_connected_peers() < N::LATEST_MAX_CERTIFICATES() as usize {
            let (addrs, handles): (Vec<_>, Vec<_>) = self
                .get_candidate_peers()
                .iter()
                .filter_map(|peer| {
                    if trusted_validators.contains(&peer.listener_addr) {
                        return None;
                    }

                    if let Some(previous_attempt) = peer.last_connection_attempt
                        && previous_attempt.elapsed() < Self::MINIMUM_TIME_BETWEEN_CONNECTION_ATTEMPTS
                    {
                        return None;
                    }

                    match self.connect(peer.listener_addr) {
                        Ok(hdl) => Some((peer.listener_addr, hdl)),
                        Err(ConnectError::AlreadyConnected { .. })
                        | Err(ConnectError::AlreadyConnecting { .. })
                        | Err(ConnectError::SelfConnect { .. }) => None,
                        Err(err) => {
                            warn!(
                                "{CONTEXT} Could not initiate connection to validator at '{}' - {err}",
                                peer.listener_addr
                            );
                            None
                        }
                    }
                })
                .unzip();

            for (addr, result) in addrs.into_iter().zip(join_all(handles).await) {
                if let Err(err) = result {
                    warn!("{CONTEXT} Failed to connect to validator at '{addr}' - {err}");
                }
            }

            // Retrieve the connected validators.
            let validators = self.connected_peers();
            // If there are no validator IPs to connect to, return early.
            if validators.is_empty() {
                return;
            }
            // Select a random validator IP.
            if let Some(validator_ip) = validators.into_iter().choose(&mut rand::rng()) {
                let self_ = self.clone();
                tokio::spawn(async move {
                    // Increment the number of outbound validators requests for this validator.
                    self_.cache.increment_outbound_validators_requests(validator_ip);
                    // Send a `ValidatorsRequest` to the validator.
                    let _ = Transport::send(&self_, validator_ip, Event::ValidatorsRequest(ValidatorsRequest)).await;
                });
            }
        }
    }

    /// Processes a message received from the network.
    async fn process_message_inner(&self, peer_addr: SocketAddr, message: Event<N>) {
        // Process the message. Disconnect if the peer violated the protocol.
        if let Err(error) = self.inbound(peer_addr, message).await
            && let Some(peer_ip) = self.resolver.read().get_listener(peer_addr)
        {
            warn!("{CONTEXT} Disconnecting from '{peer_ip}' - {error}");
            let self_ = self.clone();
            tokio::spawn(async move {
                Transport::send(&self_, peer_ip, DisconnectReason::ProtocolViolation.into()).await;
                // Disconnect from this peer.
                self_.disconnect(peer_ip);
            });
        }
    }

    // Remove addresses whose ban time has expired.
    fn handle_banned_ips(&self) {
        self.tcp.banned_peers().remove_old_bans(IP_BAN_TIME_IN_SECS);
    }
}

#[async_trait]
impl<N: Network> Transport<N> for Gateway<N> {
    /// Sends the given event to specified peer.
    ///
    /// This method is rate limited to prevent spamming the peer.
    ///
    /// This function returns as soon as the event is queued to be sent,
    /// without waiting for the actual delivery; instead, the caller is provided with a [`oneshot::Receiver`]
    /// which can be used to determine when and whether the event has been delivered.
    async fn send(&self, peer_ip: SocketAddr, mut event: Event<N>) -> Option<oneshot::Receiver<io::Result<()>>> {
        // Serialize the payload here, rather than leaving it for the connection's writer task.
        //
        // `Data` defers serialization until the event is written to the stream, which puts it on
        // the writer task -- a Tokio worker -- and inside the write timeout. For the large
        // responses (`BlockResponse` in particular, which can carry `DataBlocks::MAXIMUM_NUMBER_
        // OF_BLOCKS`) that is a substantial amount of compute on the reactor.
        //
        // The check keeps the hop off the common path: most events carry no payload, and a
        // broadcast has already serialized its payload before fanning out here, so this is a no-op
        // for every recipient after the first.
        if event.has_unserialized_payload() {
            let name = event.name();
            event = match spawn_blocking!({
                let mut event = event;
                event.serialize_payload()?;
                Ok(event)
            }) {
                Ok(event) => event,
                Err(err) => {
                    error!("{CONTEXT} Unable to serialize '{name}' for '{peer_ip}' - {err}");
                    return None;
                }
            };
        }

        macro_rules! send {
            ($self:ident, $cache_map:ident, $interval:expr, $freq:ident) => {{
                // Rate limit the number of certificate requests sent to the peer.
                while $self.cache.$cache_map(peer_ip, $interval) > $self.$freq() {
                    // Sleep for a short period of time to allow the cache to clear.
                    tokio::time::sleep(Duration::from_millis(10)).await;
                }
                // Send the event to the peer.
                $self.send_inner(peer_ip, event)
            }};
        }

        // Increment the cache for certificate, transmission and block events.
        match event {
            Event::CertificateRequest(_) | Event::CertificateResponse(_) => {
                // Update the outbound event cache. This is necessary to ensure we don't under count the outbound events.
                self.cache.insert_outbound_event(peer_ip, CACHE_EVENTS_INTERVAL);
                // Send the event to the peer.
                send!(self, insert_outbound_certificate, CACHE_REQUESTS_INTERVAL, max_cache_certificates)
            }
            Event::TransmissionRequest(_) | Event::TransmissionResponse(_) => {
                // Update the outbound event cache. This is necessary to ensure we don't under count the outbound events.
                self.cache.insert_outbound_event(peer_ip, CACHE_EVENTS_INTERVAL);
                // Send the event to the peer.
                send!(self, insert_outbound_transmission, CACHE_REQUESTS_INTERVAL, max_cache_transmissions)
            }
            Event::BlockRequest(request) => {
                // Insert the outbound request so we can match it to responses.
                self.cache.insert_outbound_block_request(peer_ip, request);
                // Send the event to the peer and update the outbound event cache, use the general rate limit.
                send!(self, insert_outbound_event, CACHE_EVENTS_INTERVAL, max_cache_events)
            }
            _ => {
                // Send the event to the peer, use the general rate limit.
                send!(self, insert_outbound_event, CACHE_EVENTS_INTERVAL, max_cache_events)
            }
        }
    }

    /// Broadcasts the given event to all connected peers.
    fn broadcast(&self, mut event: Event<N>) {
        // Ensure there are connected peers.
        if self.number_of_connected_peers() > 0 {
            let self_ = self.clone();
            let connected_peers = self.connected_peers();
            tokio::spawn(async move {
                // Serialize the event's payload once, rather than once per recipient; every
                // recipient then shares the resulting buffer. `Transport::send` would otherwise do
                // this separately for each peer below.
                if event.has_unserialized_payload() {
                    let name = event.name();
                    event = match spawn_blocking!({
                        let mut event = event;
                        event.serialize_payload()?;
                        Ok(event)
                    }) {
                        Ok(event) => event,
                        Err(err) => {
                            error!("{CONTEXT} Unable to serialize '{name}' for broadcast - {err}");
                            return;
                        }
                    };
                }
                // Iterate through all connected peers.
                for peer_ip in connected_peers {
                    // Send the event to the peer.
                    let _ = Transport::send(&self_, peer_ip, event.clone()).await;
                }
            });
        }
    }
}

impl<N: Network> P2P for Gateway<N> {
    /// Returns a reference to the TCP instance.
    fn tcp(&self) -> &Tcp {
        &self.tcp
    }
}

#[async_trait]
impl<N: Network> Reading for Gateway<N> {
    type Codec = EventCodec<N>;
    type Message = Event<N>;

    /// Creates a [`Decoder`] used to interpret messages from the network.
    /// The `side` param indicates the connection side **from the node's perspective**.
    fn codec(&self, _peer_addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Default::default()
    }

    /// Processes a message received from the network.
    async fn process_message(&self, peer_addr: SocketAddr, message: Self::Message) -> io::Result<()> {
        if matches!(message, Event::BlockRequest(_) | Event::BlockResponse(_)) {
            let self_ = self.clone();
            // Handle BlockRequest and BlockResponse messages in a separate task to not block the
            // inbound queue.
            tokio::spawn(async move {
                self_.process_message_inner(peer_addr, message).await;
            });
        } else {
            self.process_message_inner(peer_addr, message).await;
        }
        Ok(())
    }

    /// Computes the depth of per-connection queues used to process inbound messages, sufficient to process the maximum expected load at any given moment.
    /// The greater it is, the more inbound messages the node can enqueue, but a too large value can make the node more susceptible to DoS attacks.
    /// See [`per_connection_queue_depth`] for the derivation.
    fn message_queue_depth(&self) -> usize {
        per_connection_queue_depth::<N>()
    }
}

#[async_trait]
impl<N: Network> Writing for Gateway<N> {
    type Codec = EventCodec<N>;
    type Message = Event<N>;

    /// Creates an [`Encoder`] used to write the outbound messages to the target stream.
    /// The `side` parameter indicates the connection side **from the node's perspective**.
    fn codec(&self, _peer_addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Default::default()
    }

    /// Computes the depth of per-connection queues used to send outbound messages, sufficient to process the maximum expected load at any given moment.
    /// The greater it is, the more outbound messages the node can enqueue. A too large value might obscure potential issues with your implementation
    /// (like slow serialization) or network, and lets a peer that stops reading its socket pin an unreasonable amount of our heap.
    /// See [`per_connection_queue_depth`] for the derivation.
    fn message_queue_depth(&self) -> usize {
        per_connection_queue_depth::<N>()
    }
}

#[async_trait]
impl<N: Network> Disconnect for Gateway<N> {
    /// Any extra operations to be performed during a disconnect.
    async fn handle_disconnect(&self, peer_addr: SocketAddr, origin: DisconnectOrigin) {
        debug!("Physically disconnecting from {peer_addr}; origin: {origin:?}");

        if let Some(peer_ip) = self.resolve_to_listener(&peer_addr) {
            // TODO(kaimast): This can, in theory, still lead to race conditions, if we immediately reconnect to the same peer.
            // In practice, there should always be a significant delay between those two delays, so it is not an immediate issue.
            //
            // To properly fix this, we either needk hold a lock here, or add a dedicated "disconnecting" state, so that
            // a peer is not re-added while the rest of the disconnect logic is running.
            let was_fully_connected = self.downgrade_peer_to_candidate(peer_ip);

            // Remove the peer from the sync module. Except for some tests, there is always a sync sender.
            if was_fully_connected && let Some(sync_sender) = self.sync_sender.get() {
                let (tx, rx) = oneshot::channel();

                if let Err(err) = sync_sender.tx_block_sync_remove_peer.send((peer_ip, tx)).await {
                    let err: anyhow::Error = err.into();
                    let err =
                        err.context(format!("Unable to remove disconnecting peer '{peer_ip}' from the sync module"));
                    warn!("{CONTEXT} {}", flatten_error(err));
                }

                if let Err(err) = rx.await {
                    let err: anyhow::Error = err.into();
                    let err =
                        err.context(format!("Unable to remove disconnecting peer '{peer_ip}' from the sync module"));
                    warn!("{CONTEXT} {}", flatten_error(err));
                }
            }
            // We don't clear this map based on time but only on peer disconnect.
            // This is sufficient to avoid infinite growth as the committee has a fixed number
            // of members.
            self.cache.clear_outbound_validators_requests(peer_ip);
            self.cache.clear_outbound_block_requests(peer_ip);
        } else {
            warn!("{CONTEXT} Got disconnect for a peer '{peer_addr}' that is not in the peer pool");
        }
    }
}

#[async_trait]
impl<N: Network> OnConnect for Gateway<N> {
    async fn on_connect(&self, peer_addr: SocketAddr) {
        if let Some(listener_addr) = self.resolve_to_listener(&peer_addr) {
            if let Some(peer) = self.get_connected_peer(listener_addr) {
                if peer.node_type == NodeType::BootstrapClient {
                    self.cache.increment_outbound_validators_requests(listener_addr);
                    let _ =
                        <Self as Transport<N>>::send(self, listener_addr, Event::ValidatorsRequest(ValidatorsRequest))
                            .await;
                }
            }
        }
    }
}

#[async_trait]
impl<N: Network> Handshake for Gateway<N> {
    /// Performs the handshake protocol.
    async fn perform_handshake(&self, mut connection: Connection) -> Result<Connection, ConnectError> {
        // Perform the handshake.
        let peer_addr = connection.addr();
        let peer_side = connection.side();

        // Check (or impose) IP-level bans.
        #[cfg(not(test))]
        if self.dev().is_none() && peer_side == ConnectionSide::Initiator {
            // If the IP is already banned reject the connection.
            if self.is_ip_banned(peer_addr.ip()) {
                trace!("{CONTEXT} Rejected a connection request from banned IP '{}'", peer_addr.ip());
                return Err(ConnectError::BannedIp { ip: peer_addr.ip() });
            }

            let num_attempts = self.cache.insert_inbound_connection(peer_addr.ip(), CONNECTION_ATTEMPTS_SINCE_SECS);

            debug!("Number of connection attempts from '{}': {}", peer_addr.ip(), num_attempts);
            if num_attempts > MAX_CONNECTION_ATTEMPTS {
                self.update_ip_ban(peer_addr.ip());
                trace!("{CONTEXT} Rejected a consecutive connection request from IP '{}'", peer_addr.ip());
                return Err(ConnectError::other(anyhow!("'{}' appears to be spamming connections", peer_addr.ip())));
            }
        }

        let stream = self.borrow_stream(&mut connection);
        // Make the socket more robust; every other handshake in the node does this too.
        harden_socket(stream)?;

        // If this is an inbound connection, we log it, but don't know the listening address yet.
        // Otherwise, we can immediately register the listening address.
        let mut listener_addr = if peer_side == ConnectionSide::Initiator {
            debug!("{CONTEXT} Received a connection request from '{peer_addr}'");
            None
        } else {
            debug!("{CONTEXT} Shaking hands with {peer_addr}...");
            Some(peer_addr)
        };

        // Retrieve the restrictions ID.
        let restrictions_id = self.ledger.latest_restrictions_id();

        // Perform the handshake; we pass on a mutable reference to peer_ip in case the process is broken at any point in time.
        //
        // The initiator picks the handshake protocol, gated on the block height so that validators
        // can be upgraded one at a time; the responder goes along with whichever one it is offered.
        let handshake_result = if peer_side == ConnectionSide::Responder {
            if self.initiates_noise_handshake() {
                write_noise_magic(stream).await?;
                self.handshake_inner_initiator_noise(peer_addr, restrictions_id, stream).await
            } else {
                self.handshake_inner_initiator(peer_addr, restrictions_id, stream).await
            }
        } else {
            match detect_handshake_protocol(stream).await? {
                (HandshakeProtocol::Noise, _) => {
                    self.handshake_inner_responder_noise(peer_addr, &mut listener_addr, restrictions_id, stream).await
                }
                (HandshakeProtocol::Legacy, _) if !self.accepts_legacy_handshake() => {
                    Err(ConnectError::other(format!("'{peer_addr}' offered the legacy handshake, which has expired")))
                }
                (HandshakeProtocol::Legacy, prefix) => {
                    self.handshake_inner_responder(peer_addr, &mut listener_addr, restrictions_id, stream, &prefix)
                        .await
                }
            }
        };

        // Register the peer, or roll it back, if the handshake got far enough to learn its listening
        // address.
        match (&handshake_result, listener_addr) {
            (Ok(peer_info), Some(addr)) => {
                let node_type = if bootstrap_peers::<N>(self.is_dev()).contains(&addr) {
                    NodeType::BootstrapClient
                } else {
                    NodeType::Validator
                };

                let mut peer_pool = self.peer_pool.write();

                // Validators may change their listening address, but not the Aleo address; traverse
                // the peer pool, and retain previously connected (the prior Aleo address is known)
                // candidate peers with the same Aleo address only if their listening address is the
                // same; otherwise, it may be concluded that a known validator has changed their
                // listening address, and thus the old entry should be removed as outdated.
                peer_pool.retain(|_, peer| {
                    if let Peer::Candidate(peer) = peer
                        && let Some(old_aleo_addr) = peer.last_known_aleo_addr
                    {
                        old_aleo_addr != peer_info.address || peer.listener_addr == addr
                    } else {
                        true
                    }
                });

                if let Some(peer) = peer_pool.get_mut(&addr) {
                    self.resolver.write().insert_peer(addr, peer_addr, Some(peer_info.address));
                    peer.upgrade_to_connected(
                        peer_addr,
                        peer_info.listener_port,
                        peer_info.address,
                        node_type,
                        peer_info.version,
                        peer_info.snarkos_sha,
                        ConnectionMode::Gateway,
                    );
                }
                info!("{CONTEXT} Connected to '{addr}'");
            }
            (Err(_), Some(addr)) => {
                if let Some(peer) = self.peer_pool.write().get_mut(&addr) {
                    // The peer may only be downgraded if it's a ConnectingPeer.
                    if peer.is_connecting() {
                        peer.downgrade_to_candidate(addr);
                    }
                }
            }
            // Neither handshake can succeed before it has learned the peer's listening address, so
            // this is unreachable; if it ever happened, the connection would go live with neither a
            // peer pool nor a resolver entry, and every event on it would be discarded as coming
            // from an unknown peer. Refuse it instead of leaving it in that state.
            (Ok(_), None) => {
                return Err(ConnectError::other(format!(
                    "the handshake with '{peer_addr}' succeeded without a listening address"
                )));
            }
            // The handshake failed before the peer's listening address was known, so there is nothing
            // in the pool to roll back.
            (Err(_), None) => {}
        }

        // Abort the connection on failure whether or not the peer reached the pool.
        handshake_result?;

        Ok(connection)
    }
}

/// A macro unwrapping the expected handshake event or returning an error for unexpected events.
macro_rules! expect_event {
    ($event_ty:path, $framed:expr, $peer_addr:expr) => {
        match $framed.try_next().await? {
            // Received the expected event, proceed.
            Some($event_ty(data)) => {
                trace!("{CONTEXT} Received '{}' from '{}'", data.name(), $peer_addr);
                data
            }
            // Received a disconnect event, abort.
            Some(Event::Disconnect($crate::events::Disconnect { reason })) => {
                return Err(ConnectError::other(format!("'{}' disconnected with reason \"{reason}\"", $peer_addr)));
            }
            // Received an unexpected event, abort.
            Some(ty) => {
                return Err(ConnectError::other(format!(
                    "'{}' did not follow the handshake protocol: received {:?} instead of {}",
                    $peer_addr,
                    ty.name(),
                    stringify!($msg_ty),
                )));
            }
            // Received nothing.
            None => return Err(ConnectError::IoError(io::ErrorKind::BrokenPipe.into())),
        }
    };
}

/// Send the given message to the peer.
async fn send_event<N: Network>(
    framed: &mut Framed<&mut TcpStream, EventCodec<N>>,
    peer_addr: SocketAddr,
    event: Event<N>,
) -> io::Result<()> {
    trace!("{CONTEXT} Sending '{}' to '{peer_addr}'", event.name());
    framed.send(event).await
}

/// Serializes a handshake payload for transmission inside a Noise message.
fn encode_payload<T: ToBytes>(payload: &T) -> Result<Vec<u8>, ConnectError> {
    snarkos_node_bft_events::encode_payload(payload)
}

/// Deserializes a handshake payload received inside a Noise message.
fn decode_payload<T: FromBytes>(peer_addr: SocketAddr, bytes: &[u8]) -> Result<T, ConnectError> {
    snarkos_node_bft_events::decode_payload(peer_addr, bytes)
}

/// Verifies a peer's proof that it owns the Aleo address it claims: its signature over the binding
/// message for this Noise session.
///
/// This is by far the most expensive step of the handshake, which is why both sides only reach it
/// once every cheap check has already passed.
#[must_use]
async fn verify_binding_signature<N: Network>(
    peer_addr: SocketAddr,
    signature: Data<Signature<N>>,
    address: Address<N>,
    binding: &[u8],
) -> Option<DisconnectReason> {
    // Perform the deferred non-blocking deserialization of the signature.
    let Ok(signature) = spawn_blocking!(signature.deserialize_blocking()) else {
        warn!("{CONTEXT} Handshake with '{peer_addr}' failed (cannot deserialize the signature)");
        return Some(DisconnectReason::InvalidChallengeResponse);
    };
    // Verify the signature.
    if !signature.verify_bytes(&address, binding) {
        warn!("{CONTEXT} Handshake with '{peer_addr}' failed (invalid signature)");
        return Some(DisconnectReason::InvalidChallengeResponse);
    }

    None
}

/// Concludes a Noise handshake, handing the stream back to the connection.
///
/// The stream deliberately goes back unframed. [`Reading`] builds a codec of its own, capped at the
/// 256 MiB an event may be rather than the 64 KiB a Noise message may be, and takes full
/// responsibility for the stream from here on. Handing it a bare stream is only sound because the
/// session reads its messages exactly: anything the peer pipelined behind the last handshake message
/// is still on the socket rather than in a buffer about to be dropped.
fn finish_noise_handshake(noise: NoiseSession<&mut TcpStream>) {
    // Note: the transport keys are discarded here, leaving the resulting connection unencrypted.
    let _stream = noise.into_inner();
}

/// Concludes a legacy handshake, dropping its codec along with anything the peer had pipelined behind
/// the last handshake message.
///
/// Unlike the Noise handshake, this one reads through a buffering codec, so it can pull bytes off the
/// socket that belong to the events which follow - and those are lost here, leaving the event codec to
/// start in the middle of a frame. That has always been the case, and carrying them across would mean
/// wrapping the stream for a protocol that is being retired, so it is logged rather than fixed.
fn note_legacy_handshake_end<N: Network>(framed: Framed<&mut TcpStream, EventCodec<N>>, peer_addr: SocketAddr) {
    let leftover = framed.into_parts().read_buf;

    if !leftover.is_empty() {
        debug!("{CONTEXT} Discarding {} bytes '{peer_addr}' sent before the handshake was over", leftover.len());
    }
}

/// Distills a legacy challenge request into the protocol-agnostic peer information.
///
/// The peer's restrictions ID is not part of its challenge request, but by the time this is called
/// `verify_challenge_response` has established that it matches the one passed in.
fn peer_info_from_challenge_request<N: Network>(
    request: ChallengeRequest<N>,
    restrictions_id: Field<N>,
) -> PeerInfo<N> {
    let ChallengeRequest { version, listener_port, address, nonce: _, snarkos_sha } = request;
    PeerInfo { version, listener_port, address, restrictions_id, snarkos_sha }
}

impl<N: Network> Gateway<N> {
    /// Returns `true` if this node should offer the Noise handshake when it dials the given peer.
    fn initiates_noise_handshake(&self) -> bool {
        // Tests pin the choice, so that both sides of the transition can be covered.
        if let Some(initiates) = self.pinned_handshake_protocol() {
            return initiates;
        }

        // Development nodes always take the new path, so that devnets exercise it.
        self.is_dev() || self.consensus_version_reached(NOISE_HANDSHAKE_ACTIVATION)
    }

    /// Returns `true` if this node still accepts the legacy handshake from a peer that dials it.
    ///
    /// Unlike the choice of what to offer, this is not pinned in tests and not forced in development:
    /// a converted node has to keep answering unconverted ones for the whole of the transition, and
    /// the tests covering that rely on it.
    fn accepts_legacy_handshake(&self) -> bool {
        !self.consensus_version_reached(LEGACY_HANDSHAKE_EXPIRY)
    }

    /// Returns `true` if the given consensus version is scheduled and the ledger has reached it.
    fn consensus_version_reached(&self, version: Option<ConsensusVersion>) -> bool {
        version.is_some_and(|version| {
            N::CONSENSUS_HEIGHT(version).is_ok_and(|height| self.ledger.latest_block_height() >= height)
        })
    }

    /// The pinned choice of handshake protocol; always `None` outside tests.
    #[cfg(not(any(test, feature = "test")))]
    fn pinned_handshake_protocol(&self) -> Option<bool> {
        None
    }

    /// The pinned choice of handshake protocol; see `InnerGateway::initiates_noise_handshake`.
    #[cfg(any(test, feature = "test"))]
    fn pinned_handshake_protocol(&self) -> Option<bool> {
        Some(self.initiates_noise_handshake.load(std::sync::atomic::Ordering::Relaxed))
    }

    /// Pins whether this node offers the Noise handshake when it dials, regardless of the activation
    /// height.
    ///
    /// This exists so that tests can cover the transition, during which a converted node still has
    /// to be able to shake hands with unconverted ones.
    #[cfg(any(test, feature = "test"))]
    pub fn set_initiates_noise_handshake(&self, initiates: bool) {
        self.initiates_noise_handshake.store(initiates, std::sync::atomic::Ordering::Relaxed);
    }

    /// Returns the snarkOS commit hash to disclose to a peer, if any.
    fn snarkos_sha(&self) -> Option<[u8; 40]> {
        let current_block_height = self.ledger.latest_block_height();
        let consensus_version = N::CONSENSUS_VERSION(current_block_height).unwrap();
        match (self.is_dev(), consensus_version >= ConsensusVersion::V12, get_repo_commit_hash()) {
            (true, _, Some(sha)) => Some(sha),
            (_, true, Some(sha)) => Some(sha),
            _ => None,
        }
    }

    /// The connection initiator side of the Noise handshake.
    ///
    /// The initiator does the expensive work first: it signs before the responder has committed to
    /// anything, and only learns whether it was accepted when the fourth message arrives. See
    /// [`Gateway::handshake_inner_responder_noise`] for the other half of that bargain.
    async fn handshake_inner_initiator_noise<'a>(
        &'a self,
        peer_addr: SocketAddr,
        restrictions_id: Field<N>,
        stream: &'a mut TcpStream,
    ) -> Result<PeerInfo<N>, ConnectError> {
        // Note who answered here last time, before the pool entry becomes a connecting one and stops
        // carrying it.
        let expected_address = match self.peer_pool.read().get(&peer_addr) {
            Some(Peer::Candidate(peer)) => peer.last_known_aleo_addr,
            _ => None,
        };

        // Introduce the peer into the peer pool.
        self.add_connecting_peer(peer_addr)?;

        let mut noise = NoiseSession::new(stream, Role::Initiator)?;
        let our_info =
            PeerInfo::new(self.local_ip().port(), self.account.address(), restrictions_id, self.snarkos_sha());

        /* Message 1: announce ourselves in the clear, so the responder can turn us away cheaply. */

        let hint = HandshakeHint {
            version: our_info.version,
            listener_port: our_info.listener_port,
            address: our_info.address,
        };
        noise.send(&encode_payload(&hint)?).await?;

        /* Message 2: receive the responder's metadata, which deliberately carries no signature. */

        let peer_info: PeerInfo<N> = decode_payload(peer_addr, &noise.recv().await?)?;

        // The handshake hash at this point already commits to both ephemeral keys, the responder's
        // static key and every payload exchanged so far, so a signature over it is only valid for
        // this session with this responder, and cannot be relayed into another one.
        let binding = binding_message(HANDSHAKE_DOMAIN, Role::Initiator, &noise.handshake_hash()?);

        // Check the peer over before signing anything for it.
        if let Some(reason) = self.verify_peer_info(peer_addr, &peer_info, restrictions_id) {
            return Err(reason.into_connect_error(peer_addr));
        }

        // A validator is expected to keep its Aleo address and change its listening address rather
        // than the other way around (the peer pool is pruned on that assumption), so a mismatch here
        // is not a peer that has legitimately moved.
        if let Some(expected) = expected_address
            && peer_info.address != expected
        {
            warn!("{CONTEXT} Dropping '{peer_addr}': expected validator {expected}, got {}", peer_info.address);
            return Err(DisconnectReason::InvalidChallengeResponse.into_connect_error(peer_addr));
        }

        /* Message 3: disclose ourselves and prove that we own the Aleo address we claim. */

        let Ok(our_signature) = self.account.sign_bytes(&binding, &mut rand::rng()) else {
            return Err(ConnectError::other(anyhow!("Failed to sign the handshake binding")));
        };
        let our_message = InitiatorInfo { info: our_info, signature: Data::Object(our_signature) };
        noise.send(&encode_payload(&our_message)?).await?;

        // Capture the binding for the responder's proof before the hash becomes unavailable; it
        // additionally commits to our static key and to the message we have just sent.
        let peer_binding = binding_message(HANDSHAKE_DOMAIN, Role::Responder, &noise.handshake_hash()?);

        /* Message 4: receive the responder's verdict. */

        let mut noise = noise.into_transport_mode()?;
        let verdict = decode_payload::<ResponderProof<N>>(peer_addr, &noise.recv().await?)?;

        // The pattern is over, so the stream goes back to the connection. The responder considers the
        // handshake done the moment it sent this message and is free to start sending events while we
        // are still verifying below; those bytes wait on the socket for the event codec.
        finish_noise_handshake(noise);

        let peer_signature = match verdict {
            ResponderProof::Accepted { signature } => signature,
            ResponderProof::Rejected { reason } => {
                warn!("{CONTEXT} '{peer_addr}' rejected the handshake with reason \"{reason}\"");
                return Err(reason.into_connect_error(peer_addr));
            }
        };

        if let Some(reason) =
            verify_binding_signature(peer_addr, peer_signature, peer_info.address, &peer_binding).await
        {
            return Err(reason.into_connect_error(peer_addr));
        }

        Ok(peer_info)
    }

    /// The connection responder side of the Noise handshake.
    ///
    /// Every expensive operation is deferred for as long as the protocol allows: the responder
    /// verifies a signature only once the initiator's authenticated metadata has passed all of the
    /// cheap checks, and produces one only once that verification has succeeded.
    ///
    /// The legacy handshake also runs its cheap checks first, so a peer that fails one of those was
    /// never expensive under either protocol. What changes is the price of *claiming* an identity
    /// that passes them - committee membership is public, so anyone can claim it. Under the legacy
    /// handshake that claim alone bought a signature from this node, for the cost of one packet.
    /// Here it buys a handful of Diffie-Hellman operations and a signature verification; extracting
    /// a signature requires actually holding the committee key, and even reaching the verification
    /// requires completing the pattern, which a peer that cannot receive our reply - a spoofed
    /// source address - cannot do.
    async fn handshake_inner_responder_noise<'a>(
        &'a self,
        peer_addr: SocketAddr,
        peer_ip: &mut Option<SocketAddr>,
        restrictions_id: Field<N>,
        stream: &'a mut TcpStream,
    ) -> Result<PeerInfo<N>, ConnectError> {
        /* Message 1: the peer's cleartext hint. Everything it claims is re-checked in message 3. */

        // The first message is read without deriving any keys, so that everything below costs the
        // responder no more than parsing and lookups until it has decided the peer is worth talking
        // to.
        let pending = PendingSession::accept(stream).await?;
        let hint: HandshakeHint<N> = decode_payload(peer_addr, pending.first_payload()?)?;
        let listener_addr = SocketAddr::new(peer_addr.ip(), hint.listener_port);

        // Turn the peer away before performing any Diffie-Hellman if we already know we do not want
        // it. Nothing here is trustworthy yet - message 3 runs all of it again against the
        // authenticated copy - but every one of these is a peer that would be refused either way,
        // and the committee check in particular is one an attacker cannot talk its way past.
        if let Err(reason) = self.ensure_peer_is_allowed(listener_addr) {
            return Err(reason.into_connect_error(peer_addr));
        }
        if let Some(reason) = self.verify_peer_identity(peer_addr, hint.version, hint.listener_port, hint.address) {
            return Err(reason.into_connect_error(peer_addr));
        }

        // The peer pool is the only thing mutated here, so it goes last: there is no point admitting
        // a peer that one of the checks above was about to turn away. Recording the listening address
        // only once that has succeeded also stops a peer claiming somebody else's port from getting
        // their entry downgraded by failing this handshake.
        self.add_connecting_peer(listener_addr)?;
        *peer_ip = Some(listener_addr);

        /* Message 2: disclose ourselves, but do not sign anything yet. */

        // The peer has passed every free check, so it is now worth deriving keys for.
        let mut noise = pending.into_session()?;

        let our_info =
            PeerInfo::new(self.local_ip().port(), self.account.address(), restrictions_id, self.snarkos_sha());
        noise.send(&encode_payload(&our_info)?).await?;

        // The binding the initiator is expected to have signed.
        let peer_binding = binding_message(HANDSHAKE_DOMAIN, Role::Initiator, &noise.handshake_hash()?);

        /* Message 3: the peer's authenticated metadata and its proof of identity. */

        let InitiatorInfo { info: peer_info, signature: peer_signature } =
            decode_payload::<InitiatorInfo<N>>(peer_addr, &noise.recv().await?)?;

        // Our own binding additionally commits to the peer's static key and to the message it has
        // just sent, so it can only be captured after that message has been processed.
        let binding = binding_message(HANDSHAKE_DOMAIN, Role::Responder, &noise.handshake_hash()?);
        let mut noise = noise.into_transport_mode()?;

        // The cleartext hint is a claim, not a fact: reject the peer if it does not match what it
        // has now authenticated, as otherwise the hint would be a way to bypass the checks above.
        if (hint.version, hint.listener_port, hint.address)
            != (peer_info.version, peer_info.listener_port, peer_info.address)
        {
            warn!("{CONTEXT} Handshake with '{peer_addr}' failed (the handshake hint was contradicted)");
            return self.reject_noise_handshake(peer_addr, noise, DisconnectReason::ProtocolViolation).await;
        }

        // Everything below is a lookup or a comparison; only once all of it passes is the peer
        // worth the cost of a signature verification.
        if let Some(reason) = self.verify_peer_info(peer_addr, &peer_info, restrictions_id) {
            return self.reject_noise_handshake(peer_addr, noise, reason).await;
        }

        /* Message 4: having checked the peer over, verify its proof and produce our own. */

        if let Some(reason) =
            verify_binding_signature(peer_addr, peer_signature, peer_info.address, &peer_binding).await
        {
            return self.reject_noise_handshake(peer_addr, noise, reason).await;
        }

        let Ok(our_signature) = self.account.sign_bytes(&binding, &mut rand::rng()) else {
            return Err(ConnectError::other(anyhow!("Failed to sign the handshake binding")));
        };
        noise.send(&encode_payload(&ResponderProof::Accepted { signature: Data::Object(our_signature) })?).await?;

        finish_noise_handshake(noise);

        Ok(peer_info)
    }

    /// Tells the initiator why it was turned away, and fails the handshake with that reason.
    async fn reject_noise_handshake(
        &self,
        peer_addr: SocketAddr,
        mut noise: NoiseSession<&mut TcpStream>,
        reason: DisconnectReason,
    ) -> Result<PeerInfo<N>, ConnectError> {
        noise.send(&encode_payload(&ResponderProof::<N>::Rejected { reason })?).await?;

        Err(reason.into_connect_error(peer_addr))
    }

    /// The connection initiator side of the legacy handshake.
    async fn handshake_inner_initiator<'a>(
        &'a self,
        peer_addr: SocketAddr,
        restrictions_id: Field<N>,
        stream: &'a mut TcpStream,
    ) -> Result<PeerInfo<N>, ConnectError> {
        // Introduce the peer into the peer pool.
        self.add_connecting_peer(peer_addr)?;

        // Construct the stream.
        let mut framed = Framed::new(stream, EventCodec::<N>::handshake());

        /* Step 1: Send the challenge request. */

        // Sample a random nonce.
        let our_nonce: u64 = rand::random();
        // Determine the snarkOS SHA to send to the peer.
        let snarkos_sha = self.snarkos_sha();
        // Send a challenge request to the peer.
        let our_request = ChallengeRequest::new(self.local_ip().port(), self.account.address(), our_nonce, snarkos_sha);
        send_event(&mut framed, peer_addr, Event::ChallengeRequest(our_request)).await?;

        /* Step 2: Receive the peer's challenge response followed by the challenge request. */

        // Listen for the challenge response message.
        let peer_response = expect_event!(Event::ChallengeResponse, framed, peer_addr);
        // Listen for the challenge request message.
        let peer_request = expect_event!(Event::ChallengeRequest, framed, peer_addr);

        // Verify the challenge response. If a disconnect reason was returned, send the disconnect message and abort.
        if let Some(reason) = self
            .verify_challenge_response(peer_addr, peer_request.address, peer_response, restrictions_id, our_nonce)
            .await
        {
            send_event(&mut framed, peer_addr, reason.into()).await?;
            return Err(ConnectError::application(reason));
        }

        // Verify the challenge request. If a disconnect reason was returned, send the disconnect message and abort.
        if let Some(reason) = self.verify_challenge_request(peer_addr, &peer_request) {
            send_event(&mut framed, peer_addr, reason.into()).await?;
            return Err(reason.into_connect_error(peer_addr));
        }

        /* Step 3: Send the challenge response. */

        // Sign the counterparty nonce.
        let response_nonce: u64 = rand::random();
        let data = [peer_request.nonce.to_le_bytes(), response_nonce.to_le_bytes()].concat();
        let Ok(our_signature) = self.account.sign_bytes(&data, &mut rand::rng()) else {
            return Err(ConnectError::other(anyhow!("Failed to sign the challenge request nonce")));
        };
        // Send the challenge response.
        let our_response =
            ChallengeResponse { restrictions_id, signature: Data::Object(our_signature), nonce: response_nonce };
        send_event(&mut framed, peer_addr, Event::ChallengeResponse(our_response)).await?;

        note_legacy_handshake_end(framed, peer_addr);

        Ok(peer_info_from_challenge_request(peer_request, restrictions_id))
    }

    /// The connection responder side of the legacy handshake.
    ///
    /// `prefix` holds the bytes that were consumed from the stream while determining which
    /// handshake protocol the peer speaks; they are the beginning of its first frame.
    async fn handshake_inner_responder<'a>(
        &'a self,
        peer_addr: SocketAddr,
        peer_ip: &mut Option<SocketAddr>,
        restrictions_id: Field<N>,
        stream: &'a mut TcpStream,
        prefix: &[u8],
    ) -> Result<PeerInfo<N>, ConnectError> {
        // Construct the stream.
        let mut framed = prepare_framed(stream, EventCodec::<N>::handshake(), prefix);

        /* Step 1: Receive the challenge request. */

        // Listen for the challenge request message.
        let peer_request = expect_event!(Event::ChallengeRequest, framed, peer_addr);

        // Ensure the address is not the same as this node.
        if self.account.address() == peer_request.address {
            return Err(ConnectError::SelfConnect { address: peer_addr });
        }

        // Obtain the peer's listening address.
        *peer_ip = Some(SocketAddr::new(peer_addr.ip(), peer_request.listener_port));
        let peer_ip = peer_ip.unwrap();

        // Knowing the peer's listening address, ensure it is allowed to connect.
        if let Err(reason) = self.ensure_peer_is_allowed(peer_ip) {
            send_event(&mut framed, peer_addr, reason.into()).await?;
            return Err(reason.into_connect_error(peer_addr));
        }

        // Introduce the peer into the peer pool.
        self.add_connecting_peer(peer_ip)?;

        // Verify the challenge request. If a disconnect reason was returned, send the disconnect message and abort.
        if let Some(reason) = self.verify_challenge_request(peer_addr, &peer_request) {
            send_event(&mut framed, peer_addr, reason.into()).await?;
            return Err(reason.into_connect_error(peer_addr));
        }

        /* Step 2: Send the challenge response followed by own challenge request. */

        // Sign the counterparty nonce.
        let response_nonce: u64 = rand::random();
        let data = [peer_request.nonce.to_le_bytes(), response_nonce.to_le_bytes()].concat();
        let Ok(our_signature) = self.account.sign_bytes(&data, &mut rand::rng()) else {
            return Err(ConnectError::other(anyhow!("Failed to sign the challenge request nonce")));
        };
        // Send the challenge response.
        let our_response =
            ChallengeResponse { restrictions_id, signature: Data::Object(our_signature), nonce: response_nonce };
        send_event(&mut framed, peer_addr, Event::ChallengeResponse(our_response)).await?;

        // Sample a random nonce.
        let our_nonce: u64 = rand::random();
        // Determine the snarkOS SHA to send to the peer.
        let snarkos_sha = self.snarkos_sha();
        // Send the challenge request.
        let our_request = ChallengeRequest::new(self.local_ip().port(), self.account.address(), our_nonce, snarkos_sha);
        send_event(&mut framed, peer_addr, Event::ChallengeRequest(our_request)).await?;

        /* Step 3: Receive the challenge response. */

        // Listen for the challenge response message.
        let peer_response = expect_event!(Event::ChallengeResponse, framed, peer_addr);
        // Verify the challenge response. If a disconnect reason was returned, send the disconnect message and abort.
        if let Some(reason) = self
            .verify_challenge_response(peer_addr, peer_request.address, peer_response, restrictions_id, our_nonce)
            .await
        {
            send_event(&mut framed, peer_addr, reason.into()).await?;
            Err(reason.into_connect_error(peer_addr))
        } else {
            note_legacy_handshake_end(framed, peer_addr);

            Ok(peer_info_from_challenge_request(peer_request, restrictions_id))
        }
    }

    /// Verifies the given challenge request. Returns a disconnect reason if the request is invalid.
    #[must_use]
    fn verify_challenge_request(&self, peer_addr: SocketAddr, event: &ChallengeRequest<N>) -> Option<DisconnectReason> {
        // Retrieve the components of the challenge request.
        let &ChallengeRequest { version, listener_port, address, nonce: _, ref snarkos_sha } = event;
        log_repo_sha_comparison(peer_addr, snarkos_sha, CONTEXT);

        self.verify_peer_claims(peer_addr, version, listener_port, address)
    }

    /// Verifies the metadata a peer disclosed during the Noise handshake. Returns a disconnect
    /// reason if the peer is not acceptable.
    ///
    /// Every check reachable from here is a lookup or a comparison, which is precisely what makes
    /// it safe to run before verifying the peer's signature.
    #[must_use]
    fn verify_peer_info(
        &self,
        peer_addr: SocketAddr,
        info: &PeerInfo<N>,
        expected_restrictions_id: Field<N>,
    ) -> Option<DisconnectReason> {
        log_repo_sha_comparison(peer_addr, &info.snarkos_sha, CONTEXT);

        // Verify the restrictions ID. This is the only check here that the cleartext hint cannot
        // carry, as the peer would simply state the expected value; it is left to this point.
        if info.restrictions_id != expected_restrictions_id {
            warn!("{CONTEXT} Handshake with '{peer_addr}' failed (incorrect restrictions ID)");
            return Some(DisconnectReason::InvalidChallengeResponse);
        }

        self.verify_peer_identity(peer_addr, info.version, info.listener_port, info.address)
    }

    /// Everything a peer's claimed identity can be held to without any cryptography.
    ///
    /// These are all lookups and comparisons, which is what makes them safe to run against the
    /// unauthenticated hint in the Noise handshake's first message, and worth running there: a peer
    /// that fails one of them would fail it again against the authenticated copy, so there is no
    /// reason to spend a Diffie-Hellman on it first.
    #[must_use]
    fn verify_peer_identity(
        &self,
        peer_addr: SocketAddr,
        version: u32,
        listener_port: u16,
        address: Address<N>,
    ) -> Option<DisconnectReason> {
        // Ensure the address is not the same as this node's.
        if self.account.address() == address {
            return Some(DisconnectReason::SelfConnect);
        }

        self.verify_peer_claims(peer_addr, version, listener_port, address)
    }

    /// The peer checks shared by both handshakes.
    #[must_use]
    fn verify_peer_claims(
        &self,
        peer_addr: SocketAddr,
        version: u32,
        listener_port: u16,
        address: Address<N>,
    ) -> Option<DisconnectReason> {
        let listener_addr = SocketAddr::new(peer_addr.ip(), listener_port);

        // Ensure the event protocol version is not outdated.
        if version < Event::<N>::VERSION {
            return Some(DisconnectReason::OutdatedClientVersion);
        }
        // If the node is in trusted peers only mode, ensure the peer is trusted.
        if self.trusted_peers_only && !self.is_trusted(listener_addr) {
            warn!("{CONTEXT} Dropping '{peer_addr}' for being an untrusted validator ({address})");
            return Some(DisconnectReason::NoExternalPeersAllowed);
        }
        if !bootstrap_peers::<N>(self.dev().is_some()).contains(&listener_addr) {
            // Ensure the address is a current committee member.
            if !self.is_authorized_validator_address(address) {
                return Some(DisconnectReason::UnauthorizedValidator);
            }
        }

        // Ensure the address is not already connected.
        if self.is_connected_address(address) {
            return Some(DisconnectReason::AlreadyConnectedToAleoAddress);
        }

        None
    }

    /// Verifies the given challenge response. Returns a disconnect reason if the response is invalid.
    #[must_use]
    async fn verify_challenge_response(
        &self,
        peer_addr: SocketAddr,
        peer_address: Address<N>,
        response: ChallengeResponse<N>,
        expected_restrictions_id: Field<N>,
        expected_nonce: u64,
    ) -> Option<DisconnectReason> {
        // Retrieve the components of the challenge response.
        let ChallengeResponse { restrictions_id, signature, nonce } = response;

        // Verify the restrictions ID.
        if restrictions_id != expected_restrictions_id {
            warn!("{CONTEXT} Handshake with '{peer_addr}' failed (incorrect restrictions ID)");
            return Some(DisconnectReason::InvalidChallengeResponse);
        }
        // Perform the deferred non-blocking deserialization of the signature.
        let Ok(signature) = spawn_blocking!(signature.deserialize_blocking()) else {
            warn!("{CONTEXT} Handshake with '{peer_addr}' failed (cannot deserialize the signature)");
            return Some(DisconnectReason::InvalidChallengeResponse);
        };
        // Verify the signature.
        if !signature.verify_bytes(&peer_address, &[expected_nonce.to_le_bytes(), nonce.to_le_bytes()].concat()) {
            warn!("{CONTEXT} Handshake with '{peer_addr}' failed (invalid signature)");
            return Some(DisconnectReason::InvalidChallengeResponse);
        }
        None
    }
}

#[cfg(test)]
mod prop_tests {
    use crate::{
        Gateway,
        MAX_WORKERS,
        MEMORY_POOL_PORT,
        Worker,
        helpers::{Storage, init_primary_channels, init_worker_channels},
    };

    use snarkos_account::Account;
    use snarkos_node_bft_ledger_service::MockLedgerService;
    use snarkos_node_bft_storage_service::BFTMemoryService;
    use snarkos_node_network::PeerPoolHandling;
    use snarkos_node_tcp::P2P;
    use snarkos_utilities::NodeDataDir;

    use snarkos_node_bft_events::committee_prop_tests::{CommitteeContext, ValidatorSet};
    use snarkvm::{
        ledger::{
            committee::{Committee, test_helpers::sample_committee_for_round_and_members},
            narwhal::{BatchHeader, batch_certificate::test_helpers::sample_batch_certificate_for_round},
        },
        prelude::{MainnetV0, PrivateKey},
        utilities::TestRng,
    };

    use indexmap::{IndexMap, IndexSet};
    use proptest::{
        prelude::{Arbitrary, BoxedStrategy, Just, Strategy, any, any_with},
        sample::Selector,
    };
    use std::{
        fmt::{Debug, Formatter},
        net::{IpAddr, Ipv4Addr, SocketAddr},
        sync::Arc,
    };
    use test_strategy::proptest;

    type CurrentNetwork = MainnetV0;

    /// The per-connection queues are sized from the fetch-timeout window, not the GC window.
    ///
    /// Each slot can hold a full `Transmission`, so the depth is a direct multiplier on the heap a
    /// single peer can pin by not reading its socket. Pin the derivation so a regression is loud.
    #[test]
    fn test_per_connection_queue_depth() {
        use crate::gateway::{QUEUE_WINDOW_ROUNDS, per_connection_queue_depth};
        use snarkvm::console::network::Network;

        // The window is `MAX_FETCH_TIMEOUT` expressed in rounds.
        assert_eq!(QUEUE_WINDOW_ROUNDS, 3);

        let certificates = CurrentNetwork::LATEST_MAX_CERTIFICATES() as usize;
        let transmissions = BatchHeader::<CurrentNetwork>::MAX_TRANSMISSIONS_PER_BATCH;
        let depth = per_connection_queue_depth::<CurrentNetwork>();

        assert_eq!(depth, 2 * QUEUE_WINDOW_ROUNDS * certificates * (transmissions + 1));

        // It must cover a peer fetching every transmission of every certificate for the window...
        assert!(depth >= QUEUE_WINDOW_ROUNDS * certificates * transmissions);
        // ...but stay far below the old `MAX_GC_ROUNDS`-derived depth, which let one peer pin
        // 400,000 transmissions.
        assert!(depth < 2 * BatchHeader::<CurrentNetwork>::MAX_GC_ROUNDS * certificates * transmissions / 8);
    }

    impl Debug for Gateway<CurrentNetwork> {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            // TODO implement Debug properly and move it over to production code
            f.debug_tuple("Gateway").field(&self.account.address()).field(&self.tcp.config()).finish()
        }
    }

    #[derive(Debug, test_strategy::Arbitrary)]
    enum GatewayAddress {
        Dev(u8),
        Prod(Option<SocketAddr>),
    }

    impl GatewayAddress {
        fn ip(&self) -> Option<SocketAddr> {
            if let GatewayAddress::Prod(ip) = self {
                return *ip;
            }
            None
        }

        fn port(&self) -> Option<u16> {
            if let GatewayAddress::Dev(port) = self {
                return Some(*port as u16);
            }
            None
        }
    }

    impl Arbitrary for Gateway<CurrentNetwork> {
        type Parameters = ();
        type Strategy = BoxedStrategy<Gateway<CurrentNetwork>>;

        fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
            any_valid_dev_gateway()
                .prop_map(|(storage, _, private_key, address)| {
                    Gateway::new(
                        Account::try_from(private_key).unwrap(),
                        storage.clone(),
                        storage.ledger().clone(),
                        address.ip(),
                        &[],
                        false,
                        NodeDataDir::new_test(None),
                        address.port(),
                    )
                    .unwrap()
                })
                .boxed()
        }
    }

    type GatewayInput = (Storage<CurrentNetwork>, CommitteeContext, PrivateKey<CurrentNetwork>, GatewayAddress);

    fn any_valid_dev_gateway() -> BoxedStrategy<GatewayInput> {
        (any::<CommitteeContext>(), any::<Selector>())
            .prop_flat_map(|(context, account_selector)| {
                let CommitteeContext(_, ValidatorSet(validators)) = context.clone();
                (
                    any_with::<Storage<CurrentNetwork>>(context.clone()),
                    Just(context),
                    Just(account_selector.select(validators)),
                    0u8..,
                )
                    .prop_map(|(a, b, c, d)| (a, b, c.private_key, GatewayAddress::Dev(d)))
            })
            .boxed()
    }

    fn any_valid_prod_gateway() -> BoxedStrategy<GatewayInput> {
        (any::<CommitteeContext>(), any::<Selector>())
            .prop_flat_map(|(context, account_selector)| {
                let CommitteeContext(_, ValidatorSet(validators)) = context.clone();
                (
                    any_with::<Storage<CurrentNetwork>>(context.clone()),
                    Just(context),
                    Just(account_selector.select(validators)),
                    any::<Option<SocketAddr>>(),
                )
                    .prop_map(|(a, b, c, d)| (a, b, c.private_key, GatewayAddress::Prod(d)))
            })
            .boxed()
    }

    #[proptest]
    fn gateway_dev_initialization(#[strategy(any_valid_dev_gateway())] input: GatewayInput) {
        let (storage, _, private_key, dev) = input;
        let account = Account::try_from(private_key).unwrap();

        let gateway = Gateway::new(
            account.clone(),
            storage.clone(),
            storage.ledger().clone(),
            dev.ip(),
            &[],
            false,
            NodeDataDir::new_test(None),
            dev.port(),
        )
        .unwrap();
        let tcp_config = gateway.tcp().config();
        assert_eq!(tcp_config.listener_ip, Some(IpAddr::V4(Ipv4Addr::LOCALHOST)));
        assert_eq!(tcp_config.desired_listening_port, Some(MEMORY_POOL_PORT + dev.port().unwrap()));

        let tcp_config = gateway.tcp().config();
        assert_eq!(tcp_config.max_connections, Committee::<CurrentNetwork>::max_committee_size() * 10);
        assert_eq!(gateway.account().address(), account.address());
    }

    #[proptest]
    fn gateway_prod_initialization(#[strategy(any_valid_prod_gateway())] input: GatewayInput) {
        let (storage, _, private_key, dev) = input;
        let account = Account::try_from(private_key).unwrap();

        let gateway = Gateway::new(
            account.clone(),
            storage.clone(),
            storage.ledger().clone(),
            dev.ip(),
            &[],
            false,
            NodeDataDir::new_test(None),
            dev.port(),
        )
        .unwrap();
        let tcp_config = gateway.tcp().config();
        if let Some(socket_addr) = dev.ip() {
            assert_eq!(tcp_config.listener_ip, Some(socket_addr.ip()));
            assert_eq!(tcp_config.desired_listening_port, Some(socket_addr.port()));
        } else {
            assert_eq!(tcp_config.listener_ip, Some(IpAddr::V4(Ipv4Addr::UNSPECIFIED)));
            assert_eq!(tcp_config.desired_listening_port, Some(MEMORY_POOL_PORT));
        }

        let tcp_config = gateway.tcp().config();
        assert_eq!(tcp_config.max_connections, Committee::<CurrentNetwork>::max_committee_size() * 10);
        assert_eq!(gateway.account().address(), account.address());
    }

    #[proptest(async = "tokio")]
    async fn gateway_start(
        #[strategy(any_valid_dev_gateway())] input: GatewayInput,
        #[strategy(0..MAX_WORKERS)] workers_count: u8,
    ) {
        let (storage, committee, private_key, dev) = input;
        let committee = committee.0;
        let worker_storage = storage.clone();
        let account = Account::try_from(private_key).unwrap();

        let gateway = Gateway::new(
            account,
            storage.clone(),
            storage.ledger().clone(),
            dev.ip(),
            &[],
            false,
            NodeDataDir::new_test(None),
            dev.port(),
        )
        .unwrap();

        let (primary_sender, _) = init_primary_channels();

        let (workers, worker_senders) = {
            // Construct a map of the worker senders.
            let mut tx_workers = IndexMap::new();
            let mut workers = IndexMap::new();

            // Initialize the workers.
            for id in 0..workers_count {
                // Construct the worker channels.
                let (tx_worker, rx_worker) = init_worker_channels();
                // Construct the worker instance.
                let ledger = Arc::new(MockLedgerService::new(committee.clone()));
                let worker =
                    Worker::new(id, Arc::new(gateway.clone()), worker_storage.clone(), ledger, Default::default())
                        .unwrap();
                // Run the worker instance.
                worker.run(rx_worker);

                // Add the worker and the worker sender to maps
                workers.insert(id, worker);
                tx_workers.insert(id, tx_worker);
            }
            (workers, tx_workers)
        };

        gateway.run(primary_sender, worker_senders, None).await;
        assert_eq!(
            gateway.local_ip(),
            SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), MEMORY_POOL_PORT + dev.port().unwrap())
        );
        assert_eq!(gateway.num_workers(), workers.len() as u8);
    }

    #[proptest]
    fn test_is_authorized_validator(#[strategy(any_valid_dev_gateway())] input: GatewayInput) {
        let rng = &mut TestRng::default();

        // Initialize the round parameters.
        let current_round = 2;
        let committee_size = 4;
        let max_gc_rounds = BatchHeader::<CurrentNetwork>::MAX_GC_ROUNDS as u64;
        let (_, _, private_key, dev) = input;
        let account = Account::try_from(private_key).unwrap();

        // Sample the certificates.
        let mut certificates = IndexSet::new();
        for _ in 0..committee_size {
            certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        }
        let addresses: Vec<_> = certificates.iter().map(|certificate| certificate.author()).collect();
        // Initialize the committee.
        let committee = sample_committee_for_round_and_members(current_round, addresses, rng);
        // Sample extra certificates from non-committee members.
        for _ in 0..committee_size {
            certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        }
        // Initialize the ledger.
        let ledger = Arc::new(MockLedgerService::new(committee.clone()));
        // Initialize the storage.
        let storage = Storage::new(ledger.clone(), Arc::new(BFTMemoryService::new()), max_gc_rounds).unwrap();
        // Initialize the gateway.
        let gateway = Gateway::new(
            account.clone(),
            storage.clone(),
            ledger.clone(),
            dev.ip(),
            &[],
            false,
            NodeDataDir::new_test(None),
            dev.port(),
        )
        .unwrap();
        // Insert certificate to the storage.
        for certificate in certificates.iter() {
            storage.testing_only_insert_certificate_testing_only(certificate.clone());
        }
        // Check that the current committee members are authorized validators.
        for i in 0..certificates.clone().len() {
            let is_authorized = gateway.is_authorized_validator_address(certificates[i].author());
            if i < committee_size {
                assert!(is_authorized);
            } else {
                assert!(!is_authorized);
            }
        }
    }
}
