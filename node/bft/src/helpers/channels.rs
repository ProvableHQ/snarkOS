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

use crate::events::{TransmissionRequest, TransmissionResponse};
use snarkvm::{console::network::*, ledger::narwhal::TransmissionID};

use std::net::SocketAddr;
use tokio::sync::mpsc;

const MAX_CHANNEL_SIZE: usize = 8192;

#[derive(Debug)]
pub struct WorkerSender<N: Network> {
    pub tx_worker_ping: mpsc::Sender<(SocketAddr, TransmissionID<N>)>,
    pub tx_transmission_request: mpsc::Sender<(SocketAddr, TransmissionRequest<N>)>,
    pub tx_transmission_response: mpsc::Sender<(SocketAddr, TransmissionResponse<N>)>,
}

#[derive(Debug)]
pub struct WorkerReceiver<N: Network> {
    pub rx_worker_ping: mpsc::Receiver<(SocketAddr, TransmissionID<N>)>,
    pub rx_transmission_request: mpsc::Receiver<(SocketAddr, TransmissionRequest<N>)>,
    pub rx_transmission_response: mpsc::Receiver<(SocketAddr, TransmissionResponse<N>)>,
}

/// Initializes the worker channels.
pub fn init_worker_channels<N: Network>() -> (WorkerSender<N>, WorkerReceiver<N>) {
    let (tx_worker_ping, rx_worker_ping) = mpsc::channel(MAX_CHANNEL_SIZE);
    let (tx_transmission_request, rx_transmission_request) = mpsc::channel(MAX_CHANNEL_SIZE);
    let (tx_transmission_response, rx_transmission_response) = mpsc::channel(MAX_CHANNEL_SIZE);

    let sender = WorkerSender { tx_worker_ping, tx_transmission_request, tx_transmission_response };
    let receiver = WorkerReceiver { rx_worker_ping, rx_transmission_request, rx_transmission_response };

    (sender, receiver)
}
