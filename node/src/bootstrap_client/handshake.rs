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

use crate::{
    BootstrapClient,
    bft::events::{self, DisconnectReason, Event},
    bootstrap_client::{codec::BootstrapClientCodec, network::MessageOrEvent},
    network::{ConnectionMode, NodeType, PeerPoolHandling, log_repo_sha_comparison},
    router::messages::{self, Message},
    tcp::{ConnectError, Connection, ConnectionSide, protocols::*},
};
use snarkos_node_network::harden_socket;
use snarkvm::{
    ledger::narwhal::Data,
    prelude::{Address, Network, io_error},
};

use futures_util::sink::SinkExt;

use std::{io, net::SocketAddr};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

#[derive(Debug)]
enum HandshakeMessageKind {
    ChallengeRequest,
    ChallengeResponse,
}

macro_rules! send_msg {
    ($msg:expr, $framed:expr, $peer_addr:expr) => {{
        trace!("Sending '{}' to '{}'", $msg.name(), $peer_addr);
        $framed.send($msg).await
    }};
}

/// A macro handling incoming handshake messages, rejecting unexpected ones.
macro_rules! expect_handshake_msg {
    ($msg_ty:expr, $framed:expr, $peer_addr:expr) => {{
        // Read the message as bytes.
        let Some(message) = $framed.try_next().await? else {
            return Err(ConnectError::other(format!(
                "the peer disconnected before sending {:?}, likely due to peer saturation or shutdown",
                stringify!($msg_ty),
            )));
        };

        // Match the expected message type with its expected size or peer type indicator.
        match $msg_ty {
            HandshakeMessageKind::ChallengeRequest
                if matches!(
                    message,
                    MessageOrEvent::Message(Message::ChallengeRequest(_))
                        | MessageOrEvent::Event(Event::ChallengeRequest(_))
                ) =>
            {
                trace!("Received a '{}' from '{}'", stringify!($msg_ty), $peer_addr);
                message
            }
            HandshakeMessageKind::ChallengeResponse
                if matches!(
                    message,
                    MessageOrEvent::Message(Message::ChallengeResponse(_))
                        | MessageOrEvent::Event(Event::ChallengeResponse(_))
                ) =>
            {
                trace!("Received a '{}' from '{}'", stringify!($msg_ty), $peer_addr);
                message
            }
            _ => {
                let msg_name = match message {
                    MessageOrEvent::Message(message) => message.name(),
                    MessageOrEvent::Event(event) => event.name(),
                };
                return Err(ConnectError::other(format!(
                    "'{}' did not follow the handshake protocol: expected {}, got {msg_name}",
                    $peer_addr,
                    stringify!($msg_ty),
                )));
            }
        }
    }};
}

#[async_trait]
impl<N: Network> Handshake for BootstrapClient<N> {
    async fn perform_handshake(&self, mut connection: Connection) -> Result<Connection, ConnectError> {
        let peer_addr = connection.addr();
        let peer_side = connection.side();
        let stream = self.borrow_stream(&mut connection);
        // Make the socket more robust.
        harden_socket(stream)?;

        // We don't know the listening address yet, as we don't initiate connections.
        let mut listener_addr = if peer_side == ConnectionSide::Initiator {
            debug!("Received a connection request from '{peer_addr}'");
            None
        } else {
            unreachable!("The boostrapper clients don't initiate connections");
        };

        // Perform the handshake; we pass on a mutable reference to listener_addr in case the process is broken at any point in time.
        let handshake_result = if peer_side == ConnectionSide::Responder {
            unreachable!("The boostrapper clients don't initiate connections");
        } else {
            self.handshake_inner_responder(peer_addr, &mut listener_addr, stream).await
        };

        if let Some(addr) = listener_addr {
            match handshake_result {
                Ok((peer_port, peer_aleo_addr, peer_node_type, peer_version, peer_snarkos_sha, connection_mode)) => {
                    if let Some(peer) = self.peer_pool.write().get_mut(&addr) {
                        // Due to only having a single Resolver, the BootstrapClient only adds an Aleo
                        // address mapping for Gateway-mode connections, as it is only used there, and
                        // it could otherwise clash with the Router-mode mapping for validators, which
                        // may connect in both modes at the same time.
                        let aleo_addr =
                            if connection_mode == ConnectionMode::Gateway { Some(peer_aleo_addr) } else { None };
                        self.resolver.write().insert_peer(peer.listener_addr(), peer_addr, aleo_addr);
                        peer.upgrade_to_connected(
                            peer_addr,
                            peer_port,
                            peer_aleo_addr,
                            peer_node_type,
                            peer_version,
                            peer_snarkos_sha,
                            connection_mode,
                        );
                    }
                    debug!("Completed the handshake with '{peer_addr}'");
                }
                Err(_) => {
                    if let Some(peer) = self.peer_pool.write().get_mut(&addr) {
                        // The peer may only be downgraded if it's a ConnectingPeer.
                        if peer.is_connecting() {
                            peer.downgrade_to_candidate(addr);
                        }
                    }
                }
            }
        }

        handshake_result.map(|_| connection)
    }
}

