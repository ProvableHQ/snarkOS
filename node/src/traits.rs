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

use crate::router::{Routing, messages::NodeType};

use snarkos_utilities::SignalHandler;

use snarkvm::prelude::{Address, Network, PrivateKey, ViewKey};

use std::time::Duration;

#[async_trait]
pub trait NodeInterface<N: Network>: Routing<N> {
    /// Returns the node type.
    fn node_type(&self) -> NodeType {
        self.router().node_type()
    }

    /// Returns the account private key of the node.
    fn private_key(&self) -> &PrivateKey<N> {
        self.router().private_key()
    }

    /// Returns the account view key of the node.
    fn view_key(&self) -> &ViewKey<N> {
        self.router().view_key()
    }

    /// Returns the account address of the node.
    fn address(&self) -> Address<N> {
        self.router().address()
    }

    /// Returns `true` if the node is in development mode.
    fn is_dev(&self) -> bool {
        self.router().is_dev()
    }

    /// Handles OS signals for the node to intercept and perform a clean shutdown.
    /// The optional `shutdown_flag` flag can be used to cleanly terminate the syncing process.
    async fn wait_for_signals(&self, handler: &SignalHandler) {
        handler.wait_for_signals().await;

        warn!("==========================================================================================");
        warn!("⚠️  Attention - Starting the graceful shutdown procedure (ETA: 30 seconds)...");
        warn!("⚠️  Attention - To avoid DATA CORRUPTION, do NOT interrupt snarkOS (or press Ctrl+C again)");
        warn!("⚠️  Attention - Please wait until the shutdown gracefully completes (ETA: 30 seconds)");
        warn!("==========================================================================================");

        // If the node is already initialized, then shut it down.
        self.shut_down().await;

        // A best-effort attempt to let any ongoing activity conclude.
        tokio::time::sleep(Duration::from_secs(3)).await;
    }

    /// Shuts down the node.
    async fn shut_down(&self);
}
