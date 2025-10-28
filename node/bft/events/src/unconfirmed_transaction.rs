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

use super::*;

use snarkvm::prelude::Transaction;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnconfirmedTransaction<N: Network> {
    pub transaction: Data<Transaction<N>>,
}

impl<N: Network> UnconfirmedTransaction<N> {
    /// Initializes a new transmission response event.
    pub fn new(transaction: Data<Transaction<N>>) -> Self {
        Self { transaction }
    }
}

impl<N: Network> From<Data<Transaction<N>>> for UnconfirmedTransaction<N> {
    /// Initializes a new transmission response event.
    fn from(transaction: Data<Transaction<N>>) -> Self {
        Self::new(transaction)
    }
}

impl<N: Network> EventTrait for UnconfirmedTransaction<N> {
    /// Returns the event name.
    #[inline]
    fn name(&self) -> Cow<'static, str> {
        "UnconfirmedTransaction".into()
    }
}

impl<N: Network> ToBytes for UnconfirmedTransaction<N> {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.transaction.write_le(&mut writer)?;
        Ok(())
    }
}

impl<N: Network> FromBytes for UnconfirmedTransaction<N> {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let transaction = Data::read_le(&mut reader)?;

        Ok(Self { transaction })
    }
}

#[cfg(test)]
pub mod prop_tests {
    use crate::UnconfirmedTransaction;
    use snarkvm::{
        console::prelude::{FromBytes, ToBytes},
        ledger::narwhal::Data,
        prelude::Transaction,
    };

    use bytes::{Buf, BufMut, Bytes, BytesMut};
    use proptest::{
        collection,
        prelude::{BoxedStrategy, Strategy, any},
        prop_oneof,
    };
    use test_strategy::proptest;

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    pub fn any_transaction() -> BoxedStrategy<Data<Transaction<CurrentNetwork>>> {
        prop_oneof![(collection::vec(any::<u8>(), 512..=512)).prop_map(|bytes| (Data::Buffer(Bytes::from(bytes)))),]
            .boxed()
    }

    pub fn any_unconfirmed_transaction() -> BoxedStrategy<UnconfirmedTransaction<CurrentNetwork>> {
        any_transaction().prop_map(UnconfirmedTransaction::new).boxed()
    }

    #[proptest]
    fn serialize_deserialize(
        #[strategy(any_unconfirmed_transaction())] original: UnconfirmedTransaction<CurrentNetwork>,
    ) {
        let mut buf = BytesMut::default().writer();
        UnconfirmedTransaction::write_le(&original, &mut buf).unwrap();

        let deserialized = UnconfirmedTransaction::read_le(buf.into_inner().reader()).unwrap();
        assert_eq!(original, deserialized);
    }
}
