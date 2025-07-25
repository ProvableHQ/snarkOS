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
    num::NonZeroUsize,
};

use anyhow::{Result, bail};
use lru::LruCache;
use snarkvm::{ledger::Transaction, prelude::*};

use crate::{CAPACITY_FOR_DEPLOYMENTS, CAPACITY_FOR_EXECUTIONS};

pub struct TransactionsQueue<N: Network> {
    pub deployments: TransactionsQueueInner<N>,
    pub executions: TransactionsQueueInner<N>,
}

impl<N: Network> Default for TransactionsQueue<N> {
    fn default() -> Self {
        Self {
            deployments: TransactionsQueueInner::new(CAPACITY_FOR_DEPLOYMENTS),
            executions: TransactionsQueueInner::new(CAPACITY_FOR_EXECUTIONS),
        }
    }
}

impl<N: Network> TransactionsQueue<N> {
    pub fn contains(&self, transaction_id: &N::TransactionID) -> bool {
        self.executions.contains(transaction_id) || self.deployments.contains(transaction_id)
    }

    pub fn insert(
        &mut self,
        transaction_id: N::TransactionID,
        transaction: Transaction<N>,
        priority_fee: U64<N>,
    ) -> Result<()> {
        if transaction.is_execute() {
            self.executions.insert(transaction_id, transaction, priority_fee)
        } else {
            self.deployments.insert(transaction_id, transaction, priority_fee)
        }
    }

    pub fn transactions(&self) -> impl Iterator<Item = (N::TransactionID, Transaction<N>)> {
        self.deployments
            .priority_queue
            .transactions
            .clone()
            .into_iter()
            .chain(self.deployments.queue.clone())
            .chain(self.executions.priority_queue.transactions.clone())
            .chain(self.executions.queue.clone())
    }
}

pub struct TransactionsQueueInner<N: Network> {
    capacity: usize,
    queue: LruCache<N::TransactionID, Transaction<N>>,
    priority_queue: PriorityQueue<N>,
}

impl<N: Network> TransactionsQueueInner<N> {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            queue: LruCache::new(NonZeroUsize::new(capacity).unwrap()),
            priority_queue: Default::default(),
        }
    }

    pub fn len(&self) -> usize {
        self.queue.len().saturating_add(self.priority_queue.len())
    }

    fn contains(&self, transaction_id: &N::TransactionID) -> bool {
        self.queue.contains(transaction_id) || self.priority_queue.transactions.contains_key(transaction_id)
    }

    fn insert(
        &mut self,
        transaction_id: N::TransactionID,
        transaction: Transaction<N>,
        priority_fee: U64<N>,
    ) -> Result<()> {
        // If the queue is not full, insert in the appropriate queue.
        if self.len() < self.capacity {
            if priority_fee.is_zero() {
                self.queue.get_or_insert(transaction_id, || transaction);
            } else {
                self.priority_queue.insert(transaction_id, transaction, priority_fee);
            }

            return Ok(());
        }

        match (self.priority_queue.len() < self.capacity, *priority_fee) {
            // Invariant: if the queue is at capacity but the priority queue
            // isn't equal to the capacity, the low-priority queue must be non-empty.
            (true, 0) => {
                let _ = self.queue.get_or_insert(transaction_id, || transaction);
            }
            (true, _fee) => {
                // Remove an entry from the low-priority queue to make room for the high-priority transaction.
                self.queue.pop_lru();
                self.priority_queue.insert(transaction_id, transaction, priority_fee)
            }

            // Invariant: if the queue is at capacity but the priority queue is
            // equal to the capacity, the low-priority queue must be empty.
            (false, 0) => bail!("The memory pool is full"),
            (false, _fee) => self.priority_queue.compare_insert(transaction_id, transaction, priority_fee),
        }

        Ok(())
    }

    pub fn pop(&mut self) -> Option<(N::TransactionID, Transaction<N>)> {
        self.priority_queue.pop().or_else(|| self.queue.pop_lru())
    }
}

struct PriorityQueue<N: Network> {
    /// A counter to ensure fifo ordering for transmissions with the same fee.
    counter: u64,
    /// A map of transmissions ordered by fee and by fifo sequence.
    transaction_ids: BTreeMap<(Reverse<U64<N>>, u64), N::TransactionID>,
    /// A map of transmission IDs to transmissions.
    transactions: HashMap<N::TransactionID, Transaction<N>>,
}

impl<N: Network> Default for PriorityQueue<N> {
    /// Initializes a new instance of the priority queue.
    fn default() -> Self {
        Self { counter: Default::default(), transaction_ids: Default::default(), transactions: Default::default() }
    }
}

