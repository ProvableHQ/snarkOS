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

use crate::StorageService;
use snarkvm::{
    ledger::narwhal::{BatchHeader, Transmission, TransmissionID},
    prelude::{Field, Network, Result, bail},
};

use indexmap::{IndexMap, IndexSet, indexset, map::Entry};
#[cfg(feature = "locktick")]
use locktick::parking_lot::RwLock;
#[cfg(not(feature = "locktick"))]
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use tracing::error;

/// A BFT in-memory storage service.
#[derive(Debug)]
pub struct BFTMemoryService<N: Network> {
    /// The map of `transmission ID` to `(transmission, certificate IDs)` entries.
    transmissions: RwLock<IndexMap<TransmissionID<N>, (Transmission<N>, IndexSet<Field<N>>)>>,
    /// The map of `aborted transmission ID` to `certificate IDs` entries.
    aborted_transmission_ids: RwLock<IndexMap<TransmissionID<N>, IndexSet<Field<N>>>>,
}

impl<N: Network> Default for BFTMemoryService<N> {
    /// Initializes a new BFT in-memory storage service.
    fn default() -> Self {
        Self::new()
    }
}

impl<N: Network> BFTMemoryService<N> {
    /// Initializes a new BFT in-memory storage service.
    pub fn new() -> Self {
        Self { transmissions: Default::default(), aborted_transmission_ids: Default::default() }
    }
}

impl<N: Network> StorageService<N> for BFTMemoryService<N> {
    /// Returns `true` if the storage contains the specified `transmission ID`.
    fn contains_transmission(&self, transmission_id: TransmissionID<N>) -> bool {
        // Check if the transmission ID exists in storage.
        self.transmissions.read().contains_key(&transmission_id)
            || self.aborted_transmission_ids.read().contains_key(&transmission_id)
    }

    /// Returns the transmission for the given `transmission ID`.
    /// If the transmission does not exist in storage, `None` is returned.
    fn get_transmission(&self, transmission_id: TransmissionID<N>) -> Option<Transmission<N>> {
        // Get the transmission.
        self.transmissions.read().get(&transmission_id).map(|(transmission, _)| transmission).cloned()
    }

    /// Returns the missing transmissions in storage from the given transmissions.
    fn find_missing_transmissions(
        &self,
        batch_header: &BatchHeader<N>,
        mut transmissions: HashMap<TransmissionID<N>, Transmission<N>>,
        aborted_transmissions: HashSet<TransmissionID<N>>,
    ) -> Result<HashMap<TransmissionID<N>, Transmission<N>>> {
        // Initialize a list for the missing transmissions from storage.
        let mut missing_transmissions = HashMap::new();
        // Lock the existing transmissions.
        let known_transmissions = self.transmissions.read();
        // Ensure the declared transmission IDs are all present in storage or the given transmissions map.
        for transmission_id in batch_header.transmission_ids() {
            // If the transmission ID does not exist, ensure it was provided by the caller or aborted.
            if !known_transmissions.contains_key(transmission_id) {
                // Retrieve the transmission.
                match transmissions.remove(transmission_id) {
                    // Append the transmission if it exists.
                    Some(transmission) => {
                        missing_transmissions.insert(*transmission_id, transmission);
                    }
                    // If the transmission does not exist, check if it was aborted.
                    None => {
                        if !aborted_transmissions.contains(transmission_id) {
                            bail!("Failed to provide a transmission");
                        }
                    }
                }
            }
        }
        Ok(missing_transmissions)
    }

