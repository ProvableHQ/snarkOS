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

use super::SyncToken;

use snarkvm::prelude::{FromBytes, IoResult, ToBytes};

use std::{
    fmt,
    io::{Read, Write},
    net::SocketAddr,
};

/// A response to the `SyncRequest`, providing the data required to access
/// a sync stream.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyncResponse {
    /// The address of the sync stream.
    pub addr: SocketAddr,
    /// A short-lived access token to the sync stream.
    pub token: SyncToken,
}

impl SyncResponse {
    pub fn new(addr: SocketAddr) -> Self {
        Self { addr, token: Default::default() }
    }
}

impl fmt::Debug for SyncResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}@{}", self.token, self.addr)
    }
}

impl fmt::Display for SyncResponse {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SyncRequest {:?}", self)
    }
}

impl ToBytes for SyncResponse {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.addr.write_le(&mut writer)?;
        self.token.write_le(&mut writer)?;
        Ok(())
    }
}

impl FromBytes for SyncResponse {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let addr = SocketAddr::read_le(&mut reader)?;
        let token = SyncToken::read_le(&mut reader)?;
        Ok(Self { addr, token })
    }
}
