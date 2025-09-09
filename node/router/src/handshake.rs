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
    NodeType,
    Peer,
    Router,
    messages::{ChallengeRequest, ChallengeResponse, DisconnectReason, Message, MessageCodec, MessageTrait},
};
use snarkos_node_tcp::{ConnectionSide, P2P, Tcp};
use snarkvm::{
    ledger::narwhal::Data,
    prelude::{Address, Field, Network, block::Header, error},
};

use anyhow::{Result, bail};
use futures::SinkExt;
use rand::{Rng, rngs::OsRng};
use std::{collections::hash_map::Entry, io, net::SocketAddr};
use tokio::net::TcpStream;
use tokio_stream::StreamExt;
use tokio_util::codec::Framed;

impl<N: Network> P2P for Router<N> {
    /// Returns a reference to the TCP instance.
    fn tcp(&self) -> &Tcp {
        &self.tcp
    }
}

/// A macro unwrapping the expected handshake message or returning an error for unexpected messages.
#[macro_export]
macro_rules! expect_message {
    ($msg_ty:path, $framed:expr, $peer_addr:expr) => {
        match $framed.try_next().await? {
            // Received the expected message, proceed.
            Some($msg_ty(data)) => {
                trace!("Received '{}' from '{}'", data.name(), $peer_addr);
                data
            }
            // Received a disconnect message, abort.
            Some(Message::Disconnect(reason)) => {
                return Err(error(format!("'{}' disconnected: {reason:?}", $peer_addr)))
            }
            // Received an unexpected message, abort.
            Some(ty) => {
                return Err(error(format!(
                    "'{}' did not follow the handshake protocol: received {:?} instead of {}",
                    $peer_addr,
                    ty.name(),
                    stringify!($msg_ty),
                )))
            }
            // Received nothing.
            None => {
                return Err(error(format!("'{}' disconnected before sending {:?}", $peer_addr, stringify!($msg_ty),)))
            }
        }
    };
}

/// Send the given message to the peer.
async fn send<N: Network>(
    framed: &mut Framed<&mut TcpStream, MessageCodec<N>>,
    peer_addr: SocketAddr,
    message: Message<N>,
) -> io::Result<()> {
    trace!("Sending '{}' to '{peer_addr}'", message.name());
    framed.send(message).await
}

