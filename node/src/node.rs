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
    Client,
    Prover,
    Validator,
    router::{Outbound, Router, messages::NodeType},
    traits::NodeInterface,
};

use snarkos_account::Account;
use snarkos_utilities::SignalHandler;

use snarkvm::prelude::{
    Address,
    Ledger,
    Network,
    PrivateKey,
    ViewKey,
    block::Block,
    store::helpers::{memory::ConsensusMemory, rocksdb::ConsensusDB},
};

use aleo_std::StorageMode;
use anyhow::Result;
use std::{net::SocketAddr, sync::Arc};

#[derive(Clone)]
pub enum Node<N: Network> {
    /// A validator is a full node, capable of validating blocks.
    Validator(Arc<Validator<N, ConsensusDB<N>>>),
    /// A prover is a light node, capable of producing proofs for consensus.
    Prover(Arc<Prover<N, ConsensusMemory<N>>>),
    /// A client node is a full node, capable of querying with the network.
    Client(Arc<Client<N, ConsensusDB<N>>>),
}

impl<N: Network> Node<N> {
    /// Initializes a new validator node.
    pub async fn new_validator(
        node_ip: SocketAddr,
        bft_ip: Option<SocketAddr>,
        rest_ip: Option<SocketAddr>,
        rest_rps: u32,
        account: Account<N>,
        trusted_peers: &[SocketAddr],
        trusted_validators: &[SocketAddr],
        genesis: Block<N>,
        cdn: Option<http::Uri>,
        storage_mode: StorageMode,
        allow_external_peers: bool,
        dev_txs: bool,
        dev: Option<u16>,
        signal_handler: Arc<SignalHandler>,
    ) -> Result<Self> {
        Ok(Self::Validator(Arc::new(
            Validator::new(
                node_ip,
                bft_ip,
                rest_ip,
                rest_rps,
                account,
                trusted_peers,
                trusted_validators,
                genesis,
                cdn,
                storage_mode,
                allow_external_peers,
                dev_txs,
                dev,
                signal_handler,
            )
            .await?,
        )))
    }

    /// Initializes a new prover node.
    pub async fn new_prover(
        node_ip: SocketAddr,
        account: Account<N>,
        trusted_peers: &[SocketAddr],
        genesis: Block<N>,
        storage_mode: StorageMode,
        dev: Option<u16>,
        signal_handler: Arc<SignalHandler>,
    ) -> Result<Self> {
        Ok(Self::Prover(Arc::new(
            Prover::new(node_ip, account, trusted_peers, genesis, storage_mode, dev, signal_handler).await?,
        )))
    }

    /// Initializes a new client node.
    pub async fn new_client(
        node_ip: SocketAddr,
        rest_ip: Option<SocketAddr>,
        rest_rps: u32,
        account: Account<N>,
        trusted_peers: &[SocketAddr],
        genesis: Block<N>,
        cdn: Option<http::Uri>,
        storage_mode: StorageMode,
        rotate_external_peers: bool,
        dev: Option<u16>,
        signal_handler: Arc<SignalHandler>,
    ) -> Result<Self> {
        Ok(Self::Client(Arc::new(
            Client::new(
                node_ip,
                rest_ip,
                rest_rps,
                account,
                trusted_peers,
                genesis,
                cdn,
                storage_mode,
                rotate_external_peers,
                dev,
                signal_handler,
            )
            .await?,
        )))
    }

    /// Returns the node type.
    pub fn node_type(&self) -> NodeType {
        match self {
            Self::Validator(validator) => validator.node_type(),
            Self::Prover(prover) => prover.node_type(),
            Self::Client(client) => client.node_type(),
        }
    }

    /// Returns the account private key of the node.
    pub fn private_key(&self) -> &PrivateKey<N> {
        match self {
            Self::Validator(node) => node.private_key(),
            Self::Prover(node) => node.private_key(),
            Self::Client(node) => node.private_key(),
        }
    }

    /// Returns the account view key of the node.
    pub fn view_key(&self) -> &ViewKey<N> {
        match self {
            Self::Validator(node) => node.view_key(),
            Self::Prover(node) => node.view_key(),
            Self::Client(node) => node.view_key(),
        }
    }

    /// Returns the account address of the node.
    pub fn address(&self) -> Address<N> {
        match self {
            Self::Validator(node) => node.address(),
            Self::Prover(node) => node.address(),
            Self::Client(node) => node.address(),
        }
    }

    /// Returns `true` if the node is in development mode.
    pub fn is_dev(&self) -> bool {
        match self {
            Self::Validator(node) => node.is_dev(),
            Self::Prover(node) => node.is_dev(),
            Self::Client(node) => node.is_dev(),
        }
    }

    /// Get the router for P2P networking
    pub fn router(&self) -> &Router<N> {
        match self {
            Self::Validator(node) => node.router(),
            Self::Prover(node) => node.router(),
            Self::Client(node) => node.router(),
        }
    }

    /// Get the underlying ledger (if any).
    pub fn ledger(&self) -> Option<&Ledger<N, ConsensusDB<N>>> {
        match self {
            Self::Validator(node) => Some(node.ledger()),
            Self::Prover(_) => None,
            Self::Client(node) => Some(node.ledger()),
        }
    }

    /// Returns `true` if the node is synced up to the latest block (within the given tolerance).
    pub fn is_block_synced(&self) -> bool {
        match self {
            Self::Validator(node) => node.is_block_synced(),
            Self::Prover(node) => node.is_block_synced(),
            Self::Client(node) => node.is_block_synced(),
        }
    }

    /// Returns the number of blocks this node is behind the greatest peer height,
    /// or `None` if not connected to peers yet.
    pub fn num_blocks_behind(&self) -> Option<u32> {
        match self {
            Self::Validator(node) => node.num_blocks_behind(),
            Self::Prover(node) => node.num_blocks_behind(),
            Self::Client(node) => node.num_blocks_behind(),
        }
    }

    /// Calculates the current sync speed in blocks per second.
    /// Returns None if sync speed cannot be calculated (e.g., not syncing or insufficient data).
    pub fn get_sync_speed(&self) -> f64 {
        match self {
            Self::Validator(node) => node.get_sync_speed(),
            Self::Prover(node) => node.get_sync_speed(),
            Self::Client(node) => node.get_sync_speed(),
        }
    }

    /// Shuts down the node.
    pub async fn shut_down(&self) {
        match self {
            Self::Validator(node) => node.shut_down().await,
            Self::Prover(node) => node.shut_down().await,
            Self::Client(node) => node.shut_down().await,
        }
    }

    /// Waits until the node receives a signal.
    pub async fn wait_for_signals(&self, signal_handler: &SignalHandler) {
        match self {
            Self::Validator(node) => node.wait_for_signals(signal_handler).await,
            Self::Prover(node) => node.wait_for_signals(signal_handler).await,
            Self::Client(node) => node.wait_for_signals(signal_handler).await,
        }
    }
}
