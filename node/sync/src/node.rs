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

use crate::{SyncCodec, SyncSender, communication_service::CommunicationService};

use snarkos_node_bft_ledger_service::LedgerService;
use snarkos_node_network::{BlockRequest, BlockResponse, DataBlocks, SyncToken, harden_socket};
use snarkos_node_tcp::{self as tcp, ConnectError, Connection, ConnectionSide, P2P, Tcp, protocols::*};
use snarkvm::prelude::Network;

use async_trait::async_trait;
#[cfg(feature = "locktick")]
use locktick::parking_lot::Mutex;
#[cfg(not(feature = "locktick"))]
use parking_lot::Mutex;
use std::{collections::HashMap, io, marker::PhantomData, net::SocketAddr, sync::Arc, time::Duration};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    sync::{OnceCell, oneshot},
    task,
    time::timeout,
};

/// The amount of time a sync stream request token is active.
pub const SYNC_STREAM_TOKEN_LIFETIME: Duration = Duration::from_secs(5);

/// The maximum number of concurrently syncing peers.
pub const MAX_CONCURRENT_STREAMS: u16 = 5;

/// The maximum number of blocks deliverable in a single stream/session.
pub const MAX_NUM_BLOCKS_PER_REQUEST: u32 = 100;

/// The maximum number of blocks per a single response message.
pub const MAX_NUM_BLOCKS_PER_RESPONSE: u32 = 5;

/// The handler for sync streams, both inbound and outbound.
#[derive(Clone)]
pub struct SyncStreams<N: Network> {
    /// The engine for sync streams.
    tcp: Tcp,
    /// The ledger.
    ledger: Arc<dyn LedgerService<N>>,
    /// Access tokens given to peers to sync with us, and the associated requests.
    tokens_for_peers: Arc<Mutex<HashMap<SyncToken, BlockRequest>>>,
    /// Access tokens received from peers we can sync with.
    tokens_from_peers: Arc<Mutex<HashMap<SocketAddr, SyncToken>>>,
    /// Holds the currently active sync streams.
    active_streams: Arc<Mutex<HashMap<SocketAddr, BlockRequest>>>,
    /// The listener address peers access in order to sync.
    listener_addr: OnceCell<SocketAddr>,
    /// The conduit to SyncBlocks.
    sync_sender: OnceCell<SyncSender<N>>,
    _phantom: PhantomData<N>,
}

impl<N: Network> SyncStreams<N> {
    pub fn new(listener_addr: SocketAddr, ledger: Arc<dyn LedgerService<N>>) -> Self {
        let tcp_config = tcp::Config::new(listener_addr, MAX_CONCURRENT_STREAMS);
        let tcp = Tcp::new(tcp_config);

        Self {
            tcp,
            ledger,
            tokens_for_peers: Default::default(),
            tokens_from_peers: Default::default(),
            active_streams: Default::default(),
            listener_addr: Default::default(),
            sync_sender: Default::default(),
            _phantom: Default::default(),
        }
    }

    /// Start the underlying TCP stack.
    pub async fn enable(&self) {
        self.enable_handshake().await;
        self.enable_reading().await;
        self.enable_writing().await;
        self.enable_on_connect().await;
        self.enable_disconnect().await;
        let listener_addr = self.tcp.enable_listener().await.expect("Failed to enable the TCP listener");

        debug!("[SyncStreams] Listening for sync requests at {listener_addr}");

        self.listener_addr.set(listener_addr).expect("Attempted to enable SyncStreams more than once");
    }

    /// Attaches the channel for communication with BlockSync.
    pub fn set_sync_sender(&self, sync_sender: SyncSender<N>) {
        self.sync_sender.set(sync_sender).expect("Sync sender already set in SyncStreams");
    }

    /// Returns the listener address that creates the sync streams for peers.
    pub fn listener_addr(&self) -> SocketAddr {
        *self.listener_addr.get().expect("SyncStreams::enable hadn't been called") // guaranteed present for all calls
    }

    /// Activate an access token for sync requests from peers.
    pub fn register_token_for_peer(&self, token: SyncToken, request: BlockRequest) {
        self.tokens_for_peers.lock().insert(token, request);
    }

    /// Deactivate an access token given to a peer.
    pub fn remove_token_for_peer(&self, token: SyncToken) -> Option<BlockRequest> {
        self.tokens_for_peers.lock().remove(&token)
    }

    /// Save a token from a peer we requested a sync from.
    pub fn register_token_from_peer(&self, addr: SocketAddr, token: SyncToken) {
        self.tokens_from_peers.lock().insert(addr, token);
    }

