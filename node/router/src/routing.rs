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

use crate::{Heartbeat, Inbound, Outbound};
use snarkos_node_tcp::{
    P2P,
    protocols::{Disconnect, Handshake, OnConnect, Writing},
};
use snarkvm::prelude::Network;

use std::time::{Duration, Instant};

#[async_trait]
pub trait Routing<N: Network>:
    P2P + Disconnect + OnConnect + Handshake + Inbound<N> + Outbound<N> + Heartbeat<N>
{
    /// Initialize the routing.
    async fn initialize_routing(&self) {
        // Enable the TCP protocols.
        self.enable_handshake().await;
        self.enable_reading().await;
        self.router().enable_writing().await;
        self.enable_disconnect().await;
        self.enable_on_connect().await;
        // Enable the TCP listener. Note: This must be called after the above protocols.
        self.enable_listener().await;
        // Initialize the heartbeat.
        self.initialize_heartbeat();
    }

    // Start listening for inbound connections.
    async fn enable_listener(&self) {
        let listen_addr = self.tcp().enable_listener().await.expect("Failed to enable the TCP listener");
        debug!("Listening for peer connections at address {listen_addr:?}");
    }

    /// Spawns the heartbeat background task for this instance of `Routing`.
    fn initialize_heartbeat(&self) {
        let self_clone = self.clone();
        self.router().spawn(async move {
            // Sleep for `HEARTBEAT_IN_SECS` seconds.
            let min_heartbeat_interval = Duration::from_secs(Self::HEARTBEAT_IN_SECS);
            let mut last_update = Instant::now();

            loop {
                // Process a heartbeat in the router.
                self_clone.heartbeat().await;

                // Figure out how long the heartbeat took
                let now = Instant::now();
                let elapsed = now.saturating_duration_since(last_update);
                last_update = now;

                // (Potentially) sleep to avoid invoking heartbeat too frequently.
                let sleep_time = min_heartbeat_interval.saturating_sub(elapsed);
                if !sleep_time.is_zero() {
                    tokio::time::sleep(sleep_time).await;
                }
            }
        });
    }
}