impl<N: Network> PriorityQueue<N> {
    fn len(&self) -> usize {
        self.transactions.len()
    }

    fn insert(&mut self, transaction_id: N::TransactionID, transaction: Transaction<N>, fee: U64<N>) {
        if let Entry::Vacant(entry) = self.transactions.entry(transaction_id) {
            // Insert the transaction into the map.
            entry.insert(transaction);
            // Sort by fee (highest first) and counter (fifo).
            self.transaction_ids.insert((Reverse(fee), self.counter), transaction_id);
            // Increment the counter.
            self.counter += 1;
        }
    }

    fn compare_insert(&mut self, transaction_id: N::TransactionID, transaction: Transaction<N>, fee: U64<N>) {
        // Make sure the collection isn't empty.
        if self.transaction_ids.is_empty() {
            return;
        }

        // If the lowest fee in the collection is higher than the new fee, no-op.
        //
        // SAFETY: the empty check guarantees an item will be returned
        let ((Reverse(lowest_fee), _), _) = self.transaction_ids.last_key_value().expect("item must be present");
        if lowest_fee > &fee {
            return;
        }

        // Otherwise, remove the current value and insert the new.
        //
        // SAFETY: the empty check guarantees an item will be returned
        let (_, id) = self.transaction_ids.pop_last().expect("item must be present");
        self.transactions.remove(&id);
        self.insert(transaction_id, transaction, fee);
    }