    /// Delete an access token received from a peer.
    pub fn remove_token_from_peer(&self, addr: SocketAddr) -> Option<SyncToken> {
        self.tokens_from_peers.lock().remove(&addr)
    }

    /// Used post-handshake to associate a peer address with their sync request.
    pub fn register_active_stream(&self, addr: SocketAddr, request: BlockRequest) {
        self.active_streams.lock().insert(addr, request);
    }

    /// Get the block request associated with the given peer.
    pub fn get_peer_block_request(&self, addr: SocketAddr) -> Option<BlockRequest> {
        self.active_streams.lock().get(&addr).copied()
    }

    /// Perform a clean teardown.
    pub async fn shut_down(&self) {
        debug!("[SyncStreams] Shutting down");
        self.active_streams.lock().clear();
        self.tokens_for_peers.lock().clear();
        self.tokens_from_peers.lock().clear();
        self.tcp.shut_down().await;
    }
}

impl<N: Network> P2P for SyncStreams<N> {
    fn tcp(&self) -> &Tcp {
        &self.tcp
    }
}

#[async_trait]
impl<N: Network> Handshake for SyncStreams<N> {
    /// A simple, "one-sided" protocol where the initiator is expected to provide an access token,
    /// and the responder to validate it against its list of active access tokens.
    async fn perform_handshake(&self, mut connection: Connection) -> Result<Connection, ConnectError> {
        let peer_addr = connection.addr();
        let peer_side = connection.side();
        let stream = self.borrow_stream(&mut connection);

        // Make the socket more robust.
        harden_socket(stream)?;

        if peer_side == ConnectionSide::Initiator {
            debug!("[SyncStreams] Shaking hands with {peer_addr} as the responder");
            // We've received a sync request from a peer; expect an access token.
            let mut token = [0u8; 32];
            let token = match timeout(Duration::from_secs(5), stream.read_exact(&mut token)).await {
                Ok(Ok(32)) => Ok(token),
                Ok(Ok(_)) | Ok(Err(_)) => Err(ConnectError::IoError(io::ErrorKind::InvalidData.into())),
                Err(_) => Err(ConnectError::IoError(io::ErrorKind::TimedOut.into())),
            }?;

            // Check if the access token is active.
            let Some(request) = self.remove_token_for_peer(token.into()) else {
                return Err(ConnectError::IoError(io::ErrorKind::InvalidData.into()));
            };

            // Mark the stream as active and assign the associated request to it.
            self.register_active_stream(peer_addr, request);

            // All good, handshake complete.
        } else {
            debug!("[SyncStreams] Shaking hands with {peer_addr} as the initiator");
            // We're the ones who requested the sync; find the access token the peer provided us with.
            let Some(token) = self.remove_token_from_peer(peer_addr) else {
                return Err(ConnectError::IoError(io::ErrorKind::NotFound.into()));
            };

            // Send the access token.
            stream.write_all(&token).await?;

            // Done, the rest is up to the peer.
        }

        debug!("[SyncStreams] Successfully shaken hands with {peer_addr}");
        Ok(connection)
    }
}

#[async_trait]
impl<N: Network> Reading for SyncStreams<N> {
    type Codec = SyncCodec<N>;
    type Message = BlockResponse<N>;

    fn codec(&self, _peer_addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Default::default()
    }

    async fn process_message(&self, peer_addr: SocketAddr, message: Self::Message) -> io::Result<()> {
        debug!("[SyncStreams] Got a block response from {peer_addr}");
        let BlockResponse { request, latest_consensus_version, blocks, .. } = message;

        let Some(sync_sender) = self.sync_sender.get() else {
            return Err(io::ErrorKind::BrokenPipe.into());
        };

        // Perform the deferred non-blocking deserialization of the blocks.
        // The deserialization can take a long time (minutes). We should not be running
        // this on a blocking task, but on a rayon thread pool.
        let (send, recv) = oneshot::channel();
        rayon::spawn_fifo(move || {
            let blocks = blocks.deserialize_blocking();
            let _ = send.send(blocks);
        });
        let blocks = match recv.await {
            Ok(Ok(blocks)) => blocks,
            Ok(Err(error)) => {
                warn!("[SyncStreams] Peer '{peer_addr}' sent an invalid block response - {error}");
                return Err(io::ErrorKind::InvalidData.into());
            }
            Err(error) => {
                warn!("[SyncStreams] Peer '{peer_addr}' sent an invalid block response - {error}");
                return Err(io::ErrorKind::InvalidData.into());
            }
        };

        // Ensure the block response is well-formed.
        if let Err(err) = blocks.ensure_response_is_well_formed(peer_addr, request.start_height, request.end_height) {
            warn!("[SyncStreams] {err}");
            return Err(io::ErrorKind::InvalidData.into());
        }
        // Send the blocks to the sync module.
        match sync_sender.insert_block_response(peer_addr, blocks.0, latest_consensus_version).await {
            Ok(_) => Ok(()),
            Err(err) if err.is_benign() => {
                debug!("[SyncStreams] Ignoring block response from peer '{peer_addr}'");
                Ok(())
            }
            Err(err) if err.is_invalid_consensus_version() => {
                error!("[SyncStreams] Peer sent an invalid block response '{peer_addr}': {err}");
                Err(io::ErrorKind::InvalidData.into())
            }
            Err(err) => {
                warn!("[SyncStreams] Peer '{peer_addr}' sent an invalid block response: {err}");

                // TODO: disconnect instead?

                Ok(())
            }
        }
    }

