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

use crate::MessageTrait;

use snarkvm::prelude::{FromBytes, ToBytes};
use std::{
    borrow::Cow,
    fmt::{self, Write},
    io::{self, Read},
};

#[derive(Clone)]
pub struct MessageChunk {
    /// The hash of the original message.
    pub hash: [u8; 32],
    /// The index of the chunk.
    pub idx: u16,
    /// A flag indicating whether this is the final chunk.
    pub last: bool,
    /// The bytes representing the chunk.
    pub blob: Box<[u8]>,
}

impl fmt::Debug for MessageChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut hash = String::with_capacity(8);
        for byte in &self.hash[..4] {
            write!(hash, "{byte:02x}")?;
        }

        f.debug_struct("MessageChunk").field("hash", &hash).field("idx", &self.idx).field("last", &self.last).finish()
    }
}

impl PartialEq for MessageChunk {
    fn eq(&self, other: &Self) -> bool {
        self.hash == other.hash && self.idx == other.idx
    }
}
impl Eq for MessageChunk {}

impl MessageChunk {
    pub fn new(hash: [u8; 32], idx: u16, last: bool, blob: Box<[u8]>) -> Self {
        Self { hash, idx, last, blob }
    }
}

impl MessageTrait for MessageChunk {
    /// Returns the message name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        let mut hash = String::with_capacity(8);
        for byte in &self.hash[..4] {
            let _ = write!(hash, "{byte:02x}"); // in-memory, shouldn't fail
        }
        format!("Chunk {}{} of Message {}", self.idx, if self.last { " (final)" } else { "" }, hash).into()
    }
}

impl ToBytes for MessageChunk {
    fn write_le<W: io::Write>(&self, mut writer: W) -> io::Result<()> {
        self.hash.write_le(&mut writer)?;
        self.idx.write_le(&mut writer)?;
        self.last.write_le(&mut writer)?;
        (self.blob.len() as u32).write_le(&mut writer)?;
        writer.write_all(&self.blob)?;

        Ok(())
    }
}

impl FromBytes for MessageChunk {
    fn read_le<R: io::Read>(mut reader: R) -> io::Result<Self> {
        let hash = <[u8; 32]>::read_le(&mut reader)?;
        let idx = u16::read_le(&mut reader)?;
        let last = bool::read_le(&mut reader)?;
        let blob_len = u32::read_le(&mut reader)?;
        let mut blob = Vec::new();
        (&mut reader).take(blob_len as u64).read_to_end(&mut blob)?;
        let blob = blob.into();

        Ok(Self { hash, idx, last, blob })
    }
}
