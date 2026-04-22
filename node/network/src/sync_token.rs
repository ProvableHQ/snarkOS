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

use snarkvm::prelude::{FromBytes, IoResult, ToBytes};

use rand::RngExt;
use std::{
    fmt,
    io::{Read, Write},
    ops::Deref,
};

/// A short-lived, randomly generated access token to a sync stream.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SyncToken([u8; 32]);

impl Default for SyncToken {
    fn default() -> Self {
        let mut token = [0u8; 32];
        rand::rng().fill(&mut token);
        Self(token)
    }
}

impl From<[u8; 32]> for SyncToken {
    fn from(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }
}

impl Deref for SyncToken {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0[..]
    }
}

impl fmt::Debug for SyncToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hex_token: String = self.0.iter().map(|b| format!("{:02x}", b)).collect();
        write!(f, "{hex_token}")
    }
}

impl fmt::Display for SyncToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SyncToken {:?}", self)
    }
}

impl ToBytes for SyncToken {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.0.write_le(&mut writer)?;
        Ok(())
    }
}

impl FromBytes for SyncToken {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let token = <[u8; 32]>::read_le(&mut reader)?;
        Ok(Self(token))
    }
}
