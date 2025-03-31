// Copyright 2024-2025 Aleo Network Foundation
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

use snarkvm::{
    console::prelude::*,
    ledger::{
        block::Transaction,
        narwhal::{Data, Transmission, TransmissionID},
        puzzle::{Solution, SolutionID},
    },
};

use indexmap::{IndexMap, IndexSet};
use std::collections::{HashMap, VecDeque, hash_map::Entry::Vacant};

#[derive(Clone, Debug)]
pub struct Ready<N: Network> {
    /// Maps each transmission ID to its logical index (physical index + offset)
    /// in `transmissions`.
    transmission_ids: HashMap<TransmissionID<N>, i64>,
    /// An ordered collection of (transmission ID, transmission).
    transmissions: VecDeque<(TransmissionID<N>, Transmission<N>)>,
    /// An offset used to adjust logical indices when elements are inserted or
    /// removed at the front.
    offset: i64,
}

impl<N: Network> Default for Ready<N> {
    /// Initializes a new instance of the ready queue.
    fn default() -> Self {
        Self::new()
    }
}

impl<N: Network> Ready<N> {
    /// Initializes a new instance of the ready queue.
    pub fn new() -> Self {
        Self { transmission_ids: Default::default(), transmissions: Default::default(), offset: Default::default() }
    }

    /// Returns `true` if the ready queue is empty.
    pub fn is_empty(&self) -> bool {
        self.transmissions.is_empty()
    }

    /// Returns the number of transmissions in the ready queue.
    pub fn num_transmissions(&self) -> usize {
        self.transmissions.len()
    }

    /// Returns the number of ratifications in the ready queue.
    pub fn num_ratifications(&self) -> usize {
        self.transmission_ids.keys().filter(|id| matches!(id, TransmissionID::Ratification)).count()
    }

    /// Returns the number of solutions in the ready queue.
    pub fn num_solutions(&self) -> usize {
        self.transmission_ids.keys().filter(|id| matches!(id, TransmissionID::Solution(..))).count()
    }

    /// Returns the number of transactions in the ready queue.
    pub fn num_transactions(&self) -> usize {
        self.transmission_ids.keys().filter(|id| matches!(id, TransmissionID::Transaction(..))).count()
    }

    /// Returns the transmission IDs in the ready queue.
    pub fn transmission_ids(&self) -> IndexSet<TransmissionID<N>> {
        self.transmission_ids.keys().copied().collect()
    }

    /// Returns the transmissions in the ready queue.
    pub fn transmissions(&self) -> IndexMap<TransmissionID<N>, Transmission<N>> {
        self.transmissions.iter().cloned().collect()
    }

    /// Returns the solutions in the ready queue.
    pub fn solutions(&self) -> Vec<(SolutionID<N>, Data<Solution<N>>)> {
        self.transmissions
            .iter()
            .filter_map(|(id, transmission)| match (id, transmission) {
                (TransmissionID::Solution(id, _), Transmission::Solution(solution)) => Some((*id, solution.clone())),
                _ => None,
            })
            .collect()
    }

    /// Returns the transactions in the ready queue.
    pub fn transactions(&self) -> Vec<(N::TransactionID, Data<Transaction<N>>)> {
        self.transmissions
            .iter()
            .filter_map(|(id, transmission)| match (id, transmission) {
                (TransmissionID::Transaction(id, _), Transmission::Transaction(tx)) => Some((*id, tx.clone())),
                _ => None,
            })
            .collect()
    }
}

impl<N: Network> Ready<N> {
    /// Returns `true` if the ready queue contains the specified `transmission ID`.
    pub fn contains(&self, transmission_id: impl Into<TransmissionID<N>>) -> bool {
        self.transmission_ids.contains_key(&transmission_id.into())
    }

    /// Returns the transmission, given the specified `transmission ID`.
    pub fn get(&self, transmission_id: impl Into<TransmissionID<N>>) -> Option<Transmission<N>> {
        self.transmission_ids
            .get(&transmission_id.into())
            .and_then(|&index| self.transmissions.get((index - self.offset) as usize))
            .map(|(_, transmission)| transmission.clone())
    }

    /// Inserts the specified (`transmission ID`, `transmission`) to the ready queue.
    /// Returns `true` if the transmission is new, and was added to the ready queue.
    pub fn insert(&mut self, transmission_id: impl Into<TransmissionID<N>>, transmission: Transmission<N>) -> bool {
        let physical_index = self.transmissions.len();
        let transmission_id = transmission_id.into();

        if let Vacant(entry) = self.transmission_ids.entry(transmission_id) {
            entry.insert(physical_index as i64 + self.offset);
            self.transmissions.push_back((transmission_id, transmission));
            true
        } else {
            false
        }
    }

