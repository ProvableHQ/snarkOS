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

use crate::{
    BootstrapClient,
    bft::{
        MAX_VALIDATORS_TO_SEND,
        events::{self, Event},
    },
    bootstrap_client::codec::BootstrapClientCodec,
    router::{
        MAX_PEERS_TO_SEND,
        Peer,
        PeerPoolHandling,
        Resolver,
        messages::{self, Message},
    },
    tcp::{ConnectionSide, P2P, Tcp, protocols::*},
};
use snarkvm::prelude::Network;

use indexmap::IndexMap;
#[cfg(feature = "locktick")]
use locktick::parking_lot::RwLock;
#[cfg(not(feature = "locktick"))]
use parking_lot::RwLock;
use std::{collections::HashMap, io, net::SocketAddr};
use tokio::time::sleep;
use tokio_util::codec::Decoder;

impl<N: Network> P2P for BootstrapClient<N> {
    fn tcp(&self) -> &Tcp {
        &self.tcp
    }
}

impl<N: Network> PeerPoolHandling<N> for BootstrapClient<N> {
    const MAXIMUM_POOL_SIZE: usize = 10_000;
    const OWNER: &'static str = "[Network]";
    const PEER_SLASHING_COUNT: usize = 200;

    fn is_dev(&self) -> bool {
        self.dev.is_some()
    }

    fn peer_pool(&self) -> &RwLock<HashMap<SocketAddr, Peer<N>>> {
        &self.peer_pool
    }

    fn resolver(&self) -> &RwLock<Resolver<N>> {
        &self.resolver
    }
}

/// The bootstrap client can handle both validator and non-validator messages.
#[derive(Debug)]
pub enum MessageOrEvent<N: Network> {
    Message(Message<N>),
    Event(Event<N>),
}

#[async_trait]
impl<N: Network> OnConnect for BootstrapClient<N> {
    async fn on_connect(&self, peer_addr: SocketAddr) {
        // If the peer is connected in validator (Gateway) mode, save it to the collection
        // of known validators.
        if let Some(listener_addr) = self.resolve_to_listener(peer_addr) {
            if let Some(peer) = self.get_connected_peer(listener_addr) {
                if self.resolver().read().get_peer_ip_for_address(peer.aleo_addr).is_some() {
                    self.known_validators.write().insert(listener_addr, peer.aleo_addr);
                }
            }
        }
        // The peers should only ask us for the peer list; spawn a task that will
        // terminate the connection after a while.
        let tcp = self.tcp().clone();
        tokio::spawn(async move {
            sleep(Self::CONNECTION_LIFETIME).await;
            tcp.disconnect(peer_addr).await;
        });
    }
}

#[async_trait]
impl<N: Network> Disconnect for BootstrapClient<N> {
    /// Any extra operations to be performed during a disconnect.
    async fn handle_disconnect(&self, peer_addr: SocketAddr) {
        if let Some(listener_addr) = self.resolve_to_listener(peer_addr) {
            self.downgrade_peer_to_candidate(listener_addr);
        }
    }
}

#[async_trait]
impl<N: Network> Reading for BootstrapClient<N> {
    type Codec = BootstrapClientCodec<N>;
    type Message = <BootstrapClientCodec<N> as Decoder>::Item;

    /// Creates a [`Decoder`] used to interpret messages from the network.
    /// The `side` param indicates the connection side **from the node's perspective**.
    fn codec(&self, _peer_addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Default::default()
    }

    /// Processes a message received from the network.
    async fn process_message(&self, peer_addr: SocketAddr, message: Self::Message) -> io::Result<()> {
        // Identify the connected peer.
        let Some(listener_addr) = self.resolve_to_listener(peer_addr) else {
            // Already disconnecting, ignore.
            return Ok(());
        };

        // Handle the right peer request.
        match message {
            MessageOrEvent::Message(Message::PeerRequest(_)) => {
                debug!("Received a PeerRequest from '{listener_addr}'");
                let mut peers = self.get_candidate_peers();

                // Filter out peers who had been connected via the Gateway.
                let known_validators = self.get_known_validators();
                peers.retain(|peer| !known_validators.contains_key(&peer.listener_addr));
                peers.truncate(MAX_PEERS_TO_SEND);
                let peers = peers.into_iter().map(|peer| (peer.listener_addr, None)).collect::<Vec<_>>();

                debug!("Sending {} peer address(es) to '{listener_addr}'", peers.len());
                let msg = MessageOrEvent::Message(Message::PeerResponse(messages::PeerResponse { peers }));
                let _ = self.unicast(peer_addr, msg)?.await;

                debug!("Disconnecting from '{listener_addr}' - peers provided");
                self.tcp().disconnect(peer_addr).await;
            }
            MessageOrEvent::Event(Event::ValidatorsRequest(_)) => {
                debug!("Received a ValidatorsRequest from '{listener_addr}'");
                let current_committee = match self.get_or_update_committee().await {
                    Ok(new_committee) => new_committee,
                    Err(error) => {
                        error!("Couldn't update the validator committee: {error}");
                        None
                    }
                };

                // Return the known addresses of current committee members, or all known
                // validators if the committee info is unavailable.
                let mut known_validators = self.get_known_validators();
                if let Some(committee) = &current_committee {
                    known_validators.retain(|_, aleo_addr| committee.contains(aleo_addr));
                }
                let validators = known_validators.into_iter().take(MAX_VALIDATORS_TO_SEND).collect::<IndexMap<_, _>>();

                debug!("Sending {} validator address(es) to '{listener_addr}'", validators.len());
                let msg = MessageOrEvent::Event(Event::ValidatorsResponse(events::ValidatorsResponse { validators }));
                let _ = self.unicast(peer_addr, msg)?.await;

                debug!("Disconnecting from '{listener_addr}' - peers provided");
                self.tcp().disconnect(peer_addr).await;
            }
            msg => {
                let name = match msg {
                    MessageOrEvent::Message(msg) => msg.name(),
                    MessageOrEvent::Event(msg) => msg.name(),
                };
                trace!("Ignoring an unhandled message ({name}) from {listener_addr}");
            }
        }

        Ok(())
    }
}

#[async_trait]
impl<N: Network> Writing for BootstrapClient<N> {
    type Codec = BootstrapClientCodec<N>;
    type Message = MessageOrEvent<N>;

    /// Creates an [`Encoder`] used to write the outbound messages to the target stream.
    /// The `side` parameter indicates the connection side **from the node's perspective**.
    fn codec(&self, _addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Default::default()
    }
}