    fn pop(&mut self) -> Option<(N::TransactionID, Transaction<N>)> {
        let (_, transaction_id) = self.transaction_ids.pop_first()?;
        self.transactions.remove(&transaction_id).map(|transaction| (transaction_id, transaction))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use ledger_test_helpers::{sample_deployment_transaction, sample_execution_transaction_with_fee};
    use snarkvm::prelude::{MainnetV0, TestRng};

    type CurrentNetwork = MainnetV0;

    #[test]
    fn insert_and_pop_low_priority_transactions() {
        let mut rng = TestRng::default();

        /* Executions */

        // Test low-priority execution transaction.
        let execution_transaction = sample_execution_transaction_with_fee(false, &mut rng);
        let execution_id = execution_transaction.id();
        let zero_fee = U64::new(0);

        let mut transactions_queue = TransactionsQueue::<CurrentNetwork>::default();
        assert!(!transactions_queue.contains(&execution_id));
        transactions_queue.insert(execution_id, execution_transaction.clone(), zero_fee).unwrap();

        // Check execution was put into the right queue.
        assert!(transactions_queue.contains(&execution_id));
        assert!(transactions_queue.executions.contains(&execution_id));
        assert!(!transactions_queue.deployments.contains(&execution_id));

        // Pop the execution transaction.
        let (popped_execution_id, popped_execution_transaction) = transactions_queue.executions.pop().unwrap();
        assert_eq!(popped_execution_id, execution_id);
        assert_eq!(popped_execution_transaction, execution_transaction);
        assert!(!transactions_queue.contains(&execution_id));

        /* Deployments */

        // Test low-priority deployment transaction.
        let deployment_transaction = sample_deployment_transaction(false, &mut rng);
        let deployment_id = deployment_transaction.id();

        assert!(!transactions_queue.contains(&deployment_id));
        transactions_queue.insert(deployment_id, deployment_transaction.clone(), zero_fee).unwrap();

        // Check deployment was put into the right queue.
        assert!(transactions_queue.contains(&deployment_id));
        assert!(transactions_queue.deployments.contains(&deployment_id));
        assert!(!transactions_queue.executions.contains(&deployment_id));

        // Pop the deployment transaction.
        let (popped_deployment_id, popped_deployment_transaction) = transactions_queue.deployments.pop().unwrap();
        assert_eq!(popped_deployment_id, deployment_id);
        assert_eq!(popped_deployment_transaction, deployment_transaction);
        assert!(!transactions_queue.contains(&deployment_id));
    }

    #[test]
    fn insert_and_pop_high_priority_transactions() {
        let mut rng = TestRng::default();

        /* Executions */

        // Test high-priority execution transaction.
        let execution_transaction = sample_execution_transaction_with_fee(false, &mut rng);
        let execution_id = execution_transaction.id();
        let high_fee = U64::new(100);

        let mut transactions_queue = TransactionsQueue::<CurrentNetwork>::default();
        assert!(!transactions_queue.contains(&execution_id));
        transactions_queue.insert(execution_id, execution_transaction.clone(), high_fee).unwrap();

        // Check execution was put into the priority queue.
        assert!(transactions_queue.contains(&execution_id));
        assert!(transactions_queue.executions.contains(&execution_id));
        assert!(transactions_queue.executions.priority_queue.transactions.contains_key(&execution_id));

        // Pop the execution transaction.
        let (popped_execution_id, popped_execution_transaction) = transactions_queue.executions.pop().unwrap();
        assert_eq!(popped_execution_id, execution_id);
        assert_eq!(popped_execution_transaction, execution_transaction);
        assert!(!transactions_queue.contains(&execution_id));

        /* Deployments */

        // Test high-priority deployment transaction.
        let deployment_transaction = sample_deployment_transaction(false, &mut rng);
        let deployment_id = deployment_transaction.id();

        assert!(!transactions_queue.contains(&deployment_id));
        transactions_queue.insert(deployment_id, deployment_transaction.clone(), high_fee).unwrap();

        // Check deployment was put into the priority queue.
        assert!(transactions_queue.contains(&deployment_id));
        assert!(transactions_queue.deployments.contains(&deployment_id));
        assert!(transactions_queue.deployments.priority_queue.transactions.contains_key(&deployment_id));

        // Pop the deployment transaction.
        let (popped_deployment_id, popped_deployment_transaction) = transactions_queue.deployments.pop().unwrap();
        assert_eq!(popped_deployment_id, deployment_id);
        assert_eq!(popped_deployment_transaction, deployment_transaction);
        assert!(!transactions_queue.contains(&deployment_id));
    }

    #[test]
    fn insert_and_pop_ordering_with_eviction() {
        let mut rng = TestRng::default();

        let executions: Vec<_> = (0..10)
            .map(|_| {
                let execution_transaction = sample_execution_transaction_with_fee(false, &mut rng);
                (execution_transaction.id(), execution_transaction)
            })
            .collect();

        let mut executions_queue = TransactionsQueueInner::new(4);
        executions_queue.insert(executions[0].0, executions[0].1.clone(), U64::new(300)).unwrap();
        executions_queue.insert(executions[1].0, executions[1].1.clone(), U64::new(0)).unwrap();
        executions_queue.insert(executions[2].0, executions[2].1.clone(), U64::new(100)).unwrap();
        executions_queue.insert(executions[3].0, executions[3].1.clone(), U64::new(200)).unwrap();
        assert_eq!(executions_queue.len(), 4);

        // Insert a high-priority transaction and evict the remaining low-priority transactions.
        executions_queue.insert(executions[4].0, executions[4].1.clone(), U64::new(50)).unwrap();
        assert_eq!(executions_queue.queue.len(), 0);
        assert_eq!(executions_queue.priority_queue.len(), 4);
        assert!(executions_queue.priority_queue.transactions.contains_key(&executions[4].0));
        assert!(!executions_queue.priority_queue.transactions.contains_key(&executions[1].0));

        // Insert a high-priority transaction and evict the lowest high-priority transaction.
        executions_queue.insert(executions[5].0, executions[5].1.clone(), U64::new(150)).unwrap();
        assert_eq!(executions_queue.queue.len(), 0);
        assert_eq!(executions_queue.priority_queue.len(), 4);
        assert!(!executions_queue.priority_queue.transactions.contains_key(&executions[4].0));
        assert!(executions_queue.priority_queue.transactions.contains_key(&executions[5].0));

        // Try to insert a low-priority transaction and expect an error.
        assert!(executions_queue.insert(executions[6].0, executions[6].1.clone(), U64::new(0)).is_err());

        // Pop the transactions in the correct order.
        assert_eq!(executions_queue.pop().unwrap(), executions[0]);
        assert_eq!(executions_queue.pop().unwrap(), executions[3]);
        assert_eq!(executions_queue.pop().unwrap(), executions[5]);
        assert_eq!(executions_queue.pop().unwrap(), executions[2]);

        // Check the queue is empty.
        assert_eq!(executions_queue.len(), 0);
        assert_eq!(executions_queue.queue.len(), 0);
        assert_eq!(executions_queue.priority_queue.len(), 0);
        assert!(executions_queue.pop().is_none());

        // Insert a low-priority transaction and expect it to be inserted into the queue.
        executions_queue.insert(executions[6].0, executions[6].1.clone(), U64::new(0)).unwrap();
        assert_eq!(executions_queue.len(), 1);
        assert_eq!(executions_queue.queue.len(), 1);
        assert_eq!(executions_queue.priority_queue.len(), 0);
    }
}