    fn message_queue_depth(&self) -> usize {
        100
    }
}

#[async_trait]
impl<N: Network> Writing for SyncStreams<N> {
    type Codec = SyncCodec<N>;
    type Message = BlockResponse<N>;

    fn codec(&self, _peer_addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Default::default()
    }

    fn message_queue_depth(&self) -> usize {
        100
    }
}

#[async_trait]
impl<N: Network> Disconnect for SyncStreams<N> {
    async fn handle_disconnect(&self, peer_addr: SocketAddr) {
        self.active_streams.lock().remove(&peer_addr);
    }
}

#[async_trait]
impl<N: Network> OnConnect for SyncStreams<N> {
    async fn on_connect(&self, peer_addr: SocketAddr) {
        // Check if we're the ones who provide the sync.
        let Some(request) = self.get_peer_block_request(peer_addr) else {
            // If not, exit early.
            return;
        };

        let BlockRequest { start_height, end_height } = request;

        if start_height >= end_height {
            warn!("Block request from '{peer_addr}' has an invalid range ({start_height}..{end_height})");
            return;
        }
        if end_height - start_height > MAX_NUM_BLOCKS_PER_REQUEST {
            warn!("Block request from '{peer_addr}' has an excessive range ({start_height}..{end_height})");
            return;
        }

        let mut start = start_height;

        while start < end_height {
            let end = end_height.min(start + MAX_NUM_BLOCKS_PER_RESPONSE);
            let self_ = self.clone();
            let blocks = match task::spawn_blocking(move || match self_.ledger.get_blocks(start..end) {
                Ok(blocks) => Ok::<DataBlocks<N>, io::Error>(DataBlocks(blocks)),
                Err(error) => {
                    warn!("Missing blocks {start} to {end} from ledger - {error}");
                    Err(io::ErrorKind::NotFound.into())
                }
            })
            .await
            {
                Ok(Ok(blocks)) => blocks,
                Ok(Err(error)) => {
                    warn!("[SyncStreams] Error: {error}");
                    return;
                }
                Err(error) => {
                    warn!("[SyncStreams] Error: {error}");
                    return;
                }
            };

            debug!("[SyncStreams] Sending block response ({start}..{end}) to {peer_addr}");
            let Ok(latest_consensus_version) = N::CONSENSUS_VERSION(end - 1) else {
                return;
            };
            let response = BlockResponse::new(request, blocks, latest_consensus_version);
            match self.unicast(peer_addr, response) {
                Ok(rx) => {
                    if let Err(error) = rx.await {
                        warn!("[SyncStreams] Error: {error}");
                        return;
                    }
                }
                Err(error) => {
                    warn!("[SyncStreams] Error: {error}");
                    return;
                }
            }

            start += MAX_NUM_BLOCKS_PER_RESPONSE;
        }
    }
}

#[async_trait]
impl<N: Network> CommunicationService for SyncStreams<N> {
    type Message = BlockResponse<N>;

    fn prepare_block_request(_start_height: u32, _end_height: u32) -> Self::Message {
        unimplemented!("Unused by SyncStreams");
    }

    async fn send(&self, peer_addr: SocketAddr, message: Self::Message) -> Option<oneshot::Receiver<io::Result<()>>> {
        let result = self.unicast(peer_addr, message);

        if let Err(err) = &result {
            warn!("[SyncStreams] Failed to send 'BlockResponse' to '{peer_addr}': {err:?}");
            debug!("[SyncStreams] Disconnecting from '{peer_addr}' (unable to send)");
            self.tcp.disconnect(peer_addr).await;
        }
        result.ok()
    }
}
