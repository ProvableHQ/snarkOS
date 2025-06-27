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

use std::{
    cmp::Reverse,
    collections::{BTreeMap, HashMap, hash_map::Entry},
};

use snarkvm::{console::types::U64, ledger::Transaction, prelude::Network};

#[derive(Clone, Debug)]
pub struct ReadyPriority<N: Network> {
    seq_counter: u64,
    transaction_ids: BTreeMap<(Reverse<U64<N>>, u64), N::TransactionID>,
    transactions: HashMap<N::TransactionID, Transaction<N>>,
}

impl<N: Network> Default for ReadyPriority<N> {
    fn default() -> Self {
        ReadyPriority {
            seq_counter: Default::default(),
            transaction_ids: Default::default(),
            transactions: Default::default(),
        }
    }
}

impl<N: Network> ReadyPriority<N> {
    /// Creates an empty priority queue.
    pub fn new() -> Self {
        Self::default()
    }

    pub fn num_transactions(&self) -> usize {
        self.transactions.len()
    }

    pub fn contains(&self, transaction_id: &N::TransactionID) -> bool {
        self.transactions.contains_key(transaction_id)
    }

    pub fn insert(&mut self, transaction_id: N::TransactionID, transaction: Transaction<N>, fee: U64<N>) -> bool {
        if let Entry::Vacant(entry) = self.transactions.entry(transaction_id) {
            entry.insert(transaction);
            self.transaction_ids.insert((Reverse(fee), self.seq_counter), transaction_id);
            self.seq_counter += 1;
            true
        } else {
            false
        }
    }

    pub fn remove_front(&mut self) -> Option<(N::TransactionID, Transaction<N>)> {
        let (_, transaction_id) = self.transaction_ids.pop_first()?;
        self.transactions.remove(&transaction_id).map(|transaction| (transaction_id, transaction))
    }
}
