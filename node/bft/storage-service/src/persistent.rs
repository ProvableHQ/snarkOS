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
    ledger::{
        committee::Committee,
        narwhal::{BatchHeader, Transmission, TransmissionID},
        store::helpers::{
            Map,
            MapRead,
            rocksdb::{
                DataMap,
                internal::{self, BFTMap, Database, MapID},
            },
        },
    },
    prelude::{Field, Network, Result, bail},
};

use aleo_std::StorageMode;
use anyhow::anyhow;
use indexmap::{IndexSet, indexset};
#[cfg(feature = "locktick")]
use locktick::parking_lot::Mutex;
use lru::LruCache;
#[cfg(not(feature = "locktick"))]
use parking_lot::Mutex;
use std::{
    borrow::Cow,
    collections::{HashMap, HashSet},
    num::NonZeroUsize,
};
use tracing::error;

/// A BFT persistent storage service.
#[derive(Debug)]
pub struct BFTPersistentStorage<N: Network> {
    /// The map of `transmission ID` to `(transmission, certificate IDs)` entries.
    transmissions: DataMap<TransmissionID<N>, (Transmission<N>, IndexSet<Field<N>>)>,
    /// The map of `aborted transmission ID` to `certificate IDs` entries.
    aborted_transmission_ids: DataMap<TransmissionID<N>, IndexSet<Field<N>>>,
    /// The LRU cache for `transmission ID` to `(transmission, certificate IDs)` entries that are part of the persistent storage.
    ///
    /// This mutex also serializes the read-modify-write updates in `insert_transmissions` and
    /// `remove_transmissions`, which would otherwise lose updates when run concurrently: writers
    /// hold the guard for the full update. This also guarantees that `get_transmission`'s cache
    /// probe can never observe a removal that has been applied to storage but not yet to the cache.
    cache_transmissions: Mutex<LruCache<TransmissionID<N>, (Transmission<N>, IndexSet<Field<N>>)>>,
}

impl<N: Network> BFTPersistentStorage<N> {
    /// Initializes a new BFT persistent storage service.
    pub fn open(storage_mode: StorageMode) -> Result<Self> {
        let max_committee_size = Committee::<N>::max_committee_size();
        let capacity =
            NonZeroUsize::new((max_committee_size as usize) * (BatchHeader::<N>::MAX_TRANSMISSIONS_PER_BATCH) * 2)
                .ok_or_else(|| anyhow!("Could not construct NonZeroUsize"))?;

        Ok(Self {
            transmissions: internal::RocksDB::open_map(N::ID, storage_mode.clone(), MapID::BFT(BFTMap::Transmissions))?,
            aborted_transmission_ids: internal::RocksDB::open_map(
                N::ID,
                storage_mode,
                MapID::BFT(BFTMap::AbortedTransmissionIDs),
            )?,
            cache_transmissions: Mutex::new(LruCache::new(capacity)),
        })
    }
}

impl<N: Network> StorageService<N> for BFTPersistentStorage<N> {
    /// Returns `true` if the storage contains the specified `transmission ID`.
    fn contains_transmission(&self, transmission_id: TransmissionID<N>) -> bool {
        // Check if the transmission ID exists in storage.
        match self.transmissions.contains_key_confirmed(&transmission_id) {
            Ok(true) => return true,
            Ok(false) => (),
            Err(error) => error!("Failed to check if transmission ID exists in confirmed storage - {error}"),
        }
        // Check if the transmission ID is in aborted storage.
        match self.aborted_transmission_ids.contains_key_confirmed(&transmission_id) {
            Ok(result) => result,
            Err(error) => {
                error!("Failed to check if aborted transmission ID exists in storage - {error}");
                false
            }
        }
    }

    /// Returns `true` if the storage holds the transmission for the specified `transmission ID`.
    fn contains_retrievable_transmission(&self, transmission_id: TransmissionID<N>) -> bool {
        // Check the cache first: it only ever holds transmissions that are also in persistent
        // storage, so a hit answers this without a storage read. This also keeps the check in
        // agreement with `get_transmission`, which probes the cache first as well.
        if self.cache_transmissions.lock().contains(&transmission_id) {
            return true;
        }
        match self.transmissions.contains_key_confirmed(&transmission_id) {
            Ok(result) => result,
            Err(error) => {
                error!("Failed to check if transmission ID exists in confirmed storage - {error}");
                false
            }
        }
    }

