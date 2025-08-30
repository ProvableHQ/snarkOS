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

mod common;
use common::*;

use snarkos_node_router::{
    Heartbeat,
    Outbound,
    Router,
    Routing,
    messages::{Message, MessageCodec},
};
use snarkos_node_tcp::{
    ConnectionSide,
    P2P,
    protocols::{Handshake, OnConnect, Writing},
};
use snarkvm::prelude::MainnetV0 as Network;

use async_trait::async_trait;
use std::{net::SocketAddr, time::Duration};
use tokio::time::sleep;

#[derive(Clone)]
struct HeartbeatTest {
    router: TestRouter<Network>,
}

impl P2P for HeartbeatTest {
    fn tcp(&self) -> &snarkos_node_tcp::Tcp {
        self.router.tcp()
    }
}

#[async_trait]
impl Writing for HeartbeatTest {
    type Codec = MessageCodec<Network>;
    type Message = Message<Network>;

    fn codec(&self, _addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Default::default()
    }
}

impl Outbound<Network> for HeartbeatTest {
    fn router(&self) -> &Router<Network> {
        &self.router
    }

    fn is_block_synced(&self) -> bool {
        true
    }

    fn num_blocks_behind(&self) -> Option<u32> {
        None
    }
}

impl Heartbeat<Network> for HeartbeatTest {
    // This number does not actually affect any of the test, because
    // we only test get_removable_peers so far.
    const MAXIMUM_NUMBER_OF_PEERS: usize = 2;
}

/// Initiate connection to peer and wait until it is fully established
async fn connect_to(router: &TestRouter<Network>, other: &TestRouter<Network>) {
    let success = router.connect(other.local_ip()).unwrap().await.unwrap();
    assert!(success, "Connection failed");

    while !router.is_connected(&other.local_ip()) {
        sleep(Duration::from_millis(10)).await;
    }
}

/// Checks that clients are ordered before nodes and that ordering is based on when a peer was last seen.
#[tokio::test]
async fn peer_priority_ordering() {
    let router = client(0, 10).await;
    router.enable_listener().await;
    router.enable_handshake().await;
    router.enable_on_connect().await;

    let validator_peer1 = validator(0, 5, &[], true).await;
    validator_peer1.enable_listener().await;
    validator_peer1.enable_handshake().await;

    let validator_peer2 = validator(0, 5, &[], true).await;
    validator_peer2.enable_listener().await;
    validator_peer2.enable_handshake().await;

    let client_peer = client(0, 5).await;
    client_peer.enable_listener().await;
    client_peer.enable_handshake().await;

    connect_to(&router, &validator_peer1).await;
    connect_to(&router, &client_peer).await;
    connect_to(&router, &validator_peer2).await;

    // Update last seen to affect priority.
    router.update_last_seen_for_connected_peer(validator_peer1.local_ip());
    tokio::time::sleep(Duration::from_millis(500)).await;
    router.update_last_seen_for_connected_peer(validator_peer2.local_ip());

    let heartbeat = HeartbeatTest { router };

    let removable_peers = heartbeat.get_removable_peers();

    // Ensure connections were established.
    assert_eq!(heartbeat.router().get_connected_peers().len(), 3);

    // There are no trusted or bootstrap peers, so everything should be removable.
    assert_eq!(removable_peers.len(), 3);

    let mut removable_peers = removable_peers.into_iter();

    // Client should be lowest priority.
    assert_eq!(removable_peers.next().unwrap().listener_addr, client_peer.local_ip());

    // Validator 1 has lower priority now because it was seen last.
    assert_eq!(removable_peers.next().unwrap().listener_addr, validator_peer1.local_ip());

    // Validator 2 has highest priority.
    assert_eq!(removable_peers.next().unwrap().listener_addr, validator_peer2.local_ip());

    // Update last seen again to check that priorities get updated correctly.
    heartbeat.router.update_last_seen_for_connected_peer(validator_peer1.local_ip());

    // Validator 1 must have higher priority now.
    let mut removable_peers = heartbeat.get_removable_peers().into_iter();
    assert_eq!(removable_peers.next().unwrap().listener_addr, client_peer.local_ip());
    assert_eq!(removable_peers.next().unwrap().listener_addr, validator_peer2.local_ip());
    assert_eq!(removable_peers.next().unwrap().listener_addr, validator_peer1.local_ip());
}

/// Checks that trusted peers are never marked as removable.
#[tokio::test]
async fn peer_priority_trusted_peers() {
    let validator_peer = validator(0, 5, &[], true).await;
    validator_peer.enable_listener().await;
    validator_peer.enable_handshake().await;

    let client_peer = client(0, 5).await;
    client_peer.enable_listener().await;
    client_peer.enable_handshake().await;

    let router = validator(0, 5, &[validator_peer.local_ip(), client_peer.local_ip()], false).await;
    router.enable_listener().await;
    router.enable_handshake().await;
    router.enable_on_connect().await;

    connect_to(&router, &client_peer).await;
    connect_to(&router, &validator_peer).await;

    let heartbeat = HeartbeatTest { router };

    let removable_peers = heartbeat.get_removable_peers();

    // Ensure connections were established.
    assert_eq!(heartbeat.router().get_connected_peers().len(), 2);
    // All peers are trusted and cannot be removed.
    assert_eq!(removable_peers.len(), 0);
}
