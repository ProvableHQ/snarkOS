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

use super::{BlockLocators, FromBytes, MessageTrait, Network, ToBytes};

use snarkvm::prelude::IoResult;

use std::{
    borrow::Cow,
    io::{Read, Write},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockLocatorsRequest {
    /// The starting block height (inclusive).
    pub start_height: u32,
    /// The ending block height (exclusive).
    pub end_height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockLocatorsResponse<N: Network> {
    pub locators: BlockLocators<N>,
}

impl BlockLocatorsRequest {
    /// Initializes a new block request event.
    pub fn new(start_height: u32, end_height: u32) -> Self {
        Self { start_height, end_height }
    }
}

impl MessageTrait for BlockLocatorsRequest {
    /// Returns the event name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        let start = self.start_height;
        let end = self.end_height;
        match start + 1 == end {
            true => format!("BlockLocatorsRequest {start}"),
            false => format!("BlockLocatorsRequest {start}..{end}"),
        }
        .into()
    }
}

impl ToBytes for BlockLocatorsRequest {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.start_height.write_le(&mut writer)?;
        self.end_height.write_le(&mut writer)
    }
}

impl FromBytes for BlockLocatorsRequest {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let start_height = u32::read_le(&mut reader)?;
        let end_height = u32::read_le(&mut reader)?;

        Ok(Self::new(start_height, end_height))
    }
}

impl<N: Network> MessageTrait for BlockLocatorsResponse<N> {
    /// Returns the event name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        "BlockLocatorsResponse".into()
    }
}

impl<N: Network> ToBytes for BlockLocatorsResponse<N> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.locators.write_le(&mut writer)
    }
}

impl<N: Network> FromBytes for BlockLocatorsResponse<N> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let locators = BlockLocators::read_le(&mut reader)?;
        Ok(Self { locators })
    }
}
