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

use snarkos_node_tcp::ConnectError;
use snarkvm::prelude::{FromBytes, ToBytes, io_error};

use std::{io, net::SocketAddr};

/// The reason behind the node disconnecting from a peer.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum DisconnectReason {
    /// The fork length limit was exceeded.
    ExceededForkRange,
    /// The peer's challenge response is invalid.
    InvalidChallengeResponse,
    /// The peer's client uses an invalid fork depth.
    InvalidForkDepth,
    /// The node is a sync node and the peer is ahead.
    INeedToSyncFirst,
    /// No reason given.
    NoReasonGiven,
    /// The peer is not following the protocol.
    ProtocolViolation,
    /// The peer's client is outdated, judging by its version.
    OutdatedClientVersion,
    /// Dropping a dead connection.
    PeerHasDisconnected,
    /// Dropping a connection for a periodic refresh.
    PeerRefresh,
    /// The node is shutting down.
    ShuttingDown,
    /// The sync node has served its purpose.
    SyncComplete,
    /// The peer has caused too many failures.
    TooManyFailures,
    /// The node has too many connections already.
    TooManyPeers,
    /// The peer is a sync node that's behind our node, and it needs to sync itself first.
    YouNeedToSyncFirst,
    /// The peer's listening port is closed
    YourPortIsClosed(u16),
    /// The two peers are the same node.
    SelfConnect,
    /// No untrusted external peers are allowed.
    NoExternalPeersAllowed,
    /// Already connecting to the same node (through another TCP channel).
    AlreadyConnecting,
    /// Already connected to the same node (through another TCP channel).
    AlreadyConnected,
    /// The disconnect reason is not known. This is used for when the peers sends a disconnect reason that is not known to us.
    UnknownReason,
}

impl snarkos_node_tcp::ApplicationError for DisconnectReason {}

impl ToBytes for DisconnectReason {
    fn write_le<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        match self {
            Self::ExceededForkRange => 0u8.write_le(writer),
            Self::InvalidChallengeResponse => 1u8.write_le(writer),
            Self::InvalidForkDepth => 2u8.write_le(writer),
            Self::INeedToSyncFirst => 3u8.write_le(writer),
            Self::NoReasonGiven => 4u8.write_le(writer),
            Self::ProtocolViolation => 5u8.write_le(writer),
            Self::OutdatedClientVersion => 6u8.write_le(writer),
            Self::PeerHasDisconnected => 7u8.write_le(writer),
            Self::PeerRefresh => 8u8.write_le(writer),
            Self::ShuttingDown => 9u8.write_le(writer),
            Self::SyncComplete => 10u8.write_le(writer),
            Self::TooManyFailures => 11u8.write_le(writer),
            Self::TooManyPeers => 12u8.write_le(writer),
            Self::YouNeedToSyncFirst => 13u8.write_le(writer),
            Self::YourPortIsClosed(port) => {
                14u8.write_le(&mut writer)?;
                port.write_le(writer)
            }
            Self::SelfConnect => 15u8.write_le(writer),
            Self::NoExternalPeersAllowed => 16u8.write_le(writer),
            Self::AlreadyConnecting => 17u8.write_le(writer),
            Self::AlreadyConnected => 18u8.write_le(writer),
            Self::UnknownReason => Err(io_error("Cannot serialize unknown disconnect reason")),
        }
    }
}

impl FromBytes for DisconnectReason {
    fn read_le<R: io::Read>(mut reader: R) -> io::Result<Self> {
        let index = match u8::read_le(&mut reader) {
            Ok(index) => index,
            Err(err) => return Err(io_error(format!("Failed to deserialize disconnect reason: {err}"))),
        };

        let reason = match index {
            0 => Self::ExceededForkRange,
            1 => Self::InvalidChallengeResponse,
            2 => Self::InvalidForkDepth,
            3 => Self::INeedToSyncFirst,
            4 => Self::NoReasonGiven,
            5 => Self::ProtocolViolation,
            6 => Self::OutdatedClientVersion,
            7 => Self::PeerHasDisconnected,
            8 => Self::PeerRefresh,
            9 => Self::ShuttingDown,
            10 => Self::SyncComplete,
            11 => Self::TooManyFailures,
            12 => Self::TooManyPeers,
            13 => Self::YouNeedToSyncFirst,
            14 => {
                let port = u16::read_le(reader)?;
                Self::YourPortIsClosed(port)
            }
            15 => Self::SelfConnect,
            16 => Self::NoExternalPeersAllowed,
            17 => Self::AlreadyConnecting,
            18 => Self::AlreadyConnected,
            val => {
                warn!("Received unknown disconnect reason (id={val})");
                Self::UnknownReason
            }
        };

        Ok(reason)
    }
}

impl std::fmt::Display for DisconnectReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ExceededForkRange => write!(f, "exceeded fork range"),
            Self::InvalidChallengeResponse => write!(f, "invalid challenge response"),
            Self::InvalidForkDepth => write!(f, "invalid fork depth"),
            Self::INeedToSyncFirst => write!(f, "I need to sync first"),
            Self::NoReasonGiven => write!(f, "no reason given"),
            Self::ProtocolViolation => write!(f, "protocol violation"),
            Self::OutdatedClientVersion => write!(f, "outdated client version"),
            Self::PeerHasDisconnected => write!(f, "peer has disconnected"),
            Self::PeerRefresh => write!(f, "periodic peer refresh"),
            Self::ShuttingDown => write!(f, "shutting down"),
            Self::SyncComplete => write!(f, "block sync complete"),
            Self::TooManyFailures => write!(f, "too many failures"),
            Self::TooManyPeers => write!(f, "too many peers"),
            Self::YouNeedToSyncFirst => write!(f, "you need to sync first"),
            Self::YourPortIsClosed(port) => write!(f, "your port is closed ({port})"),
            Self::UnknownReason => write!(f, "unknown reason"),
            Self::SelfConnect => write!(f, "self connect"),
            Self::NoExternalPeersAllowed => write!(f, "no external peers allowed"),
            Self::AlreadyConnecting => write!(f, "already connecting"),
            Self::AlreadyConnected => write!(f, "already connected"),
        }
    }
}

impl DisconnectReason {
    pub fn into_connect_error(self, address: SocketAddr) -> ConnectError {
        match self {
            DisconnectReason::SelfConnect => ConnectError::SelfConnect { address },
            DisconnectReason::AlreadyConnected => ConnectError::AlreadyConnected { address },
            DisconnectReason::AlreadyConnecting => ConnectError::AlreadyConnecting { address },
            _ => ConnectError::application(self),
        }
    }
}