    /// Inserts the specified (`transmission ID`, `transmission`) at the front
    /// of the ready queue.
    /// Returns `true` if the transmission is new, and was added to the ready queue.
    pub fn insert_front(
        &mut self,
        transmission_id: impl Into<TransmissionID<N>>,
        transmission: Transmission<N>,
    ) -> bool {
        let transmission_id = transmission_id.into();
        if let Vacant(entry) = self.transmission_ids.entry(transmission_id) {
            self.offset -= 1;
            let index = self.offset;

            entry.insert(index);
            self.transmissions.push_front((transmission_id, transmission));
            true
        } else {
            false
        }
    }

    /// Removes and returns the transmission at the front of the queue.
    pub fn remove_front(&mut self) -> Option<(TransmissionID<N>, Transmission<N>)> {
        if let Some((transmission_id, transmission)) = self.transmissions.pop_front() {
            self.transmission_ids.remove(&transmission_id);

            if self.transmission_ids.is_empty() {
                debug_assert!(self.transmissions.is_empty());
                self.offset = 0;
            } else {
                self.offset += 1;
            }

            Some((transmission_id, transmission))
        } else {
            None
        }
    }

    /// Removes all solution transmissions from the queue (O(n)).
    pub fn clear_solutions(&mut self) {
        self.transmissions.retain(|(_, transmission)| !matches!(transmission, Transmission::Solution(_)));

        // Rebuild the index and reset the offset.
        self.transmission_ids.clear();
        self.offset = 0;
        for (i, (id, _)) in self.transmissions.iter().enumerate() {
            self.transmission_ids.insert(*id, i as i64);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::{ledger::narwhal::Data, prelude::Field};

    use ::bytes::Bytes;

    type CurrentNetwork = snarkvm::prelude::MainnetV0;

    #[test]
    fn test_ready() {
        let rng = &mut TestRng::default();

        // Sample random fake bytes.
        let data = |rng: &mut TestRng| Data::Buffer(Bytes::from((0..512).map(|_| rng.gen::<u8>()).collect::<Vec<_>>()));

        // Initialize the ready queue.
        let mut ready = Ready::<CurrentNetwork>::new();

        // Initialize the solution IDs.
        let solution_id_1 = TransmissionID::Solution(
            rng.gen::<u64>().into(),
            rng.gen::<<CurrentNetwork as Network>::TransmissionChecksum>(),
        );
        let solution_id_2 = TransmissionID::Solution(
            rng.gen::<u64>().into(),
            rng.gen::<<CurrentNetwork as Network>::TransmissionChecksum>(),
        );
        let solution_id_3 = TransmissionID::Solution(
            rng.gen::<u64>().into(),
            rng.gen::<<CurrentNetwork as Network>::TransmissionChecksum>(),
        );

        // Initialize the solutions.
        let solution_1 = Transmission::Solution(data(rng));
        let solution_2 = Transmission::Solution(data(rng));
        let solution_3 = Transmission::Solution(data(rng));

        // Insert the solution IDs.
        assert!(ready.insert(solution_id_1, solution_1.clone()));
        assert!(ready.insert(solution_id_2, solution_2.clone()));
        assert!(ready.insert(solution_id_3, solution_3.clone()));

        // Check the number of transmissions.
        assert_eq!(ready.num_transmissions(), 3);

        // Check the transmission IDs.
        let transmission_ids = vec![solution_id_1, solution_id_2, solution_id_3].into_iter().collect::<IndexSet<_>>();
        assert_eq!(ready.transmission_ids(), transmission_ids);
        transmission_ids.iter().for_each(|id| assert!(ready.contains(*id)));

        // Check that an unknown solution ID is not in the ready queue.
        let solution_id_unknown = TransmissionID::Solution(
            rng.gen::<u64>().into(),
            rng.gen::<<CurrentNetwork as Network>::TransmissionChecksum>(),
        );
        assert!(!ready.contains(solution_id_unknown));

        // Check the transmissions.
        assert_eq!(ready.get(solution_id_1), Some(solution_1.clone()));
        assert_eq!(ready.get(solution_id_2), Some(solution_2.clone()));
        assert_eq!(ready.get(solution_id_3), Some(solution_3.clone()));
        assert_eq!(ready.get(solution_id_unknown), None);

        // Drain the ready queue.
        let mut transmissions = Vec::with_capacity(3);
        for _ in 0..3 {
            transmissions.push(ready.remove_front().unwrap())
        }

        // Check the number of transmissions.
        assert!(ready.is_empty());
        // Check the transmission IDs.
        assert_eq!(ready.transmission_ids(), IndexSet::new());
        // Check the transmissions.
        assert_eq!(transmissions, vec![
            (solution_id_1, solution_1),
            (solution_id_2, solution_2),
            (solution_id_3, solution_3)
        ]);
    }

    #[test]
    fn test_ready_duplicate() {
        use rand::RngCore;
        let rng = &mut TestRng::default();

        // Sample random fake bytes.
        let mut vec = vec![0u8; 512];
        rng.fill_bytes(&mut vec);
        let data = Data::Buffer(Bytes::from(vec));

        // Initialize the ready queue.
        let mut ready = Ready::<CurrentNetwork>::new();

        // Initialize the solution ID.
        let solution_id = TransmissionID::Solution(
            rng.gen::<u64>().into(),
            rng.gen::<<CurrentNetwork as Network>::TransmissionChecksum>(),
        );

        // Initialize the solution.
        let solution = Transmission::Solution(data);

        // Insert the solution ID.
        assert!(ready.insert(solution_id, solution.clone()));
        assert!(!ready.insert(solution_id, solution));

        // Check the number of transmissions.
        assert_eq!(ready.num_transmissions(), 1);
    }

    #[test]
    fn test_insert_front() {
        let rng = &mut TestRng::default();
        let data = |rng: &mut TestRng| Data::Buffer(Bytes::from((0..512).map(|_| rng.gen::<u8>()).collect::<Vec<_>>()));

        // Initialize the ready queue.
        let mut ready = Ready::<CurrentNetwork>::new();

        // Initialize the solution IDs.
        let solution_id_1 = TransmissionID::Solution(
            rng.gen::<u64>().into(),
            rng.gen::<<CurrentNetwork as Network>::TransmissionChecksum>(),
        );
        let solution_id_2 = TransmissionID::Solution(
            rng.gen::<u64>().into(),
            rng.gen::<<CurrentNetwork as Network>::TransmissionChecksum>(),
        );

        // Initialize the solutions.
        let solution_1 = Transmission::Solution(data(rng));
        let solution_2 = Transmission::Solution(data(rng));

        // Insert the two solutions at the front, check the offset.
        assert!(ready.insert_front(solution_id_1, solution_1.clone()));
        assert_eq!(ready.offset, -1);
        assert!(ready.insert_front(solution_id_2, solution_2.clone()));
        assert_eq!(ready.offset, -2);

        // Check retrieval.
        assert_eq!(ready.get(solution_id_1), Some(solution_1.clone()));
        assert_eq!(ready.get(solution_id_2), Some(solution_2.clone()));

        // Remove from the front, offset should have increased by 1.
        let removed_solution = ready.remove_front().unwrap();
        assert_eq!(removed_solution, (solution_id_2, solution_2));
        assert_eq!(ready.offset, -1);

        // Remove another transmission from the front, the offset should be back to 0.
        let removed_solution = ready.remove_front().unwrap();
        assert_eq!(removed_solution, (solution_id_1, solution_1));
        assert_eq!(ready.offset, 0);
    }

    #[test]
    fn test_clear_solutions() {
        let rng = &mut TestRng::default();
        let solution_data =
            |rng: &mut TestRng| Data::Buffer(Bytes::from((0..512).map(|_| rng.gen::<u8>()).collect::<Vec<_>>()));
        let transaction_data =
            |rng: &mut TestRng| Data::Buffer(Bytes::from((0..512).map(|_| rng.gen::<u8>()).collect::<Vec<_>>()));

        // Initialize the ready queue.
        let mut ready = Ready::<CurrentNetwork>::new();

        // Initialize the solution IDs.
        let solution_id_1 = TransmissionID::Solution(
            rng.gen::<u64>().into(),
            rng.gen::<<CurrentNetwork as Network>::TransmissionChecksum>(),
        );
        let solution_id_2 = TransmissionID::Solution(
            rng.gen::<u64>().into(),
            rng.gen::<<CurrentNetwork as Network>::TransmissionChecksum>(),
        );
        let transaction_id = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.gen::<u128>()),
        );

        // Initialize the transmissions.
        let solution_1 = Transmission::Solution(solution_data(rng));
        let solution_2 = Transmission::Solution(solution_data(rng));
        let transaction = Transmission::Transaction(transaction_data(rng));

        // Insert the solution, check the offset should be decremented.
        assert!(ready.insert_front(solution_id_1, solution_1.clone()));
        assert_eq!(ready.offset, -1);

        // Insert the transaction and the second solution, the offset should remain unchanged.
        assert!(ready.insert(transaction_id, transaction.clone()));
        assert_eq!(ready.offset, -1);
        assert!(ready.insert(solution_id_2, solution_2.clone()));
        assert_eq!(ready.offset, -1);

        // Clear all solution transmissions.
        ready.clear_solutions();
        // Only the transaction should remain.
        assert_eq!(ready.num_transmissions(), 1);
        // The offset should now be reset to 0.
        assert_eq!(ready.offset, 0);
        // The remaining transmission is the transaction.
        assert_eq!(ready.get(transaction_id), Some(transaction));
    }
}
