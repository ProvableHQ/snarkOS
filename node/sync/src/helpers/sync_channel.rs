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

use crate::{InsertBlockResponseError, locators::BlockLocators};

use snarkos_node_network::{CertificateRequest, CertificateResponse};
use snarkvm::prelude::{Block, ConsensusVersion, Network};

use anyhow::{Result, anyhow};
use std::net::SocketAddr;
use tokio::sync::{mpsc, oneshot};

const MAX_CHANNEL_SIZE: usize = 8192;

#[derive(Clone, Debug)]
pub struct SyncSender<N: Network> {
    tx_block_sync_insert_block_response: mpsc::Sender<(
        SocketAddr,
        Vec<Block<N>>,
        Option<ConsensusVersion>,
        oneshot::Sender<Result<(), InsertBlockResponseError<N>>>,
    )>,
    pub tx_block_sync_remove_peer: mpsc::Sender<(SocketAddr, oneshot::Sender<()>)>,
    tx_block_sync_update_peer_locators: mpsc::Sender<(SocketAddr, BlockLocators<N>, oneshot::Sender<Result<()>>)>,
    pub tx_certificate_request: mpsc::Sender<(SocketAddr, CertificateRequest<N>)>,
    pub tx_certificate_response: mpsc::Sender<(SocketAddr, CertificateResponse<N>)>,
}

impl<N: Network> SyncSender<N> {
    /// Sends the request to update the peer locators.
    pub async fn update_peer_locators(&self, peer_ip: SocketAddr, block_locators: BlockLocators<N>) -> Result<()> {
        // Initialize a callback sender and receiver.
        let (callback_sender, callback_receiver) = oneshot::channel();
        // Send the request to update the peer locators.
        // This `tx_block_sync_update_peer_locators.send()` call
        // causes the `rx_block_sync_update_peer_locators.recv()` call
        // in one of the loops in [`Sync::run()`] to return.
        self.tx_block_sync_update_peer_locators.send((peer_ip, block_locators, callback_sender)).await?;
        // Await the callback to continue.
        callback_receiver.await?
    }

    /// Sends the request to insert a new block response.
    pub async fn insert_block_response(
        &self,
        peer_ip: SocketAddr,
        blocks: Vec<Block<N>>,
        latest_consensus_version: Option<ConsensusVersion>,
    ) -> Result<(), InsertBlockResponseError<N>> {
        // Initialize a callback sender and receiver.
        let (callback_sender, callback_receiver) = oneshot::channel();
        // Send the request to advance with sync blocks.
        // This `tx_block_sync_advance_with_sync_blocks.send()` call
        // causes the `rx_block_sync_advance_with_sync_blocks.recv()` call
        // in one of the loops in [`Sync::run()`] to return.
        if let Err(err) = self
            .tx_block_sync_insert_block_response
            .send((peer_ip, blocks, latest_consensus_version, callback_sender))
            .await
        {
            return Err(anyhow!("Failed to send block response - {err}").into());
        }

        // Await the callback to continue.
        match callback_receiver.await {
            Ok(result) => result,
            Err(err) => Err(anyhow!("Failed to wait for block response insertion - {err}").into()),
        }
    }
}

#[derive(Debug)]
pub struct SyncReceiver<N: Network> {
    pub rx_block_sync_insert_block_response: mpsc::Receiver<(
        SocketAddr,
        Vec<Block<N>>,
        Option<ConsensusVersion>,
        oneshot::Sender<Result<(), InsertBlockResponseError<N>>>,
    )>,
    pub rx_block_sync_remove_peer: mpsc::Receiver<(SocketAddr, oneshot::Sender<()>)>,
    pub rx_block_sync_update_peer_locators: mpsc::Receiver<(SocketAddr, BlockLocators<N>, oneshot::Sender<Result<()>>)>,
    pub rx_certificate_request: mpsc::Receiver<(SocketAddr, CertificateRequest<N>)>,
    pub rx_certificate_response: mpsc::Receiver<(SocketAddr, CertificateResponse<N>)>,
}

/// Initializes the sync channels.
pub fn init_sync_channels<N: Network>() -> (SyncSender<N>, SyncReceiver<N>) {
    let (tx_block_sync_insert_block_response, rx_block_sync_insert_block_response) = mpsc::channel(MAX_CHANNEL_SIZE);
    let (tx_block_sync_remove_peer, rx_block_sync_remove_peer) = mpsc::channel(MAX_CHANNEL_SIZE);
    let (tx_block_sync_update_peer_locators, rx_block_sync_update_peer_locators) = mpsc::channel(MAX_CHANNEL_SIZE);
    let (tx_certificate_request, rx_certificate_request) = mpsc::channel(MAX_CHANNEL_SIZE);
    let (tx_certificate_response, rx_certificate_response) = mpsc::channel(MAX_CHANNEL_SIZE);

    let sender = SyncSender {
        tx_block_sync_insert_block_response,
        tx_block_sync_remove_peer,
        tx_block_sync_update_peer_locators,
        tx_certificate_request,
        tx_certificate_response,
    };
    let receiver = SyncReceiver {
        rx_block_sync_insert_block_response,
        rx_block_sync_remove_peer,
        rx_block_sync_update_peer_locators,
        rx_certificate_request,
        rx_certificate_response,
    };

    (sender, receiver)
}