impl<N: Network> BootstrapClient<N> {
    /// The connection responder side of the handshake.
    async fn handshake_inner_responder<'a>(
        &'a self,
        peer_addr: SocketAddr,
        listener_addr: &mut Option<SocketAddr>,
        stream: &'a mut TcpStream,
    ) -> Result<(u16, Address<N>, NodeType, u32, Option<[u8; 40]>, ConnectionMode), ConnectError> {
        // Construct the stream.
        let mut framed = Framed::new(stream, BootstrapClientCodec::<N>::handshake());

        /* Step 1: Receive the challenge request. */

        // Listen for the challenge request message, which can be either from a regular peer, or a validator.
        let peer_request = expect_handshake_msg!(HandshakeMessageKind::ChallengeRequest, framed, peer_addr);
        let (peer_port, peer_nonce, peer_aleo_addr, peer_node_type, peer_version, peer_snarkos_sha, connection_mode) =
            match peer_request {
                MessageOrEvent::Message(Message::ChallengeRequest(ref msg)) => (
                    msg.listener_port,
                    msg.nonce,
                    msg.address,
                    msg.node_type,
                    msg.version,
                    msg.snarkos_sha,
                    ConnectionMode::Router,
                ),
                MessageOrEvent::Event(Event::ChallengeRequest(ref msg)) => (
                    msg.listener_port,
                    msg.nonce,
                    msg.address,
                    NodeType::Validator,
                    msg.version,
                    msg.snarkos_sha,
                    ConnectionMode::Gateway,
                ),
                _ => unreachable!(),
            };
        debug!("Handshake mode: {connection_mode:?}");

        // Obtain the peer's listening address.
        *listener_addr = Some(SocketAddr::new(peer_addr.ip(), peer_port));

        // Introduce the peer into the peer pool.
        self.add_connecting_peer(listener_addr.unwrap())?;

        // Verify the challenge request.
        if !self.verify_challenge_request(peer_addr, &mut framed, &peer_request).await? {
            return Err(ConnectError::application(DisconnectReason::InvalidChallengeRequest));
        };

        /* Step 2: Send the challenge response followed by own challenge request. */

        // Sign the counterparty nonce.
        let response_nonce: u64 = rand::random();
        let data = [peer_nonce.to_le_bytes(), response_nonce.to_le_bytes()].concat();
        let Ok(our_signature) = self.account.sign_bytes(&data, &mut rand::rng()) else {
            return Err(ConnectError::other(format!("Failed to sign the challenge request nonce from '{peer_addr}'")));
        };

        // Send the challenge response.
        if connection_mode == ConnectionMode::Router {
            let our_response = messages::ChallengeResponse {
                genesis_header: self.genesis_header,
                restrictions_id: self.restrictions_id,
                signature: Data::Object(our_signature),
                nonce: response_nonce,
            };
            let msg = Message::ChallengeResponse::<N>(our_response);
            send_msg!(msg, framed, peer_addr)?;
        } else {
            let our_response = events::ChallengeResponse {
                restrictions_id: self.restrictions_id,
                signature: Data::Object(our_signature),
                nonce: response_nonce,
            };
            let msg = Event::ChallengeResponse::<N>(our_response);
            send_msg!(msg, framed, peer_addr)?;
        }

        // Sample a random nonce.
        let our_nonce: u64 = rand::random();
        // Do not send a snarkOS SHA as the bootstrap client is not aware of height.
        let snarkos_sha = None;
        // Send the challenge request.
        if connection_mode == ConnectionMode::Router {
            let our_request = messages::ChallengeRequest::new(
                self.local_ip().port(),
                NodeType::BootstrapClient,
                self.account.address(),
                our_nonce,
                snarkos_sha,
            );
            let msg = Message::ChallengeRequest(our_request);
            send_msg!(msg, framed, peer_addr)?;
        } else {
            let our_request =
                events::ChallengeRequest::new(self.local_ip().port(), self.account.address(), our_nonce, snarkos_sha);
            let msg = Event::ChallengeRequest(our_request);
            send_msg!(msg, framed, peer_addr)?;
        }

        /* Step 3: Receive the challenge response. */

        // Listen for the challenge response message.
        let peer_response = expect_handshake_msg!(HandshakeMessageKind::ChallengeResponse, framed, peer_addr);
        // Verify the challenge response.
        if !self.verify_challenge_response(peer_addr, peer_aleo_addr, our_nonce, &peer_response).await {
            if connection_mode == ConnectionMode::Router {
                let msg = Message::Disconnect::<N>(messages::DisconnectReason::InvalidChallengeResponse.into());
                send_msg!(msg, framed, peer_addr)?;
            } else {
                let msg = Event::Disconnect::<N>(events::DisconnectReason::InvalidChallengeResponse.into());
                send_msg!(msg, framed, peer_addr)?;
            }
            return Err(ConnectError::application(DisconnectReason::InvalidChallengeResponse));
        }

        Ok((peer_port, peer_aleo_addr, peer_node_type, peer_version, peer_snarkos_sha, connection_mode))
    }

    async fn verify_challenge_request(
        &self,
        peer_addr: SocketAddr,
        framed: &mut Framed<&mut TcpStream, BootstrapClientCodec<N>>,
        request: &MessageOrEvent<N>,
    ) -> io::Result<bool> {
        match request {
            MessageOrEvent::Message(Message::ChallengeRequest(msg)) => {
                log_repo_sha_comparison(peer_addr, &msg.snarkos_sha, Self::OWNER);

                if msg.version < Message::<N>::latest_message_version() {
                    let msg = Message::Disconnect::<N>(messages::DisconnectReason::OutdatedClientVersion.into());
                    send_msg!(msg, framed, peer_addr)?;
                    return Ok(false);
                }

                // Reject validators that aren't members of the committee.
                if msg.node_type == NodeType::Validator {
                    if let Some(current_committee) =
                        self.get_or_update_committee().await.map_err(|_| io_error("Couldn't load the committee"))?
                    {
                        if !current_committee.contains(&msg.address) {
                            let msg = Message::Disconnect::<N>(messages::DisconnectReason::ProtocolViolation.into());
                            send_msg!(msg, framed, peer_addr)?;
                            return Ok(false);
                        }
                    }
                }
            }
            MessageOrEvent::Event(Event::ChallengeRequest(msg)) => {
                log_repo_sha_comparison(peer_addr, &msg.snarkos_sha, Self::OWNER);

                if msg.version < Event::<N>::VERSION {
                    let msg = Event::Disconnect::<N>(events::DisconnectReason::OutdatedClientVersion.into());
                    send_msg!(msg, framed, peer_addr)?;
                    return Ok(false);
                }

                // Reject validators that aren't members of the committee.
                if let Some(current_committee) =
                    self.get_or_update_committee().await.map_err(|_| io_error("Couldn't load the committee"))?
                {
                    if !current_committee.contains(&msg.address) {
                        let msg = Message::Disconnect::<N>(messages::DisconnectReason::ProtocolViolation.into());
                        send_msg!(msg, framed, peer_addr)?;
                        return Ok(false);
                    }
                }
            }
            _ => unreachable!(),
        }

        Ok(true)
    }

    async fn verify_challenge_response(
        &self,
        peer_addr: SocketAddr,
        peer_aleo_addr: Address<N>,
        our_nonce: u64,
        response: &MessageOrEvent<N>,
    ) -> bool {
        let (peer_restrictions_id, peer_signature, peer_nonce) = match response {
            MessageOrEvent::Message(Message::ChallengeResponse(msg)) => {
                (msg.restrictions_id, msg.signature.clone(), msg.nonce)
            }
            MessageOrEvent::Event(Event::ChallengeResponse(msg)) => {
                (msg.restrictions_id, msg.signature.clone(), msg.nonce)
            }
            _ => unreachable!(),
        };

        // Verify the restrictions ID.
        if peer_restrictions_id != self.restrictions_id {
            warn!("{} Handshake with '{peer_addr}' failed (incorrect restrictions ID)", Self::OWNER);
            return false;
        }
        // Perform the deferred non-blocking deserialization of the signature.
        let Ok(signature) = peer_signature.deserialize().await else {
            warn!("{} Handshake with '{peer_addr}' failed (cannot deserialize the signature)", Self::OWNER);
            return false;
        };
        // Verify the signature.
        if !signature.verify_bytes(&peer_aleo_addr, &[our_nonce.to_le_bytes(), peer_nonce.to_le_bytes()].concat()) {
            warn!("{} Handshake with '{peer_addr}' failed (invalid signature)", Self::OWNER);
            return false;
        }

        true
    }
}
