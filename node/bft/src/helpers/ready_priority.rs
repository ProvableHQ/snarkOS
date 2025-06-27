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

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::{
        ledger::narwhal::Data,
        prelude::{Field, TestRng, Uniform},
    };

    use ::bytes::Bytes;
    use indexmap::IndexSet;
    use rand::{Rng, RngCore};

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    #[test]
    fn test_ready_priority() {
        let rng = &mut TestRng::default();

        // Sample random fake bytes.
        let data = |rng: &mut TestRng| Data::Buffer(Bytes::from((0..512).map(|_| rng.gen::<u8>()).collect::<Vec<_>>()));

        // Initialize the priority queue.
        let mut ready_priority = ReadyPriority::<CurrentNetwork>::new();

        // Initialize the transmission IDs.
        let transmission_id_1 = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );
        let transmission_id_2 = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );
        let transmission_id_3 = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );

        // Initialize the transmissions.
        let transmission_1 = Transmission::Transaction(data(rng));
        let transmission_2 = Transmission::Transaction(data(rng));
        let transmission_3 = Transmission::Transaction(data(rng));

        // Insert the transmissions with different fees.
        assert!(ready_priority.insert(transmission_id_1, transmission_1.clone(), U64::new(100u64)));
        assert!(ready_priority.insert(transmission_id_2, transmission_2.clone(), U64::new(200u64)));
        assert!(ready_priority.insert(transmission_id_3, transmission_3.clone(), U64::new(150u64)));

        // Check the number of transmissions.
        assert_eq!(ready_priority.num_transmissions(), 3);
        assert_eq!(ready_priority.num_transactions(), 3);

        // Check the transmission IDs.
        let transmission_ids =
            vec![transmission_id_1, transmission_id_2, transmission_id_3].into_iter().collect::<IndexSet<_>>();
        assert_eq!(ready_priority.transmission_ids().copied().collect::<IndexSet<_>>(), transmission_ids);
        transmission_ids.iter().for_each(|id| assert!(ready_priority.contains(id)));

        // Check that an unknown transmission ID is not in the priority queue.
        let transmission_id_unknown = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );
        assert!(!ready_priority.contains(&transmission_id_unknown));

        // Check the transmissions.
        assert_eq!(ready_priority.get(&transmission_id_1), Some(transmission_1.clone()));
        assert_eq!(ready_priority.get(&transmission_id_2), Some(transmission_2.clone()));
        assert_eq!(ready_priority.get(&transmission_id_3), Some(transmission_3.clone()));
        assert_eq!(ready_priority.get(&transmission_id_unknown), None);

        // Check that transmissions are removed in priority order (highest fee first).
        // transmission_id_2 has fee 200 (highest)
        let (removed_id, removed_transmission) = ready_priority.remove_front().unwrap();
        assert_eq!(removed_id, transmission_id_2);
        assert_eq!(removed_transmission, transmission_2);

        // transmission_id_3 has fee 150 (second highest)
        let (removed_id, removed_transmission) = ready_priority.remove_front().unwrap();
        assert_eq!(removed_id, transmission_id_3);
        assert_eq!(removed_transmission, transmission_3);

        // transmission_id_1 has fee 100 (lowest)
        let (removed_id, removed_transmission) = ready_priority.remove_front().unwrap();
        assert_eq!(removed_id, transmission_id_1);
        assert_eq!(removed_transmission, transmission_1);

        // Check the priority queue is now empty.
        assert_eq!(ready_priority.num_transmissions(), 0);
        assert_eq!(ready_priority.num_transactions(), 0);
        assert!(ready_priority.remove_front().is_none());
    }

    #[test]
    fn test_ready_priority_duplicate() {
        let rng = &mut TestRng::default();

        // Sample random fake bytes.
        let mut vec = vec![0u8; 512];
        rng.fill_bytes(&mut vec);
        let data = Data::Buffer(Bytes::from(vec));

        // Initialize the priority queue.
        let mut ready_priority = ReadyPriority::<CurrentNetwork>::new();

        // Initialize the transmission ID.
        let transmission_id = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );

        // Initialize the transmission.
        let transmission = Transmission::Transaction(data);

        // Insert the transmission ID.
        assert!(ready_priority.insert(transmission_id, transmission.clone(), U64::new(100u64)));
        assert!(!ready_priority.insert(transmission_id, transmission, U64::new(200u64)));

        // Check the number of transmissions.
        assert_eq!(ready_priority.num_transmissions(), 1);
        assert_eq!(ready_priority.num_transactions(), 1);
    }

    #[test]
    fn test_ready_priority_same_fee_fifo() {
        let rng = &mut TestRng::default();

        // Sample random fake bytes.
        let data = |rng: &mut TestRng| Data::Buffer(Bytes::from((0..512).map(|_| rng.gen::<u8>()).collect::<Vec<_>>()));

        // Initialize the priority queue.
        let mut ready_priority = ReadyPriority::<CurrentNetwork>::new();

        // Initialize the transmission IDs.
        let transmission_id_1 = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );
        let transmission_id_2 = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );
        let transmission_id_3 = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );

        // Initialize the transmissions.
        let transmission_1 = Transmission::Transaction(data(rng));
        let transmission_2 = Transmission::Transaction(data(rng));
        let transmission_3 = Transmission::Transaction(data(rng));

        // Insert the transmissions with the same fee (should maintain FIFO order).
        let fee = U64::new(100u64);
        assert!(ready_priority.insert(transmission_id_1, transmission_1.clone(), fee));
        assert!(ready_priority.insert(transmission_id_2, transmission_2.clone(), fee));
        assert!(ready_priority.insert(transmission_id_3, transmission_3.clone(), fee));

        // Check that transmissions are removed in FIFO order when fees are the same.
        let (removed_id, removed_transmission) = ready_priority.remove_front().unwrap();
        assert_eq!(removed_id, transmission_id_1);
        assert_eq!(removed_transmission, transmission_1);

        let (removed_id, removed_transmission) = ready_priority.remove_front().unwrap();
        assert_eq!(removed_id, transmission_id_2);
        assert_eq!(removed_transmission, transmission_2);

        let (removed_id, removed_transmission) = ready_priority.remove_front().unwrap();
        assert_eq!(removed_id, transmission_id_3);
        assert_eq!(removed_transmission, transmission_3);

        // Check the priority queue is now empty.
        assert_eq!(ready_priority.num_transmissions(), 0);
        assert!(ready_priority.remove_front().is_none());
    }

    #[test]
    fn test_ready_priority_mixed_fees() {
        let rng = &mut TestRng::default();

        // Sample random fake bytes.
        let data = |rng: &mut TestRng| Data::Buffer(Bytes::from((0..512).map(|_| rng.gen::<u8>()).collect::<Vec<_>>()));

        // Initialize the priority queue.
        let mut ready_priority = ReadyPriority::<CurrentNetwork>::new();

        // Initialize the transmission IDs.
        let transmission_id_1 = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );
        let transmission_id_2 = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );
        let transmission_id_3 = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );
        let transmission_id_4 = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );

        // Initialize the transmissions.
        let transmission_1 = Transmission::Transaction(data(rng));
        let transmission_2 = Transmission::Transaction(data(rng));
        let transmission_3 = Transmission::Transaction(data(rng));
        let transmission_4 = Transmission::Transaction(data(rng));

        // Insert transmissions with mixed fees and some same fees.
        assert!(ready_priority.insert(transmission_id_1, transmission_1.clone(), U64::new(100u64))); // First with fee 100
        assert!(ready_priority.insert(transmission_id_2, transmission_2.clone(), U64::new(200u64))); // Highest fee
        assert!(ready_priority.insert(transmission_id_3, transmission_3.clone(), U64::new(100u64))); // Second with fee 100
        assert!(ready_priority.insert(transmission_id_4, transmission_4.clone(), U64::new(150u64))); // Middle fee

        // Check that transmissions are removed in priority order:
        // 1. transmission_id_2 (fee 200 - highest)
        let (removed_id, removed_transmission) = ready_priority.remove_front().unwrap();
        assert_eq!(removed_id, transmission_id_2);
        assert_eq!(removed_transmission, transmission_2);

        // 2. transmission_id_4 (fee 150 - second highest)
        let (removed_id, removed_transmission) = ready_priority.remove_front().unwrap();
        assert_eq!(removed_id, transmission_id_4);
        assert_eq!(removed_transmission, transmission_4);

        // 3. transmission_id_1 (fee 100 - first inserted)
        let (removed_id, removed_transmission) = ready_priority.remove_front().unwrap();
        assert_eq!(removed_id, transmission_id_1);
        assert_eq!(removed_transmission, transmission_1);

        // 4. transmission_id_3 (fee 100 - second inserted, FIFO order)
        let (removed_id, removed_transmission) = ready_priority.remove_front().unwrap();
        assert_eq!(removed_id, transmission_id_3);
        assert_eq!(removed_transmission, transmission_3);

        // Check the priority queue is now empty.
        assert_eq!(ready_priority.num_transmissions(), 0);
        assert!(ready_priority.remove_front().is_none());
    }

    #[test]
    fn test_ready_priority_transactions_method() {
        let rng = &mut TestRng::default();

        // Sample random fake bytes.
        let data = |rng: &mut TestRng| Data::Buffer(Bytes::from((0..512).map(|_| rng.gen::<u8>()).collect::<Vec<_>>()));

        // Initialize the priority queue.
        let mut ready_priority = ReadyPriority::<CurrentNetwork>::new();

        // Initialize the transaction IDs directly.
        let tx_id_1 = <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng));
        let tx_id_2 = <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng));

        let transmission_id_1 = TransmissionID::Transaction(
            tx_id_1,
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );
        let transmission_id_2 = TransmissionID::Transaction(
            tx_id_2,
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );

        // Initialize the transmissions.
        let transmission_1 = Transmission::Transaction(data(rng));
        let transmission_2 = Transmission::Transaction(data(rng));

        // Insert the transmissions.
        assert!(ready_priority.insert(transmission_id_1, transmission_1.clone(), U64::new(100u64)));
        assert!(ready_priority.insert(transmission_id_2, transmission_2.clone(), U64::new(200u64)));

        // Check the transactions method returns all transactions.
        let transactions = ready_priority.transactions();
        assert_eq!(transactions.len(), 2);

        // Verify both transactions are present (order doesn't matter for this test).
        let transaction_ids: Vec<_> = transactions.iter().map(|(id, _)| *id).collect();
        assert!(transaction_ids.contains(&tx_id_1));
        assert!(transaction_ids.contains(&tx_id_2));
    }

    #[test]
    fn test_ready_priority_default() {
        let ready_priority = ReadyPriority::<CurrentNetwork>::default();

        assert_eq!(ready_priority.num_transmissions(), 0);
        assert_eq!(ready_priority.num_transactions(), 0);
        assert!(ready_priority.transmission_ids().next().is_none());
        assert!(ready_priority.transmissions().next().is_none());
        assert!(ready_priority.transactions().is_empty());
    }

    #[test]
    fn test_ready_priority_empty_operations() {
        let mut ready_priority = ReadyPriority::<CurrentNetwork>::new();

        // Test operations on empty queue
        assert_eq!(ready_priority.num_transmissions(), 0);
        assert_eq!(ready_priority.num_transactions(), 0);
        assert!(ready_priority.transmission_ids().next().is_none());
        assert!(ready_priority.transmissions().next().is_none());
        assert!(ready_priority.transactions().is_empty());
        assert!(ready_priority.remove_front().is_none());

        // Test contains and get on empty queue
        let rng = &mut TestRng::default();
        let transmission_id = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );
        assert!(!ready_priority.contains(&transmission_id));
        assert_eq!(ready_priority.get(&transmission_id), None);
    }
}