    /// Inserts the given certificate ID for each of the transmission IDs, using the missing transmissions map, into storage.
    fn insert_transmissions(
        &self,
        certificate_id: Field<N>,
        transmission_ids: IndexSet<TransmissionID<N>>,
        aborted_transmission_ids: HashSet<TransmissionID<N>>,
        mut missing_transmissions: HashMap<TransmissionID<N>, Transmission<N>>,
    ) {
        // Acquire the transmissions write lock.
        let mut transmissions = self.transmissions.write();
        // Acquire the aborted transmission IDs write lock.
        let mut aborted_transmission_ids_lock = self.aborted_transmission_ids.write();
        // Inserts the following:
        //   - Inserts **only the missing** transmissions from storage.
        //   - Inserts the certificate ID into the corresponding set for **all** transmissions.
        'outer: for transmission_id in transmission_ids {
            // Retrieve the transmission entry.
            match transmissions.entry(transmission_id) {
                Entry::Occupied(mut occupied_entry) => {
                    let (_, certificate_ids) = occupied_entry.get_mut();
                    // Insert the certificate ID into the set.
                    certificate_ids.insert(certificate_id);
                }
                Entry::Vacant(vacant_entry) => {
                    // Retrieve the missing transmission.
                    let Some(transmission) = missing_transmissions.remove(&transmission_id) else {
                        if !aborted_transmission_ids.contains(&transmission_id)
                            && !self.contains_transmission(transmission_id)
                        {
                            error!("Failed to provide a missing transmission {transmission_id}");
                        }
                        continue 'outer;
                    };
                    // Prepare the set of certificate IDs.
                    let certificate_ids = indexset! { certificate_id };
                    // Insert the transmission and a new set with the certificate ID.
                    vacant_entry.insert((transmission, certificate_ids));
                }
            }
        }
        // Inserts the aborted transmission IDs.
        for aborted_transmission_id in aborted_transmission_ids {
            // Retrieve the transmission entry.
            match aborted_transmission_ids_lock.entry(aborted_transmission_id) {
                Entry::Occupied(mut occupied_entry) => {
                    let certificate_ids = occupied_entry.get_mut();
                    // Insert the certificate ID into the set.
                    certificate_ids.insert(certificate_id);
                }
                Entry::Vacant(vacant_entry) => {
                    // Prepare the set of certificate IDs.
                    let certificate_ids = indexset! { certificate_id };
                    // Insert the transmission and a new set with the certificate ID.
                    vacant_entry.insert(certificate_ids);
                }
            }
        }
    }

    /// Removes the certificate ID for the transmissions from storage.
    ///
    /// If the transmission no longer references any certificate IDs, the entry is removed from storage.
    fn remove_transmissions(&self, certificate_id: &Field<N>, transmission_ids: &IndexSet<TransmissionID<N>>) {
        // Acquire the transmissions write lock.
        let mut transmissions = self.transmissions.write();
        // Acquire the aborted transmission IDs write lock.
        let mut aborted_transmission_ids = self.aborted_transmission_ids.write();
        // If this is the last certificate ID for the transmission ID, remove the transmission.
        for transmission_id in transmission_ids {
            // Remove the certificate ID for the transmission ID, and determine if there are any more certificate IDs.
            match transmissions.entry(*transmission_id) {
                Entry::Occupied(mut occupied_entry) => {
                    let (_, certificate_ids) = occupied_entry.get_mut();
                    // Remove the certificate ID for the transmission ID.
                    certificate_ids.swap_remove(certificate_id);
                    // If there are no more certificate IDs for the transmission ID, remove the transmission.
                    if certificate_ids.is_empty() {
                        // Remove the entry for the transmission ID.
                        occupied_entry.shift_remove();
                    }
                }
                Entry::Vacant(_) => {}
            }
            // Remove the certificate ID for the aborted transmission ID, and determine if there are any more certificate IDs.
            match aborted_transmission_ids.entry(*transmission_id) {
                Entry::Occupied(mut occupied_entry) => {
                    let certificate_ids = occupied_entry.get_mut();
                    // Remove the certificate ID for the transmission ID.
                    certificate_ids.swap_remove(certificate_id);
                    // If there are no more certificate IDs for the transmission ID, remove the transmission.
                    if certificate_ids.is_empty() {
                        // Remove the entry for the transmission ID.
                        occupied_entry.shift_remove();
                    }
                }
                Entry::Vacant(_) => {}
            }
        }
    }

    /// Returns a HashMap over the `(transmission ID, (transmission, certificate IDs))` entries.
    #[cfg(any(test, feature = "test"))]
    fn as_hashmap(&self) -> HashMap<TransmissionID<N>, (Transmission<N>, IndexSet<Field<N>>)> {
        self.transmissions.read().clone().into_iter().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use snarkvm::{
        console::network::MainnetV0,
        ledger::narwhal::Data,
        prelude::{PrivateKey, Rng, TestRng, Uniform},
    };

    use bytes::Bytes;
    use std::{sync::mpsc, time::Duration};

    type CurrentNetwork = MainnetV0;

    fn sample_transmission_id(rng: &mut TestRng) -> TransmissionID<CurrentNetwork> {
        TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.random::<u128>()),
        )
    }

    fn sample_transmission(payload: &[u8]) -> Transmission<CurrentNetwork> {
        Transmission::Transaction(Data::Buffer(Bytes::from(payload.to_vec())))
    }

    /// Builds a batch header declaring the given transmission IDs.
    fn sample_batch_header(
        transmission_ids: &[TransmissionID<CurrentNetwork>],
        rng: &mut TestRng,
    ) -> BatchHeader<CurrentNetwork> {
        let private_key = PrivateKey::<CurrentNetwork>::new(rng).unwrap();
        // Round 1 is the only round that may carry no previous certificate IDs.
        BatchHeader::new(
            &private_key,
            1,
            0,
            Field::rand(rng),
            transmission_ids.iter().copied().collect(),
            Default::default(),
            rng,
        )
        .unwrap()
    }

    #[test]
    fn a_new_service_holds_nothing() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();
        let transmission_id = sample_transmission_id(rng);

        assert!(!service.contains_transmission(transmission_id));
        assert!(service.get_transmission(transmission_id).is_none());
        assert!(service.as_hashmap().is_empty());
    }

    #[test]
    fn an_inserted_transmission_is_retrievable() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();
        let transmission_id = sample_transmission_id(rng);
        let transmission = sample_transmission(b"payload");
        let certificate_id = Field::rand(rng);

        service.insert_transmissions(
            certificate_id,
            indexset![transmission_id],
            Default::default(),
            [(transmission_id, transmission.clone())].into(),
        );

        assert!(service.contains_transmission(transmission_id));
        assert_eq!(service.get_transmission(transmission_id), Some(transmission));
    }

    #[test]
    fn a_transmission_survives_until_its_last_certificate_is_removed() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();
        let transmission_id = sample_transmission_id(rng);
        let transmission = sample_transmission(b"payload");
        let (first, second) = (Field::rand(rng), Field::rand(rng));

        // Two certificates referencing one transmission, as happens when the primary processes
        // batches from several peers that all include the same transaction.
        service.insert_transmissions(
            first,
            indexset![transmission_id],
            Default::default(),
            [(transmission_id, transmission.clone())].into(),
        );
        service.insert_transmissions(second, indexset![transmission_id], Default::default(), Default::default());

        let (_, certificate_ids) = service.as_hashmap().remove(&transmission_id).unwrap();
        assert_eq!(certificate_ids.len(), 2);

        // Dropping one reference must not drop the transmission...
        service.remove_transmissions(&first, &indexset![transmission_id]);
        assert!(service.contains_transmission(transmission_id));

        // ...but dropping the last one must.
        service.remove_transmissions(&second, &indexset![transmission_id]);
        assert!(!service.contains_transmission(transmission_id));
        assert!(service.get_transmission(transmission_id).is_none());
    }

    #[test]
    fn a_stored_transmission_is_not_overwritten_by_a_later_insert() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();
        let transmission_id = sample_transmission_id(rng);
        let original = sample_transmission(b"original");
        let (first, second) = (Field::rand(rng), Field::rand(rng));

        service.insert_transmissions(
            first,
            indexset![transmission_id],
            Default::default(),
            [(transmission_id, original.clone())].into(),
        );
        // A second certificate arrives carrying a different payload under the same ID. The stored
        // transmission wins: the occupied branch only records the new certificate ID.
        service.insert_transmissions(
            second,
            indexset![transmission_id],
            Default::default(),
            [(transmission_id, sample_transmission(b"replacement"))].into(),
        );

        assert_eq!(service.get_transmission(transmission_id), Some(original));
    }

    #[test]
    fn an_aborted_transmission_id_is_contained_but_has_no_payload() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();
        let transmission_id = sample_transmission_id(rng);

        service.insert_transmissions(
            Field::rand(rng),
            Default::default(),
            [transmission_id].into(),
            Default::default(),
        );

        // The asymmetry is deliberate: an aborted ID is known to storage, but there is no
        // transmission to hand back for it.
        assert!(service.contains_transmission(transmission_id));
        assert!(service.get_transmission(transmission_id).is_none());
        assert!(service.as_hashmap().is_empty());
    }

    #[test]
    fn aborted_transmission_ids_are_reference_counted_too() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();
        let transmission_id = sample_transmission_id(rng);
        let (first, second) = (Field::rand(rng), Field::rand(rng));

        for certificate_id in [first, second] {
            service.insert_transmissions(
                certificate_id,
                Default::default(),
                [transmission_id].into(),
                Default::default(),
            );
        }

        service.remove_transmissions(&first, &indexset![transmission_id]);
        assert!(service.contains_transmission(transmission_id));

        service.remove_transmissions(&second, &indexset![transmission_id]);
        assert!(!service.contains_transmission(transmission_id));
    }

    #[test]
    fn removing_an_unrelated_certificate_leaves_the_transmission_in_place() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();
        let transmission_id = sample_transmission_id(rng);
        let certificate_id = Field::rand(rng);

        service.insert_transmissions(
            certificate_id,
            indexset![transmission_id],
            Default::default(),
            [(transmission_id, sample_transmission(b"payload"))].into(),
        );

        // Removing a certificate that never referenced this transmission must not evict it.
        service.remove_transmissions(&Field::rand(rng), &indexset![transmission_id]);

        assert!(service.contains_transmission(transmission_id));
    }

    #[test]
    fn removing_from_an_empty_service_is_a_no_op() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();

        service.remove_transmissions(&Field::rand(rng), &indexset![sample_transmission_id(rng)]);

        assert!(service.as_hashmap().is_empty());
    }

    #[test]
    fn find_missing_transmissions_returns_only_what_storage_lacks() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();
        let (stored_id, missing_id) = (sample_transmission_id(rng), sample_transmission_id(rng));
        let missing = sample_transmission(b"missing");

        service.insert_transmissions(
            Field::rand(rng),
            indexset![stored_id],
            Default::default(),
            [(stored_id, sample_transmission(b"stored"))].into(),
        );

        let batch_header = sample_batch_header(&[stored_id, missing_id], rng);
        let result = service
            .find_missing_transmissions(
                &batch_header,
                [(stored_id, sample_transmission(b"stored")), (missing_id, missing.clone())].into(),
                Default::default(),
            )
            .unwrap();

        // The already-stored one is dropped even though the caller offered it, so the primary does
        // not re-insert a payload it already holds.
        assert_eq!(result.len(), 1);
        assert_eq!(result.get(&missing_id), Some(&missing));
    }

    #[test]
    fn find_missing_transmissions_accepts_an_aborted_declaration_without_returning_it() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();
        let aborted_id = sample_transmission_id(rng);

        let batch_header = sample_batch_header(&[aborted_id], rng);
        let result =
            service.find_missing_transmissions(&batch_header, Default::default(), [aborted_id].into()).unwrap();

        // An aborted declaration is satisfied without a payload, and contributes nothing to fetch.
        assert!(result.is_empty());
    }

    #[test]
    fn find_missing_transmissions_rejects_a_declaration_that_is_neither_provided_nor_aborted() {
        let rng = &mut TestRng::default();
        let service = BFTMemoryService::<CurrentNetwork>::new();
        let undeclared_id = sample_transmission_id(rng);

        let batch_header = sample_batch_header(&[undeclared_id], rng);

        // This is the check that stops a peer from having its certificate accepted while
        // withholding the transmissions the certificate commits to.
        assert!(service.find_missing_transmissions(&batch_header, Default::default(), Default::default()).is_err());
    }

    /// Documents a live deadlock rather than asserting correct behavior; ignored so it cannot hang
    /// CI. Run it explicitly with `--ignored` to reproduce.
    ///
    /// When `insert_transmissions` reaches a vacant entry for a transmission ID that was neither
    /// provided in `missing_transmissions` nor aborted, it calls `self.contains_transmission(..)`
    /// while already holding the write guard on `self.transmissions`. That method takes a read
    /// guard on the same lock, which `parking_lot::RwLock` does not grant re-entrantly, so the
    /// thread parks forever.
    ///
    /// The production path reaches this branch through the gap between `find_missing_transmissions`
    /// and `insert_transmissions`: the two take separate locks, so a concurrent
    /// `remove_transmissions` can evict an entry that the first call saw and therefore skipped.
    /// Only `BFTMemoryService` is affected. `BFTPersistentStorage` has the same shape but its
    /// `contains_transmission` reads RocksDB rather than the lock it is holding.
    #[test]
    #[ignore = "reproduces a deadlock in insert_transmissions; see the PR description"]
    fn insert_transmissions_deadlocks_on_an_unprovided_transmission() {
        let (sender, receiver) = mpsc::channel();

        // Run the call on its own thread so a hang is reported as a failure, not as a hung test
        // binary. The thread is leaked if it parks, which is the behavior under test.
        std::thread::spawn(move || {
            let rng = &mut TestRng::default();
            let service = BFTMemoryService::<CurrentNetwork>::new();
            let transmission_id = sample_transmission_id(rng);

            service.insert_transmissions(
                Field::rand(rng),
                indexset![transmission_id],
                Default::default(),
                Default::default(),
            );

            let _ = sender.send(());
        });

        assert!(
            receiver.recv_timeout(Duration::from_secs(10)).is_ok(),
            "insert_transmissions did not return: it re-entered its own lock",
        );
    }
}
