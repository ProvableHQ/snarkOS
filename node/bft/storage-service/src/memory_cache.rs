// Copyright (C) 2019-2023 Aleo Systems Inc.
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
    prelude::{bail, Field, Network, Result},
};

use indexmap::{indexset, IndexSet};
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use aleo_std::StorageMode;
use lru::LruCache;

/// A BFT in-memory storage service.
#[derive(Debug)]
pub struct BFTCacheService<N: Network> {
    /// The LRU cache for `transmission ID` to `(transmission, certificate IDs)` entries.
    transmissions: RwLock<LruCache<TransmissionID<N>, (Transmission<N>, IndexSet<Field<N>>)>>,
    /// The LRU cache for `aborted transmission ID` to `certificate IDs` entries.
    aborted_transmission_ids: RwLock<LruCache<TransmissionID<N>, IndexSet<Field<N>>>>,
}

impl<N: Network> BFTCacheService<N> {
    /// Initializes a new BFT in-memory storage service with an LRU cache.
    pub fn new(transmission_capacity: usize, aborted_capacity: usize) -> Self {
        Self {
            transmissions: RwLock::new(LruCache::new(transmission_capacity)),
            aborted_transmission_ids: RwLock::new(LruCache::new(aborted_capacity)),
        }
    }

    /// Opens a BFT in-memory storage service based on the given mode.
    pub fn open(storage_mode: StorageMode) -> Result<Self> {
        // Determine the capacities based on the storage mode.
        let (transmission_capacity, aborted_capacity) = match storage_mode {
            StorageMode::Production => (1000, 500),
            StorageMode::Development(_) => (500, 250),
            StorageMode::Custom(_) => (2000, 1000),
        };

        // Initialize the service with the determined capacities.
        Ok(Self::new(transmission_capacity, aborted_capacity))
    }
}

// Implement the StorageService<N> trait for BFTCacheService<N>
impl<N: Network> StorageService<N> for BFTCacheService<N> {
    /// Returns `true` if the storage contains the specified `transmission ID`.
    fn contains_transmission(&self, transmission_id: TransmissionID<N>) -> bool {
        let transmissions = self.transmissions.read();
        let aborted_transmissions = self.aborted_transmission_ids.read();
        transmissions.contains(&transmission_id) || aborted_transmissions.contains(&transmission_id)
    }

    /// Returns the transmission for the given `transmission ID`.
    /// If the transmission does not exist in storage, `None` is returned.
    fn get_transmission(&self, transmission_id: TransmissionID<N>) -> Option<Transmission<N>> {
        let mut transmissions = self.transmissions.write();
        if let Some((transmission, _)) = transmissions.get_mut(&transmission_id) {
            Some(transmission.clone())
        } else {
            None
        }
    }

    /// Returns the missing transmissions in storage from the given transmissions.
    fn find_missing_transmissions(
        &self,
        batch_header: &BatchHeader<N>,
        mut transmissions: HashMap<TransmissionID<N>, Transmission<N>>,
        aborted_transmissions: HashSet<TransmissionID<N>>,
    ) -> Result<HashMap<TransmissionID<N>, Transmission<N>>> {
        let mut missing_transmissions = HashMap::new();
        let known_transmissions = self.transmissions.read();
        for transmission_id in batch_header.transmission_ids() {
            if !known_transmissions.contains(&*transmission_id) {
                match transmissions.remove(&transmission_id) {
                    Some(transmission) => {
                        missing_transmissions.insert(transmission_id.clone(), transmission);
                    }
                    None => {
                        if !aborted_transmissions.contains(&transmission_id) {
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
        let mut transmissions = self.transmissions.write();
        let mut aborted_transmission_ids_lock = self.aborted_transmission_ids.write();
        'outer: for transmission_id in transmission_ids {
            match transmissions.get_mut(&transmission_id) {
                Some((_, certificate_ids)) => {
                    certificate_ids.insert(certificate_id);
                }
                None => {
                    if let Some(transmission) = missing_transmissions.remove(&transmission_id) {
                        let certificate_ids = indexset! { certificate_id };
                        transmissions.put(transmission_id, (transmission, certificate_ids));
                    } else if !aborted_transmission_ids.contains(&transmission_id) {
                        continue 'outer;
                    }
                }
            }
        }
        for aborted_transmission_id in aborted_transmission_ids {
            match aborted_transmission_ids_lock.get_mut(&aborted_transmission_id) {
                Some(certificate_ids) => {
                    certificate_ids.insert(certificate_id);
                }
                None => {
                    let certificate_ids = indexset! { certificate_id };
                    aborted_transmission_ids_lock.put(aborted_transmission_id, certificate_ids);
                }
            }
        }
    }

    /// Removes the certificate ID for the transmissions from storage.
    fn remove_transmissions(&self, certificate_id: &Field<N>, transmission_ids: &IndexSet<TransmissionID<N>>) {
        let mut transmissions = self.transmissions.write();
        let mut aborted_transmission_ids = self.aborted_transmission_ids.write();
        for transmission_id in transmission_ids {
            if let Some((_, certificate_ids)) = transmissions.get_mut(transmission_id) {
                certificate_ids.swap_remove(certificate_id);
                if certificate_ids.is_empty() {
                    transmissions.pop(transmission_id);
                }
            }
            if let Some(certificate_ids) = aborted_transmission_ids.get_mut(transmission_id) {
                certificate_ids.swap_remove(certificate_id);
                if certificate_ids.is_empty() {
                    aborted_transmission_ids.pop(transmission_id);
                }
            }
        }
    }

    /// Returns a HashMap over the `(transmission ID, (transmission, certificate IDs))` entries.
    #[cfg(any(test, feature = "test"))]
    fn as_hashmap(&self) -> HashMap<TransmissionID<N>, (Transmission<N>, IndexSet<Field<N>>)> {
        self.transmissions.read().clone().into_iter().collect()
    }
}