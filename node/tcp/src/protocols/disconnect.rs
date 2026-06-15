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

use std::{net::SocketAddr, time::Duration};

use tokio::{
    sync::{mpsc, oneshot},
    task::JoinHandle,
    time::timeout,
};
use tracing::*;

#[cfg(doc)]
use crate::{Connection, protocols::Writing};
use crate::{
    P2P,
    connections::{DisconnectOrigin, create_connection_span},
    protocols::ProtocolHandler,
};

/// Can be used to automatically perform some extra actions when the node disconnects from its
/// peer, which is especially practical if the disconnect is triggered automatically, e.g. due
/// to the peer exceeding the allowed number of failures or severing its connection with the node
/// on its own.
#[async_trait::async_trait]
pub trait Disconnect: P2P
where
    Self: Clone + Send + Sync + 'static,
{
    /// The maximum time allowed for the on_disconnect hook to execute.
    /// If the hook exceeds this time, it will be aborted to ensure the node cleans up
    /// resources promptly.
    const TIMEOUT: Duration = Duration::from_secs(3);

    /// Attaches the behavior specified in [`Disconnect::handle_disconnect`] to every occurrence of the
    /// node disconnecting from a peer.
    async fn enable_disconnect(&self) {
        let (from_node_sender, mut from_node_receiver) = mpsc::channel::<(
            (SocketAddr, DisconnectOrigin),
            oneshot::Sender<(JoinHandle<()>, oneshot::Receiver<()>)>,
        )>(self.tcp().config().max_connections as usize);

        // use a channel to know when the disconnect task is ready
        let (tx, rx) = oneshot::channel::<()>();

        // spawn a background task dedicated to handling disconnect events
        let self_clone = self.clone();
        let disconnect_task = tokio::spawn(async move {
            trace!(parent: self_clone.tcp().span(), "spawned the Disconnect handler task");
            tx.send(()).unwrap(); // safe; the channel was just opened

            while let Some(((peer_addr, origin), notifier)) = from_node_receiver.recv().await {
                let self_clone2 = self_clone.clone();
                // create a channel for waiting on completion
                let (done_tx, done_rx) = oneshot::channel();
                let handle = tokio::spawn(async move {
                    // perform the specified extra actions
                    if timeout(Self::TIMEOUT, self_clone2.handle_disconnect(peer_addr, origin)).await.is_err() {
                        let conn_span = create_connection_span(peer_addr, self_clone2.tcp().span());
                        warn!(parent: conn_span, "Disconnect logic timed out");
                    }
                    // notify the node that the extra actions have concluded
                    // and that the related connection can be dropped
                    let _ = done_tx.send(());
                });
                // provide the node with a handle to the scheduled task,
                // and a receiver that will notify it of its completion
                let _ = notifier.send((handle, done_rx)); // can't really fail
            }
        });
        let _ = rx.await;
        self.tcp().tasks.lock().push(disconnect_task);

        // register the Disconnect handler with the Tcp
        let hdl = Box::new(ProtocolHandler(from_node_sender));
        assert!(
            self.tcp().protocols.disconnect.set(hdl).is_ok(),
            "the Disconnect protocol was enabled more than once!"
        );
    }

    /// Any extra actions to be executed during a disconnect; in order to still be able to
    /// communicate with the peer in the usual manner (i.e. via [`Writing`]), only its [`SocketAddr`]
    /// (as opposed to the related [`Connection`] object) is provided as an argument.
    async fn handle_disconnect(&self, peer_addr: SocketAddr, origin: DisconnectOrigin);
}
