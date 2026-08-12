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

use snarkvm::{
    ledger::narwhal::{BatchHeader, Transmission, TransmissionID},
    prelude::{Field, Network, Result, bail},
};

use indexmap::IndexSet;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
};

pub trait StorageService<N: Network>: Debug + Send + Sync {
    /// Returns `true` if the storage contains the specified `transmission ID`.
    ///
    /// Note: this is also `true` for transmission IDs only recorded as aborted, for which no
    /// transmission is retrievable; see [`Self::contains_stored_transmission`].
    fn contains_transmission(&self, transmission_id: TransmissionID<N>) -> bool;

    /// Returns `true` if the storage holds the actual transmission for the specified
    /// `transmission ID`.
    ///
    /// Unlike [`Self::contains_transmission`], this is `false` for transmission IDs only recorded
    /// as aborted, for which no transmission is retrievable.
    fn contains_stored_transmission(&self, transmission_id: TransmissionID<N>) -> bool;

    /// Returns the transmission for the given `transmission ID`.
    /// If the transmission ID does not exist in storage, `None` is returned.
    fn get_transmission(&self, transmission_id: TransmissionID<N>) -> Option<Transmission<N>>;

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
            // Check only the stored transmissions, not the aborted transmission IDs: if the caller
            // provided a transmission for a previously-aborted ID, it must still be collected here
            // so it is persisted and remains retrievable at commit time.
            // If the transmission ID does not exist, ensure it was provided by the caller or aborted.
            if !self.contains_stored_transmission(*transmission_id) {
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
        missing_transmissions: HashMap<TransmissionID<N>, Transmission<N>>,
    );

    /// Removes the certificate ID for the transmissions from storage.
    ///
    /// If the transmission no longer references any certificate IDs, the entry is removed from storage.
    fn remove_transmissions(&self, certificate_id: &Field<N>, transmission_ids: &IndexSet<TransmissionID<N>>);

    /// Returns a HashMap over the `(transmission ID, (transmission, certificate IDs))` entries.
    #[cfg(any(test, feature = "test"))]
    fn as_hashmap(&self) -> HashMap<TransmissionID<N>, (Transmission<N>, IndexSet<Field<N>>)>;
}
