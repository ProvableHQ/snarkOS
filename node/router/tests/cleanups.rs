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

use deadline::deadline;
use peak_alloc::PeakAlloc;
use proptest::prelude::*;
use snarkos_node_network::{NodeType, Peer, PeerPoolHandling};
use snarkos_node_router::{Outbound, Routing};
use snarkos_node_tcp::protocols::{Disconnect, Handshake, OnConnect};
use snarkvm::utilities::TestRng;

use core::time::Duration;

#[global_allocator]
static PEAK_ALLOC: PeakAlloc = PeakAlloc;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(9))]
    #[test_log::test]
    fn test_connection_cleanups(node_types in (0u8..=2, 0u8..=2)) {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            // The number of connections to start and close.
            const NUM_CONNECTIONS: usize = 10;

            // Initialize an Rng.
            let mut rng = TestRng::default();

            // Create 2 routers of specified types.
            let mut nodes = Vec::with_capacity(2);
            for node_type in [node_types.0, node_types.1] {
                let node = match node_type {
                    0 => client(0, 1, &mut rng).await,
                    1 => prover(0, 1, &mut rng).await,
                    2 => validator(0, 1, &[], false, &mut rng).await,
                    _ => unreachable!(),
                };

                nodes.push(node);
            }

            // Enable protocols for all nodes
            for node in nodes.iter() {
                node.enable_handshake().await;
                node.enable_disconnect().await;
                node.enable_on_connect().await;
                node.enable_listener().await;
            }

            for node in nodes.iter() {


              // Validators need to add other nodes as trusted peers, so they can connect.
               if node.node_type() == NodeType::Validator {
                for other in nodes.iter().filter(|other| other.local_ip() != node.local_ip()) {
                    node.router().peer_pool().write().insert(other.local_ip(), Peer::new_candidate(other.local_ip(), true));
                }
               }
            }

            // We'll want to register heap use after a single connection, after the related collections are initialized.
            let mut heap_after_one_conn = None;

            // Connect and disconnect in a loop.
            for i in 0..NUM_CONNECTIONS {
                // Connect one of the nodes to the other one.
                nodes[1].connect(nodes[0].local_ip());

                // Wait until the connection is complete.
                let node0 = nodes[0].clone();
                let node1 = nodes[1].clone();
                deadline!(Duration::from_secs(3), move || node0.router().number_of_connected_peers() == 1
                    && node1.router().number_of_connected_peers() == 1);

                // Since the connectee doesn't read from the connector, it can't tell that the connector disconnected
                // from it, so it needs to disconnect from it manually.
                nodes[0].disconnect(nodes[1].local_ip());
                nodes[1].disconnect(nodes[0].local_ip());

                // Wait until the disconnect is complete.
                let node0 = nodes[0].clone();
                let node1 = nodes[1].clone();
                deadline!(Duration::from_secs(3), move || node0.router().number_of_connected_peers() == 0
                    && node1.router().number_of_connected_peers() == 0);

                // Register heap use after a single connection.
                if i == 0 {
                    heap_after_one_conn = Some(PEAK_ALLOC.current_usage());
                }
            }

            // Register final heap use.
            let heap_after_loop = PEAK_ALLOC.current_usage();

            // Final heap use should equal that after the first connection.
            assert_eq!(heap_after_one_conn.unwrap(), heap_after_loop);

            for node in nodes {
                node.shut_down().await;
            }
        });
    }
}