impl<N: Network> Router<N> {
    /// Executes the handshake protocol.
    pub async fn handshake<'a>(
        &'a self,
        peer_addr: SocketAddr,
        stream: &'a mut TcpStream,
        peer_side: ConnectionSide,
        genesis_header: Header<N>,
        restrictions_id: Field<N>,
    ) -> io::Result<ChallengeRequest<N>> {
        // If this is an inbound connection, we log it, but don't know the listening address yet.
        // Otherwise, we can immediately register the listening address.
        let mut listener_addr = if peer_side == ConnectionSide::Initiator {
            debug!("Received a connection request from '{peer_addr}'");
            None
        } else {
            debug!("Connecting to '{peer_addr}'...");
            Some(peer_addr)
        };

        // Check (or impose) IP-level bans.
        #[cfg(not(any(test)))]
        if !self.is_dev() && peer_side == ConnectionSide::Initiator {
            // If the IP is already banned reject the connection.
            if self.is_ip_banned(peer_addr.ip()) {
                trace!("Rejected a connection request from banned IP '{}'", peer_addr.ip());
                return Err(error(format!("'{}' is a banned IP address", peer_addr.ip())));
            }

            let num_attempts =
                self.cache.insert_inbound_connection(peer_addr.ip(), Router::<N>::CONNECTION_ATTEMPTS_SINCE_SECS);

            debug!("Number of connection attempts from '{}': {}", peer_addr.ip(), num_attempts);
            if num_attempts > Router::<N>::MAX_CONNECTION_ATTEMPTS {
                self.update_ip_ban(peer_addr.ip());
                trace!("Rejected a consecutive connection request from IP '{}'", peer_addr.ip());
                return Err(error(format!("'{}' appears to be spamming connections", peer_addr.ip())));
            }
        }

        // Perform the handshake; we pass on a mutable reference to listener_addr in case the process is broken at any point in time.
        let handshake_result = if peer_side == ConnectionSide::Responder {
            self.handshake_inner_initiator(peer_addr, stream, genesis_header, restrictions_id).await
        } else {
            self.handshake_inner_responder(peer_addr, &mut listener_addr, stream, genesis_header, restrictions_id).await
        };

        if let Some(addr) = listener_addr {
            if let Ok(ref challenge_request) = handshake_result {
                if let Some(peer) = self.peer_pool.write().get_mut(&addr) {
                    peer.upgrade_to_connected(peer_addr, challenge_request, self.clone());
                }
                #[cfg(feature = "metrics")]
                self.update_metrics();
                debug!("Completed the handshake with '{peer_addr}'");
            } else if let Some(peer) = self.peer_pool.write().get_mut(&addr) {
                peer.downgrade_to_candidate(addr);
            }
        }

        handshake_result
    }

    /// The connection initiator side of the handshake.
    async fn handshake_inner_initiator<'a>(
        &'a self,
        peer_addr: SocketAddr,
        stream: &'a mut TcpStream,
        genesis_header: Header<N>,
        restrictions_id: Field<N>,
    ) -> io::Result<ChallengeRequest<N>> {
        match self.peer_pool.write().entry(peer_addr) {
            Entry::Vacant(entry) => {
                entry.insert(Peer::new_connecting(false, peer_addr));
            }
            Entry::Occupied(mut entry) if matches!(entry.get(), Peer::Candidate(_)) => {
                entry.insert(Peer::new_connecting(entry.get().is_trusted(), peer_addr));
            }
            Entry::Occupied(_) => {
                return Err(error(format!("Duplicate connection attempt with '{peer_addr}'")));
            }
        }
        // Construct the stream.
        let mut framed = Framed::new(stream, MessageCodec::<N>::handshake());

        // Initialize an RNG.
        let rng = &mut OsRng;

        /* Step 1: Send the challenge request. */

        // Sample a random nonce.
        let our_nonce = rng.r#gen();
        // Send a challenge request to the peer.
        let our_request = ChallengeRequest::new(self.local_ip().port(), self.node_type, self.address(), our_nonce);
        send(&mut framed, peer_addr, Message::ChallengeRequest(our_request)).await?;

        /* Step 2: Receive the peer's challenge response followed by the challenge request. */

        // Listen for the challenge response message.
        let peer_response = expect_message!(Message::ChallengeResponse, framed, peer_addr);
        // Listen for the challenge request message.
        let peer_request = expect_message!(Message::ChallengeRequest, framed, peer_addr);

        // Verify the challenge response. If a disconnect reason was returned, send the disconnect message and abort.
        if let Some(reason) = self
            .verify_challenge_response(
                peer_addr,
                peer_request.address,
                peer_request.node_type,
                peer_response,
                genesis_header,
                restrictions_id,
                our_nonce,
            )
            .await
        {
            send(&mut framed, peer_addr, reason.into()).await?;
            return Err(error(format!("Dropped '{peer_addr}' for reason: {reason:?}")));
        }
        // Verify the challenge request. If a disconnect reason was returned, send the disconnect message and abort.
        if let Some(reason) = self.verify_challenge_request(peer_addr, &peer_request) {
            send(&mut framed, peer_addr, reason.into()).await?;
            return Err(error(format!("Dropped '{peer_addr}' for reason: {reason:?}")));
        }

        /* Step 3: Send the challenge response. */

        let response_nonce: u64 = rng.r#gen();
        let data = [peer_request.nonce.to_le_bytes(), response_nonce.to_le_bytes()].concat();
        // Sign the counterparty nonce.
        let Ok(our_signature) = self.account.sign_bytes(&data, rng) else {
            return Err(error(format!("Failed to sign the challenge request nonce from '{peer_addr}'")));
        };
        // Send the challenge response.
        let our_response = ChallengeResponse {
            genesis_header,
            restrictions_id,
            signature: Data::Object(our_signature),
            nonce: response_nonce,
        };
        send(&mut framed, peer_addr, Message::ChallengeResponse(our_response)).await?;

        Ok(peer_request)
    }

    /// The connection responder side of the handshake.
    async fn handshake_inner_responder<'a>(
        &'a self,
        peer_addr: SocketAddr,
        listener_addr: &mut Option<SocketAddr>,
        stream: &'a mut TcpStream,
        genesis_header: Header<N>,
        restrictions_id: Field<N>,
    ) -> io::Result<ChallengeRequest<N>> {
        // Construct the stream.
        let mut framed = Framed::new(stream, MessageCodec::<N>::handshake());

        /* Step 1: Receive the challenge request. */

        // Listen for the challenge request message.
        let peer_request = expect_message!(Message::ChallengeRequest, framed, peer_addr);

        // Obtain the peer's listening address.
        *listener_addr = Some(SocketAddr::new(peer_addr.ip(), peer_request.listener_port));
        let listener_addr = listener_addr.unwrap();

        // Knowing the peer's listening address, ensure it is allowed to connect.
        if let Err(forbidden_message) = self.ensure_peer_is_allowed(listener_addr) {
            return Err(error(format!("{forbidden_message}")));
        }
        // Verify the challenge request. If a disconnect reason was returned, send the disconnect message and abort.
        if let Some(reason) = self.verify_challenge_request(peer_addr, &peer_request) {
            send(&mut framed, peer_addr, reason.into()).await?;
            return Err(error(format!("Dropped '{peer_addr}' for reason: {reason:?}")));
        }

        /* Step 2: Send the challenge response followed by own challenge request. */

        // Initialize an RNG.
        let rng = &mut OsRng;

        // Sign the counterparty nonce.
        let response_nonce: u64 = rng.r#gen();
        let data = [peer_request.nonce.to_le_bytes(), response_nonce.to_le_bytes()].concat();
        let Ok(our_signature) = self.account.sign_bytes(&data, rng) else {
            return Err(error(format!("Failed to sign the challenge request nonce from '{peer_addr}'")));
        };
        // Send the challenge response.
        let our_response = ChallengeResponse {
            genesis_header,
            restrictions_id,
            signature: Data::Object(our_signature),
            nonce: response_nonce,
        };
        send(&mut framed, peer_addr, Message::ChallengeResponse(our_response)).await?;

        // Sample a random nonce.
        let our_nonce = rng.r#gen();
        // Send the challenge request.
        let our_request = ChallengeRequest::new(self.local_ip().port(), self.node_type, self.address(), our_nonce);
        send(&mut framed, peer_addr, Message::ChallengeRequest(our_request)).await?;

        /* Step 3: Receive the challenge response. */

        // Listen for the challenge response message.
        let peer_response = expect_message!(Message::ChallengeResponse, framed, peer_addr);
        // Verify the challenge response. If a disconnect reason was returned, send the disconnect message and abort.
        if let Some(reason) = self
            .verify_challenge_response(
                peer_addr,
                peer_request.address,
                peer_request.node_type,
                peer_response,
                genesis_header,
                restrictions_id,
                our_nonce,
            )
            .await
        {
            send(&mut framed, peer_addr, reason.into()).await?;
            return Err(error(format!("Dropped '{peer_addr}' for reason: {reason:?}")));
        }

        Ok(peer_request)
    }

    /// Ensure the peer is allowed to connect.
    fn ensure_peer_is_allowed(&self, listener_addr: SocketAddr) -> Result<()> {
        // Ensure that it's not a self-connect attempt.
        if self.is_local_ip(&listener_addr) {
            bail!("Dropping connection request from '{listener_addr}' (attempted to self-connect)")
        }
        // Unknown peers are untrusted, so check if `allow_external_peers` is true.
        if !self.allow_external_peers() && !self.is_trusted(&listener_addr) {
            bail!("Dropping connection request from '{listener_addr}' (untrusted)")
        }

        match self.peer_pool.write().entry(listener_addr) {
            Entry::Vacant(entry) => {
                entry.insert(Peer::new_connecting(false, listener_addr));
            }
            Entry::Occupied(mut entry) => match entry.get_mut() {
                peer @ Peer::Candidate(_) => {
                    *peer = Peer::new_connecting(peer.is_trusted(), listener_addr);
                }
                Peer::Connecting(_) => {
                    bail!("Dropping connection request from '{listener_addr}' (already connecting)");
                }
                Peer::Connected(_) => {
                    bail!("Dropping connection request from '{listener_addr}' (already connected)");
                }
            },
        };
        Ok(())
    }

    /// Verifies the given challenge request. Returns a disconnect reason if the request is invalid.
    fn verify_challenge_request(
        &self,
        peer_addr: SocketAddr,
        message: &ChallengeRequest<N>,
    ) -> Option<DisconnectReason> {
        // Retrieve the components of the challenge request.
        let &ChallengeRequest { version, listener_port: _, node_type: _, address: _, nonce: _ } = message;

        // Ensure the message protocol version is not outdated.
        if !self.is_valid_message_version(version) {
            warn!("Dropping '{peer_addr}' on version {version} (outdated)");
            return Some(DisconnectReason::OutdatedClientVersion);
        }
        None
    }

    /// Verifies the given challenge response. Returns a disconnect reason if the response is invalid.
    #[allow(clippy::too_many_arguments)]
    async fn verify_challenge_response(
        &self,
        peer_addr: SocketAddr,
        peer_address: Address<N>,
        peer_node_type: NodeType,
        response: ChallengeResponse<N>,
        expected_genesis_header: Header<N>,
        expected_restrictions_id: Field<N>,
        expected_nonce: u64,
    ) -> Option<DisconnectReason> {
        // Retrieve the components of the challenge response.
        let ChallengeResponse { genesis_header, restrictions_id, signature, nonce } = response;

        // Verify the challenge response, by checking that the block header matches.
        if genesis_header != expected_genesis_header {
            warn!("Handshake with '{peer_addr}' failed (incorrect block header)");
            return Some(DisconnectReason::InvalidChallengeResponse);
        }
        // Verify the restrictions ID.
        if !peer_node_type.is_prover() && !self.node_type.is_prover() && restrictions_id != expected_restrictions_id {
            warn!("Handshake with '{peer_addr}' failed (incorrect restrictions ID)");
            return Some(DisconnectReason::InvalidChallengeResponse);
        }
        // Perform the deferred non-blocking deserialization of the signature.
        let Ok(signature) = signature.deserialize().await else {
            warn!("Handshake with '{peer_addr}' failed (cannot deserialize the signature)");
            return Some(DisconnectReason::InvalidChallengeResponse);
        };
        // Verify the signature.
        if !signature.verify_bytes(&peer_address, &[expected_nonce.to_le_bytes(), nonce.to_le_bytes()].concat()) {
            warn!("Handshake with '{peer_addr}' failed (invalid signature)");
            return Some(DisconnectReason::InvalidChallengeResponse);
        }
        None
    }
}
