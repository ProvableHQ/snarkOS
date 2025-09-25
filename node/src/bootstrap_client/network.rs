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
    bft::events::{self, Event},
    bootstrap_client::codec::BootstrapClientCodec,
    router::{
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
                let peers = self.get_best_connected_peers(Some(u8::MAX as usize));
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
                        Default::default()
                    }
                };

                let validators = if !current_committee.is_empty() {
                    let peers = self.get_best_connected_peers(Some(u8::MAX as usize));
                    let mut validators = IndexMap::with_capacity(current_committee.len());
                    for validator in peers.into_iter().filter(|peer| current_committee.contains(&peer.aleo_addr)) {
                        validators.insert(validator.listener_addr, validator.aleo_addr);
                    }
                    validators
                } else {
                    Default::default()
                };

                debug!("Sending {} validator address(es) to '{listener_addr}'", validators.len());
                let msg = MessageOrEvent::Event(Event::ValidatorsResponse(events::ValidatorsResponse { validators }));
                let _ = self.unicast(peer_addr, msg)?.await;

                debug!("Disconnecting from '{listener_addr}' - peers provided");
                self.tcp().disconnect(peer_addr).await;
            }
            _ => {
                trace!("Ignoring an unhandled message from {listener_addr}");
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
