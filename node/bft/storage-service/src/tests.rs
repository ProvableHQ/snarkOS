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

//! Shared tests for the `StorageService` trait, exercised against every implementation to keep
//! their semantics aligned.

use crate::{StorageService, memory::BFTMemoryService, persistent::BFTPersistentStorage};
use snarkvm::{
    console::network::MainnetV0,
    ledger::narwhal::{BatchHeader, Transmission, TransmissionID, batch_header, transmission},
    prelude::{Field, TestRng, Uniform},
};

use aleo_std::StorageMode;
use std::collections::{HashMap, HashSet};

type CurrentNetwork = MainnetV0;

/// Samples a batch header along with a transmission for each of its transmission IDs.
fn sample_header_and_transmissions(
    rng: &mut TestRng,
) -> (BatchHeader<CurrentNetwork>, HashMap<TransmissionID<CurrentNetwork>, Transmission<CurrentNetwork>>) {
    let batch_header = batch_header::test_helpers::sample_batch_header(rng);
    let transmissions: HashMap<_, _> = batch_header
        .transmission_ids()
        .iter()
        .copied()
        .zip(transmission::test_helpers::sample_transmissions(rng))
        .collect();
    assert_eq!(transmissions.len(), batch_header.transmission_ids().len());
    (batch_header, transmissions)
}

fn check_provided_bytes_collected_for_previously_aborted_ids(storage: &impl StorageService<CurrentNetwork>) {
    let rng = &mut TestRng::default();
    let (batch_header, transmissions) = sample_header_and_transmissions(rng);

    // Record all of the transmission IDs as aborted, without any transmissions.
    storage.insert_transmissions(
        Field::rand(rng),
        batch_header.transmission_ids().clone(),
        batch_header.transmission_ids().iter().copied().collect(),
        HashMap::new(),
    );
    // Sanity check - the aborted IDs are "contained", but hold no retrievable transmission.
    for transmission_id in batch_header.transmission_ids() {
        assert!(storage.contains_transmission(*transmission_id));
        assert!(storage.get_transmission(*transmission_id).is_none());
    }

    // The provided transmissions must be collected as missing, despite the IDs being aborted,
    // so that they get persisted and remain retrievable at commit time.
    let missing = storage.find_missing_transmissions(&batch_header, transmissions.clone(), HashSet::new()).unwrap();
    assert_eq!(missing, transmissions);
}

fn check_stored_transmissions_are_not_missing(storage: &impl StorageService<CurrentNetwork>) {
    let rng = &mut TestRng::default();
    let (batch_header, transmissions) = sample_header_and_transmissions(rng);

    // Store the transmissions concretely.
    storage.insert_transmissions(
        Field::rand(rng),
        batch_header.transmission_ids().clone(),
        HashSet::new(),
        transmissions.clone(),
    );
    // Sanity check - the transmissions are retrievable.
    for transmission_id in batch_header.transmission_ids() {
        assert!(storage.get_transmission(*transmission_id).is_some());
    }

    // Stored transmissions are not reported as missing, whether provided again or not.
    let missing = storage.find_missing_transmissions(&batch_header, transmissions.clone(), HashSet::new()).unwrap();
    assert!(missing.is_empty());
    let missing = storage.find_missing_transmissions(&batch_header, HashMap::new(), HashSet::new()).unwrap();
    assert!(missing.is_empty());
}

fn check_aborted_ids_declared_by_caller_are_allowed_without_bytes(storage: &impl StorageService<CurrentNetwork>) {
    let rng = &mut TestRng::default();
    let (batch_header, _) = sample_header_and_transmissions(rng);

    // Declaring all of the transmission IDs as aborted requires no transmissions.
    let aborted = batch_header.transmission_ids().iter().copied().collect();
    let missing = storage.find_missing_transmissions(&batch_header, HashMap::new(), aborted).unwrap();
    assert!(missing.is_empty());
}

fn check_previously_aborted_ids_without_bytes_or_declaration_error(storage: &impl StorageService<CurrentNetwork>) {
    let rng = &mut TestRng::default();
    let (batch_header, _) = sample_header_and_transmissions(rng);

    // Record all of the transmission IDs as aborted, without any transmissions.
    storage.insert_transmissions(
        Field::rand(rng),
        batch_header.transmission_ids().clone(),
        batch_header.transmission_ids().iter().copied().collect(),
        HashMap::new(),
    );

    // An aborted ID in storage does not satisfy the check by itself - the caller must
    // either provide the transmission or declare the ID as aborted.
    assert!(storage.find_missing_transmissions(&batch_header, HashMap::new(), HashSet::new()).is_err());
}

fn check_unknown_unprovided_transmission_errors(storage: &impl StorageService<CurrentNetwork>) {
    let rng = &mut TestRng::default();
    let (batch_header, _) = sample_header_and_transmissions(rng);

    // The transmission IDs are unknown to storage, not provided, and not declared as aborted.
    assert!(storage.find_missing_transmissions(&batch_header, HashMap::new(), HashSet::new()).is_err());
}

/// Instantiates every shared `StorageService` test against the given storage backend.
macro_rules! storage_service_tests {
    ($module:ident, $storage:expr) => {
        mod $module {
            use super::*;

            #[test]
            fn test_provided_bytes_collected_for_previously_aborted_ids() {
                check_provided_bytes_collected_for_previously_aborted_ids(&$storage);
            }

            #[test]
            fn test_stored_transmissions_are_not_missing() {
                check_stored_transmissions_are_not_missing(&$storage);
            }

            #[test]
            fn test_aborted_ids_declared_by_caller_are_allowed_without_bytes() {
                check_aborted_ids_declared_by_caller_are_allowed_without_bytes(&$storage);
            }

            #[test]
            fn test_previously_aborted_ids_without_bytes_or_declaration_error() {
                check_previously_aborted_ids_without_bytes_or_declaration_error(&$storage);
            }

            #[test]
            fn test_unknown_unprovided_transmission_errors() {
                check_unknown_unprovided_transmission_errors(&$storage);
            }
        }
    };
}

storage_service_tests!(memory, BFTMemoryService::<CurrentNetwork>::new());
storage_service_tests!(persistent, BFTPersistentStorage::<CurrentNetwork>::open(StorageMode::new_test(None)).unwrap());
