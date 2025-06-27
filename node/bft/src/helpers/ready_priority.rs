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

use snarkvm::{
    console::types::U64,
    ledger::{
        Transaction,
        narwhal::{Data, Transmission, TransmissionID},
    },
    prelude::Network,
};

/// Maintains a queue of verified and prioritised ("ready") transmissions.
#[derive(Clone, Debug)]
pub struct ReadyPriority<N: Network> {
    /// A counter to ensure fifo ordering for transmissions with the same fee.
    seq_counter: u64,
    /// A map of transmissions ordered by fee and by fifo sequence.
    transmission_ids: BTreeMap<(Reverse<U64<N>>, u64), TransmissionID<N>>,
    /// A map of transmission IDs to transmissions.
    transmissions: HashMap<TransmissionID<N>, Transmission<N>>,
}

impl<N: Network> Default for ReadyPriority<N> {
    /// Initializes a new instance of the priority queue.
    fn default() -> Self {
        ReadyPriority {
            seq_counter: Default::default(),
            transmission_ids: Default::default(),
            transmissions: Default::default(),
        }
    }
}

impl<N: Network> ReadyPriority<N> {
    /// Creates an empty priority queue.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the number of transmissions in the priority queue.
    pub fn num_transmissions(&self) -> usize {
        self.transmissions.len()
    }

    /// Returns the number of transactions in the priority queue.
    pub fn num_transactions(&self) -> usize {
        // All transmissions in the priority queue are transactions.
        self.transmissions.len()
    }

    /// Returns the transmission IDs in the priority queue.
    pub fn transmission_ids(&self) -> impl Iterator<Item = &TransmissionID<N>> {
        self.transmissions.keys()
    }

    /// Returns the transmissions in the priority queue.
    pub fn transmissions(&self) -> impl Iterator<Item = (&TransmissionID<N>, &Transmission<N>)> {
        self.transmissions.iter()
    }

    /// Returns the transactions in the priority queue.
    pub fn transactions(&self) -> Vec<(N::TransactionID, Data<Transaction<N>>)> {
        self.transmissions
            .iter()
            .filter_map(|(id, transmission)| match (id, transmission) {
                (TransmissionID::Transaction(id, _), Transmission::Transaction(tx)) => Some((*id, tx.clone())),
                _ => None,
            })
            .collect()
    }

    /// Returns `true` if the priority queue contains the specified `transmission ID`.
    pub fn contains(&self, transmission_id: &TransmissionID<N>) -> bool {
        self.transmissions.contains_key(transmission_id)
    }

    /// Returns the transmission, given the specified `transmission ID`.
    pub fn get(&self, transmission_id: &TransmissionID<N>) -> Option<Transmission<N>> {
        self.transmissions.get(transmission_id).cloned()
    }

    /// Inserts the specified (`transmission ID`, `transmission`) to the priority queue.
    /// Returns `true` if the transmission is new, and was added to the priority queue.
    pub fn insert(&mut self, transmission_id: TransmissionID<N>, transmission: Transmission<N>, fee: U64<N>) -> bool {
        if let Entry::Vacant(entry) = self.transmissions.entry(transmission_id) {
            entry.insert(transmission);
            self.transmission_ids.insert((Reverse(fee), self.seq_counter), transmission_id);
            self.seq_counter += 1;
            true
        } else {
            false
        }
    }

    /// Removes and returns the transmission at the front of the priority queue.
    pub fn remove_front(&mut self) -> Option<(TransmissionID<N>, Transmission<N>)> {
        let (_, transmission_id) = self.transmission_ids.pop_first()?;
        self.transmissions.remove(&transmission_id).map(|transmission| (transmission_id, transmission))
    }
}
