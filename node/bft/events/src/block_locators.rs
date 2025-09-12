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

use super::{BlockLocators, EventTrait, FromBytes, IoResult, Network, ToBytes};

use std::{
    borrow::Cow,
    io::{Read, Write},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockLocatorsRequest {
    pub start_height: u32,
    pub end_height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockLocatorsResponse<N: Network> {
    pub locators: BlockLocators<N>,
}

impl EventTrait for BlockLocatorsRequest {
    /// Returns the event name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        "BlockLocatorsRequest".into()
    }
}

impl FromBytes for BlockLocatorsRequest {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let start_height = u32::read_le(&mut reader)?;
        let end_height = u32::read_le(&mut reader)?;

        Ok(Self { start_height, end_height })
    }
}

impl ToBytes for BlockLocatorsRequest {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.start_height.write_le(&mut writer)?;
        self.end_height.write_le(&mut writer)?;
        Ok(())
    }
}

impl<N: Network> EventTrait for BlockLocatorsResponse<N> {
    /// Returns the event name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        "BlockLocatorsResponse".into()
    }
}

impl<N: Network> FromBytes for BlockLocatorsResponse<N> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let locators = BlockLocators::read_le(&mut reader)?;
        Ok(Self { locators })
    }
}

impl<N: Network> ToBytes for BlockLocatorsResponse<N> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.locators.write_le(&mut writer)
    }
}
