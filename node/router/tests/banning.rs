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

// mod common;
// use common::*;

// use snarkos_node_network::PeerPoolHandling;
// use snarkos_node_tcp::{
//     P2P,
//     protocols::{Disconnect, Handshake},
// };
// use snarkvm::prelude::TestRng;

// use std::time::Duration;

// #[tokio::test]
// async fn ban_connection_responder() {
//     let mut rng = TestRng::default();

//     // Create 2 client routers.
//     let node0 = client(0, 1, &mut rng).await;
//     let node1 = client(0, 1, &mut rng).await;

//     // Start listening on both sides.
//     node0.tcp().enable_listener().await.unwrap();
//     node1.tcp().enable_listener().await.unwrap();
//     // Enable handshake protocol in order to populate the peer pool.
//     node0.enable_handshake().await;
//     node1.enable_handshake().await;
//     // Enable disconnect protocol in order for the ban to update the peer pool.
//     node0.enable_disconnect().await;

//     // Connect node0 to node1.
//     let _ = node0.connect(node1.local_ip()).unwrap().await;
//     // Wait a moment to ensure that node1 has fully registered the connection too.
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     // Ensure that everyone is fully connected.
//     assert_eq!(node0.tcp().num_connected(), 1);
//     assert_eq!(node1.tcp().num_connected(), 1);
//     assert_eq!(node0.connected_peers().len(), 1);
//     assert_eq!(node1.connected_peers().len(), 1);

//     // Make node0 ban node1.
//     let node1_addr = node1.tcp().listening_addr().unwrap();
//     node0.ip_ban_peer(node1_addr, None);
//     // Wait a moment to ensure that the associated disconnect has concluded.
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     // Ensure that node0 is fully disconnected from node1; node1 may not be aware
//     // of it yet, so don't check it.
//     assert!(node0.is_ip_banned(node1_addr.ip()));
//     assert_eq!(node0.tcp().num_connected(), 0);
//     assert_eq!(node0.connected_peers().len(), 0);
// }

// #[tokio::test]
// async fn ban_connection_initiator() {
//     let mut rng = TestRng::default();

//     // Create 2 client routers.
//     let node0 = client(0, 1, &mut rng).await;
//     let node1 = client(0, 1, &mut rng).await;

//     // Start listening on both sides.
//     node0.tcp().enable_listener().await.unwrap();
//     node1.tcp().enable_listener().await.unwrap();
//     // Enable handshake protocol in order to populate the peer pool.
//     node0.enable_handshake().await;
//     node1.enable_handshake().await;
//     // Enable disconnect protocol in order for the ban to update the peer pool.
//     node0.enable_disconnect().await;

//     // Connect node1 to node0.
//     let _ = node1.connect(node0.local_ip()).unwrap().await;
//     // Wait a moment to ensure that node1 has fully registered the connection too.
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     // Ensure that everyone is fully connected.
//     assert_eq!(node0.tcp().num_connected(), 1);
//     assert_eq!(node1.tcp().num_connected(), 1);
//     assert_eq!(node0.connected_peers().len(), 1);
//     assert_eq!(node1.connected_peers().len(), 1);

//     // Make node0 ban node1.
//     let node1_addr = node1.tcp().listening_addr().unwrap();
//     node0.ip_ban_peer(node1_addr, None);
//     // Wait a moment to ensure that the associated disconnect has concluded.
//     tokio::time::sleep(Duration::from_millis(100)).await;

//     // Ensure that node0 is fully disconnected from node1; node1 may not be aware
//     // of it yet, so don't check it.
//     assert!(node0.is_ip_banned(node1_addr.ip()));
//     assert_eq!(node0.tcp().num_connected(), 0);
//     assert_eq!(node0.connected_peers().len(), 0);
// }