    /// Returns `true` if the specified `transmission ID` is recorded as aborted.
    fn contains_aborted_transmission(&self, transmission_id: TransmissionID<N>) -> bool {
        match self.aborted_transmission_ids.contains_key_confirmed(&transmission_id) {
            Ok(result) => result,
            Err(error) => {
                error!("Failed to check if aborted transmission ID exists in storage - {error}");
                false
            }
        }
    }

    /// Returns the transmission for the given `transmission ID`.
    /// If the transmission ID does not exist in storage, `None` is returned.
    fn get_transmission(&self, transmission_id: TransmissionID<N>) -> Option<Transmission<N>> {
        // Try to get the transmission from the cache first.
        if let Some((transmission, _)) = self.cache_transmissions.lock().get_mut(&transmission_id) {
            return Some(transmission.clone());
        }

        // If not found in cache, check persistent storage.
        match self.transmissions.get_confirmed(&transmission_id) {
            Ok(Some(Cow::Owned((transmission, _)))) => Some(transmission),
            Ok(Some(Cow::Borrowed((transmission, _)))) => Some(transmission.clone()),
            Ok(None) => None,
            Err(error) => {
                error!("Failed to get transmission from storage - {error}");
                None
            }
        }
    }

    /// Takes a certificate and its transmissions, and returns the subset of transmissions that
    /// did not yet exists in the storage.
    fn find_missing_transmissions(
        &self,
        batch_header: &BatchHeader<N>,
        mut transmissions: HashMap<TransmissionID<N>, Transmission<N>>,
        aborted_transmissions: HashSet<TransmissionID<N>>,
    ) -> Result<HashMap<TransmissionID<N>, Transmission<N>>> {
        // Initialize a list for the missing transmissions from storage.
        let mut missing_transmissions = HashMap::new();
        // Ensure the declared transmission IDs are all present in storage or the given transmissions map.
        for transmission_id in batch_header.transmission_ids() {
            // If the transmission ID does not exist, ensure it was provided by the caller or aborted.
            if !self.contains_transmission(*transmission_id) {
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
        // Hold the cache lock for the entire update, to serialize the read-modify-write updates
        // below against concurrent inserts and removals, and to keep the cache and the persistent
        // storage consistent with one another from the perspective of concurrent readers.
        let mut cache = self.cache_transmissions.lock();

        // First, handle the non-aborted transmissions.
        'outer: for transmission_id in transmission_ids {
            // Try to fetch from the persistent storage.
            let (transmission, certificate_ids) = match self.transmissions.get_confirmed(&transmission_id) {
                Ok(Some(entry)) => {
                    // The transmission exists in storage; update its certificate IDs.
                    let (transmission, mut certificate_ids) = (*entry).clone();
                    certificate_ids.insert(certificate_id);
                    (transmission, certificate_ids)
                }
                Ok(None) => {
                    // The transmission is missing from persistent storage.
                    // Check if it exists in the `missing_transmissions` map provided.
                    let Some(transmission) = missing_transmissions.remove(&transmission_id) else {
                        // This is the branch where persistent storage does not hold the
                        // transmission, so asking whether it is aborted is the same question as
                        // asking whether storage knows the ID at all - without repeating the
                        // lookup that just came back empty.
                        if !aborted_transmission_ids.contains(&transmission_id)
                            && !self.contains_aborted_transmission(transmission_id)
                        {
                            error!("Failed to provide a missing transmission {transmission_id}");
                        }
                        continue 'outer;
                    };
                    // Prepare the set of certificate IDs.
                    let certificate_ids = indexset! { certificate_id };
                    (transmission, certificate_ids)
                }
                Err(e) => {
                    // Handle any errors during the retrieval.
                    error!("Failed to process the 'insert' for transmission {transmission_id} into storage - {e}");
                    continue;
                }
            };
            // Insert the transmission into persistent storage.
            match self.transmissions.insert(transmission_id, (transmission.clone(), certificate_ids.clone())) {
                // Insert the transmission into the cache.
                Ok(()) => {
                    cache.put(transmission_id, (transmission, certificate_ids));
                }
                // On failure, evict the cache entry, to keep the cache consistent with the persistent storage.
                Err(e) => {
                    error!("Failed to insert transmission {transmission_id} into storage - {e}");
                    cache.pop(&transmission_id);
                }
            }
        }

        // Next, handle the aborted transmission IDs.
        for aborted_transmission_id in aborted_transmission_ids {
            let certificate_ids = match self.aborted_transmission_ids.get_confirmed(&aborted_transmission_id) {
                Ok(Some(entry)) => {
                    let mut certificate_ids = (*entry).clone();
                    // Insert the certificate ID into the set.
                    certificate_ids.insert(certificate_id);
                    certificate_ids
                }
                Ok(None) => indexset! { certificate_id },
                Err(e) => {
                    error!(
                        "Failed to process the 'insert' for aborted transmission ID {aborted_transmission_id} into storage - {e}"
                    );
                    continue;
                }
            };
            // Insert the certificate IDs into the persistent storage.
            if let Err(e) = self.aborted_transmission_ids.insert(aborted_transmission_id, certificate_ids) {
                error!("Failed to insert aborted transmission ID {aborted_transmission_id} into storage - {e}");
            }
        }
    }

    /// Removes the certificate ID for the transmissions from storage.
    ///
    /// If the transmission no longer references any certificate IDs, the entry is removed from storage.
    fn remove_transmissions(&self, certificate_id: &Field<N>, transmission_ids: &IndexSet<TransmissionID<N>>) {
        // Hold the cache lock for the entire update, to serialize the read-modify-write updates
        // below against concurrent inserts and removals, and to keep the cache and the persistent
        // storage consistent with one another from the perspective of concurrent readers.
        let mut cache = self.cache_transmissions.lock();

        // If this is the last certificate ID for the transmission ID, remove the transmission.
        for transmission_id in transmission_ids {
            // Retrieve the transmission entry.
            match self.transmissions.get_confirmed(transmission_id) {
                Ok(Some(entry)) => {
                    let (transmission, mut certificate_ids) = (*entry).clone();
                    // Insert the certificate ID into the set.
                    certificate_ids.swap_remove(certificate_id);
                    // If there are no more certificate IDs for the transmission ID, remove the transmission.
                    if certificate_ids.is_empty() {
                        // Remove the transmission entry.
                        if let Err(e) = self.transmissions.remove(transmission_id) {
                            error!("Failed to remove transmission {transmission_id} (now empty) from storage - {e}");
                        }
                        // Remove the transmission from the cache.
                        cache.pop(transmission_id);
                    }
                    // Otherwise, update the transmission entry.
                    else {
                        // Update the transmission entry.
                        match self
                            .transmissions
                            .insert(*transmission_id, (transmission.clone(), certificate_ids.clone()))
                        {
                            // Update the transmission in the cache.
                            Ok(()) => {
                                cache.put(*transmission_id, (transmission, certificate_ids));
                            }
                            // On failure, evict the cache entry, to keep the cache consistent with the persistent storage.
                            Err(e) => {
                                error!(
                                    "Failed to remove transmission {transmission_id} for certificate {certificate_id} from storage - {e}"
                                );
                                cache.pop(transmission_id);
                            }
                        }
                    }
                }
                Ok(None) => { /* no-op */ }
                Err(e) => {
                    error!("Failed to process the 'remove' for transmission {transmission_id} from storage - {e}");
                }
            }
            // Retrieve the aborted transmission ID entry.
            // Note: while the cache is not consulted here, the lock is intentionally still held, to
            // serialize the read-modify-write updates below against concurrent inserts and removals.
            match self.aborted_transmission_ids.get_confirmed(transmission_id) {
                Ok(Some(entry)) => {
                    let mut certificate_ids = (*entry).clone();
                    // Insert the certificate ID into the set.
                    certificate_ids.swap_remove(certificate_id);
                    // If there are no more certificate IDs for the transmission ID, remove the transmission.
                    if certificate_ids.is_empty() {
                        // Remove the transmission entry.
                        if let Err(e) = self.aborted_transmission_ids.remove(transmission_id) {
                            error!(
                                "Failed to remove aborted transmission ID {transmission_id} (now empty) from storage - {e}"
                            );
                        }
                    }
                    // Otherwise, update the transmission entry.
                    else {
                        // Update the transmission entry.
                        if let Err(e) = self.aborted_transmission_ids.insert(*transmission_id, certificate_ids) {
                            error!(
                                "Failed to remove aborted transmission ID {transmission_id} for certificate {certificate_id} from storage - {e}"
                            );
                        }
                    }
                }
                Ok(None) => { /* no-op */ }
                Err(e) => {
                    error!(
                        "Failed to process the 'remove' for aborted transmission ID {transmission_id} from storage - {e}"
                    );
                }
            }
        }
    }

    /// Returns a HashMap over the `(transmission ID, (transmission, certificate IDs))` entries.
    #[cfg(any(test, feature = "test"))]
    fn as_hashmap(&self) -> HashMap<TransmissionID<N>, (Transmission<N>, IndexSet<Field<N>>)> {
        self.transmissions.iter_confirmed().map(|(k, v)| (k.into_owned(), (*v).clone())).collect()
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
    use std::sync::{Arc, Barrier};

    type CurrentNetwork = MainnetV0;

    // Note: these mirror the helpers in the in-memory service's tests; they are candidates for a
    // shared harness once both services are exercised against the same expectations.

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

    /// Every certificate that references a transmission must contribute its certificate ID to the
    /// transmission's reference set, even when the insertions happen concurrently — as they do when
    /// the primary processes batch certificates from multiple peers that include the same
    /// transaction or solution.
    #[test]
    fn test_concurrent_insert_transmissions_do_not_lose_certificate_ids() {
        const NUM_CERTIFICATES: usize = 8;
        const NUM_ITERATIONS: usize = 20;

        let rng = &mut TestRng::default();
        let storage = Arc::new(BFTPersistentStorage::<CurrentNetwork>::open(StorageMode::new_test(None)).unwrap());

        for _ in 0..NUM_ITERATIONS {
            // One transmission, referenced by many certificates.
            let transmission_id = TransmissionID::Transaction(
                <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
                <CurrentNetwork as Network>::TransmissionChecksum::from(rng.random::<u128>()),
            );
            let transmission = Transmission::Transaction(Data::Buffer(Bytes::from(vec![0u8; 32])));
            let certificate_ids: Vec<Field<CurrentNetwork>> = (0..NUM_CERTIFICATES).map(|_| Field::rand(rng)).collect();

            // Insert the certificate IDs concurrently, all referencing the same transmission.
            let barrier = Arc::new(Barrier::new(NUM_CERTIFICATES));
            let handles: Vec<_> = certificate_ids
                .iter()
                .map(|certificate_id| {
                    let storage = storage.clone();
                    let barrier = barrier.clone();
                    let certificate_id = *certificate_id;
                    let transmission = transmission.clone();
                    std::thread::spawn(move || {
                        barrier.wait();
                        storage.insert_transmissions(
                            certificate_id,
                            indexset![transmission_id],
                            Default::default(),
                            [(transmission_id, transmission)].into(),
                        );
                    })
                })
                .collect();
            for handle in handles {
                handle.join().unwrap();
            }

            // Every certificate ID must be present in the transmission's reference set.
            let entries = storage.as_hashmap();
            let (_, stored_certificate_ids) =
                entries.get(&transmission_id).expect("the transmission must exist in storage");
            assert_eq!(
                stored_certificate_ids.len(),
                NUM_CERTIFICATES,
                "lost {} of {NUM_CERTIFICATES} certificate references",
                NUM_CERTIFICATES - stored_certificate_ids.len(),
            );
            for certificate_id in &certificate_ids {
                assert!(stored_certificate_ids.contains(certificate_id));
            }
        }
    }

    /// Once the last referencing certificate is removed, the transmission must no longer be
    /// returned — including via the LRU cache that `get_transmission` consults first.
    #[test]
    fn test_remove_transmissions_purges_the_cache() {
        let rng = &mut TestRng::default();
        let storage = BFTPersistentStorage::<CurrentNetwork>::open(StorageMode::new_test(None)).unwrap();

        let transmission_id = TransmissionID::Transaction(
            <CurrentNetwork as Network>::TransactionID::from(Field::rand(rng)),
            <CurrentNetwork as Network>::TransmissionChecksum::from(rng.random::<u128>()),
        );
        let transmission = Transmission::Transaction(Data::Buffer(Bytes::from(vec![0u8; 32])));
        let certificate_id = Field::rand(rng);

        // Insert the transmission with a single referencing certificate, then remove the certificate.
        storage.insert_transmissions(
            certificate_id,
            indexset![transmission_id],
            Default::default(),
            [(transmission_id, transmission)].into(),
        );
        storage.remove_transmissions(&certificate_id, &indexset![transmission_id]);

        // The transmission must be gone from both the persistent storage and the cache.
        assert!(!storage.as_hashmap().contains_key(&transmission_id));
        assert!(storage.get_transmission(transmission_id).is_none());
        assert!(!storage.contains_transmission(transmission_id));
    }

    /// The three containment queries must answer three different questions: a stored transmission
    /// is retrievable, an aborted ID is known but has nothing to return, and both are contained.
    #[test]
    fn test_the_containment_queries_distinguish_stored_from_aborted_ids() {
        let rng = &mut TestRng::default();
        let storage = BFTPersistentStorage::<CurrentNetwork>::open(StorageMode::new_test(None)).unwrap();
        let (stored_id, aborted_id, unknown_id) =
            (sample_transmission_id(rng), sample_transmission_id(rng), sample_transmission_id(rng));

        storage.insert_transmissions(
            Field::rand(rng),
            indexset![stored_id],
            [aborted_id].into(),
            [(stored_id, sample_transmission(b"payload"))].into(),
        );

        // The stored transmission is contained and retrievable, but not aborted.
        assert!(storage.contains_transmission(stored_id));
        assert!(storage.contains_retrievable_transmission(stored_id));
        assert!(!storage.contains_aborted_transmission(stored_id));

        // The aborted ID is contained and aborted, but not retrievable.
        assert!(storage.contains_transmission(aborted_id));
        assert!(!storage.contains_retrievable_transmission(aborted_id));
        assert!(storage.contains_aborted_transmission(aborted_id));

        // An unknown ID is none of the three.
        assert!(!storage.contains_transmission(unknown_id));
        assert!(!storage.contains_retrievable_transmission(unknown_id));
        assert!(!storage.contains_aborted_transmission(unknown_id));
    }

    /// The cache `contains_retrievable_transmission` consults first must not outlive the storage
    /// entry it stands for, or the query would report a removed transmission as retrievable.
    #[test]
    fn test_contains_retrievable_transmission_follows_a_removal_through_the_cache() {
        let rng = &mut TestRng::default();
        let storage = BFTPersistentStorage::<CurrentNetwork>::open(StorageMode::new_test(None)).unwrap();
        let transmission_id = sample_transmission_id(rng);
        let certificate_id = Field::rand(rng);

        storage.insert_transmissions(
            certificate_id,
            indexset![transmission_id],
            Default::default(),
            [(transmission_id, sample_transmission(b"payload"))].into(),
        );
        assert!(storage.contains_retrievable_transmission(transmission_id));

        storage.remove_transmissions(&certificate_id, &indexset![transmission_id]);
        assert!(!storage.contains_retrievable_transmission(transmission_id));
    }

    /// A transmission whose ID an earlier certificate recorded as aborted must still be persisted
    /// when a later certificate provides its bytes.
    ///
    /// An aborted entry records the ID and nothing else, so `get_transmission` has nothing to hand
    /// back for it. `find_missing_transmissions` nevertheless decides whether the bytes are needed
    /// with `contains_transmission`, which is also true for an aborted ID - so the bytes are
    /// discarded, and the certificate that declares the ID is accepted into storage while the
    /// transmission it commits to can never be materialized.
    ///
    /// Note the in-memory service does not share this behavior: it consults only its concrete map,
    /// collects the bytes, and stores them.
    #[test]
    fn test_provided_bytes_are_kept_for_a_previously_aborted_id() {
        let rng = &mut TestRng::default();
        let storage = BFTPersistentStorage::<CurrentNetwork>::open(StorageMode::new_test(None)).unwrap();
        let transmission_id = sample_transmission_id(rng);
        let transmission = sample_transmission(b"payload");

        // An earlier certificate recorded the ID as aborted, so storage contains it without bytes.
        storage.insert_transmissions(
            Field::rand(rng),
            Default::default(),
            [transmission_id].into(),
            Default::default(),
        );
        assert!(storage.contains_transmission(transmission_id));
        assert!(storage.get_transmission(transmission_id).is_none());

        // A later certificate declares the same ID and provides its bytes.
        let batch_header = sample_batch_header(&[transmission_id], rng);
        let missing_transmissions = storage
            .find_missing_transmissions(
                &batch_header,
                [(transmission_id, transmission.clone())].into(),
                Default::default(),
            )
            .unwrap();

        // The bytes have to be collected...
        assert_eq!(
            missing_transmissions.get(&transmission_id),
            Some(&transmission),
            "the provided bytes were discarded, so they will never reach storage"
        );

        // ...and, once inserted, they have to be retrievable, or the certificate that declares this
        // transmission ID cannot be committed.
        storage.insert_transmissions(
            Field::rand(rng),
            indexset![transmission_id],
            Default::default(),
            missing_transmissions,
        );
        assert_eq!(storage.get_transmission(transmission_id), Some(transmission));
    }
}
