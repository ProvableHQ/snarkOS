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

//! The expectations that every [`StorageService`] implementation has to meet.
//!
//! These are the queries whose answers the rest of the BFT reasons about, and
//! `find_missing_transmissions` is now shared by every implementation, so they are asserted here
//! once and instantiated per service rather than restated in each one. Behavior specific to a single
//! service - the persistent one's cache, the in-memory one's locking - stays in its own module.

use crate::{StorageService, memory::BFTMemoryService, persistent::BFTPersistentStorage};
use snarkvm::{
    console::network::MainnetV0,
    ledger::narwhal::{BatchHeader, Data, Transmission, TransmissionID},
    prelude::{Field, Network, PrivateKey, Rng, TestRng, Uniform},
};

use aleo_std::StorageMode;
use bytes::Bytes;
use indexmap::indexset;

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

/// Records the given transmission ID as aborted, as an earlier certificate would have.
fn record_as_aborted(
    service: &impl StorageService<CurrentNetwork>,
    transmission_id: TransmissionID<CurrentNetwork>,
    rng: &mut TestRng,
) {
    service.insert_transmissions(Field::rand(rng), Default::default(), [transmission_id].into(), Default::default());
}

/// The three containment queries must answer three different questions: a stored transmission is
/// retrievable, an aborted ID is known but has nothing to return, and both are contained.
fn check_containment_queries_distinguish_stored_from_aborted_ids(service: &impl StorageService<CurrentNetwork>) {
    let rng = &mut TestRng::default();
    let (stored_id, aborted_id, unknown_id) =
        (sample_transmission_id(rng), sample_transmission_id(rng), sample_transmission_id(rng));

    service.insert_transmissions(
        Field::rand(rng),
        indexset![stored_id],
        [aborted_id].into(),
        [(stored_id, sample_transmission(b"payload"))].into(),
    );

    // The stored transmission is contained and retrievable, but not aborted.
    assert!(service.contains_transmission(stored_id));
    assert!(service.contains_retrievable_transmission(stored_id));
    assert!(!service.contains_aborted_transmission(stored_id));

    // The aborted ID is contained and aborted, but not retrievable.
    assert!(service.contains_transmission(aborted_id));
    assert!(!service.contains_retrievable_transmission(aborted_id));
    assert!(service.contains_aborted_transmission(aborted_id));

    // An unknown ID is none of the three.
    assert!(!service.contains_transmission(unknown_id));
    assert!(!service.contains_retrievable_transmission(unknown_id));
    assert!(!service.contains_aborted_transmission(unknown_id));
}

/// Being recorded as aborted and holding a transmission are not mutually exclusive: one certificate
/// can record an ID as aborted while another provides the bytes for it.
///
/// Each entry is reference counted against its own certificates, so removing either certificate has
/// to leave the other entry untouched. Code that means "aborted, with nothing to hand back" has to
/// say so with both queries; `contains_aborted_transmission` alone does not express it.
fn check_an_id_can_be_aborted_and_retrievable_at_once(service: &impl StorageService<CurrentNetwork>) {
    let rng = &mut TestRng::default();
    let transmission = sample_transmission(b"payload");

    // Puts an ID into the both-true state, returning the aborting and the storing certificate ID.
    let record_both_ways = |transmission_id, rng: &mut TestRng| {
        let (aborting, storing) = (Field::rand(rng), Field::rand(rng));
        service.insert_transmissions(aborting, Default::default(), [transmission_id].into(), Default::default());
        service.insert_transmissions(
            storing,
            indexset![transmission_id],
            Default::default(),
            [(transmission_id, transmission.clone())].into(),
        );
        assert!(service.contains_retrievable_transmission(transmission_id));
        assert!(service.contains_aborted_transmission(transmission_id));
        assert_eq!(service.get_transmission(transmission_id), Some(transmission.clone()));
        (aborting, storing)
    };

    // Dropping the certificate that recorded the abort leaves the transmission retrievable.
    let aborted_first = sample_transmission_id(rng);
    let (aborting, _) = record_both_ways(aborted_first, rng);
    service.remove_transmissions(&aborting, &indexset![aborted_first]);
    assert!(service.contains_retrievable_transmission(aborted_first));
    assert!(!service.contains_aborted_transmission(aborted_first));
    assert_eq!(service.get_transmission(aborted_first), Some(transmission.clone()));

    // Dropping the certificate that provided the bytes leaves the aborted entry in place.
    let stored_first = sample_transmission_id(rng);
    let (_, storing) = record_both_ways(stored_first, rng);
    service.remove_transmissions(&storing, &indexset![stored_first]);
    assert!(!service.contains_retrievable_transmission(stored_first));
    assert!(service.contains_aborted_transmission(stored_first));
    assert!(service.contains_transmission(stored_first));
}

/// Only the transmissions that storage lacks are returned, so the primary does not re-insert a
/// payload it already holds.
fn check_find_missing_transmissions_returns_only_what_storage_lacks(service: &impl StorageService<CurrentNetwork>) {
    let rng = &mut TestRng::default();
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

    assert_eq!(result.len(), 1);
    assert_eq!(result.get(&missing_id), Some(&missing));
}

/// An ID the caller declares as aborted is satisfied without a payload, and contributes nothing to
/// fetch.
fn check_find_missing_transmissions_accepts_a_declared_aborted_id(service: &impl StorageService<CurrentNetwork>) {
    let rng = &mut TestRng::default();
    let aborted_id = sample_transmission_id(rng);

    let batch_header = sample_batch_header(&[aborted_id], rng);
    let result = service.find_missing_transmissions(&batch_header, Default::default(), [aborted_id].into()).unwrap();

    assert!(result.is_empty());
}

/// An ID that storage already recorded as aborted needs neither bytes nor a fresh declaration:
/// there is nothing to fetch, because no peer holds a transmission for an aborted ID either.
///
/// Only the caller that has the block can declare the ID itself; a batch proposed or certified by a
/// peer arrives with an empty set of aborted IDs.
fn check_find_missing_transmissions_accepts_an_already_aborted_id_without_bytes(
    service: &impl StorageService<CurrentNetwork>,
) {
    let rng = &mut TestRng::default();
    let aborted_id = sample_transmission_id(rng);

    record_as_aborted(service, aborted_id, rng);

    let batch_header = sample_batch_header(&[aborted_id], rng);
    let result = service.find_missing_transmissions(&batch_header, Default::default(), Default::default()).unwrap();

    assert!(result.is_empty());
}

/// A transmission whose ID an earlier certificate recorded as aborted must still be persisted when a
/// later certificate provides its bytes.
///
/// An aborted entry records the ID and nothing else, so `get_transmission` has nothing to hand back
/// for it. Discarding the bytes would accept the certificate that declares the ID into storage while
/// the transmission it commits to could never be materialized.
fn check_find_missing_transmissions_keeps_provided_bytes_for_an_aborted_id(
    service: &impl StorageService<CurrentNetwork>,
) {
    let rng = &mut TestRng::default();
    let transmission_id = sample_transmission_id(rng);
    let transmission = sample_transmission(b"payload");

    // An earlier certificate recorded the ID as aborted, so storage contains it without bytes.
    record_as_aborted(service, transmission_id, rng);
    assert!(service.contains_transmission(transmission_id));
    assert!(service.get_transmission(transmission_id).is_none());

    // A later certificate declares the same ID and provides its bytes.
    let batch_header = sample_batch_header(&[transmission_id], rng);
    let missing_transmissions = service
        .find_missing_transmissions(&batch_header, [(transmission_id, transmission.clone())].into(), Default::default())
        .unwrap();

    // The bytes have to be collected...
    assert_eq!(
        missing_transmissions.get(&transmission_id),
        Some(&transmission),
        "the provided bytes were discarded, so they will never reach storage"
    );

    // ...and, once inserted, they have to be retrievable, or the certificate that declares this
    // transmission ID cannot be committed.
    service.insert_transmissions(
        Field::rand(rng),
        indexset![transmission_id],
        Default::default(),
        missing_transmissions,
    );
    assert_eq!(service.get_transmission(transmission_id), Some(transmission));
}

/// This is the check that stops a peer from having its certificate accepted while withholding the
/// transmissions the certificate commits to.
fn check_find_missing_transmissions_rejects_an_unprovided_undeclared_id(service: &impl StorageService<CurrentNetwork>) {
    let rng = &mut TestRng::default();
    let undeclared_id = sample_transmission_id(rng);

    let batch_header = sample_batch_header(&[undeclared_id], rng);

    assert!(service.find_missing_transmissions(&batch_header, Default::default(), Default::default()).is_err());
}

/// Instantiates every shared expectation against the given storage service.
macro_rules! storage_service_tests {
    ($module:ident, $service:expr) => {
        mod $module {
            use super::*;

            #[test]
            fn containment_queries_distinguish_stored_from_aborted_ids() {
                check_containment_queries_distinguish_stored_from_aborted_ids(&$service);
            }

            #[test]
            fn an_id_can_be_aborted_and_retrievable_at_once() {
                check_an_id_can_be_aborted_and_retrievable_at_once(&$service);
            }

            #[test]
            fn find_missing_transmissions_returns_only_what_storage_lacks() {
                check_find_missing_transmissions_returns_only_what_storage_lacks(&$service);
            }

            #[test]
            fn find_missing_transmissions_accepts_a_declared_aborted_id() {
                check_find_missing_transmissions_accepts_a_declared_aborted_id(&$service);
            }

            #[test]
            fn find_missing_transmissions_accepts_an_already_aborted_id_without_bytes() {
                check_find_missing_transmissions_accepts_an_already_aborted_id_without_bytes(&$service);
            }

            #[test]
            fn find_missing_transmissions_keeps_provided_bytes_for_an_aborted_id() {
                check_find_missing_transmissions_keeps_provided_bytes_for_an_aborted_id(&$service);
            }

            #[test]
            fn find_missing_transmissions_rejects_an_unprovided_undeclared_id() {
                check_find_missing_transmissions_rejects_an_unprovided_undeclared_id(&$service);
            }
        }
    };
}

storage_service_tests!(memory, BFTMemoryService::<CurrentNetwork>::new());
storage_service_tests!(persistent, BFTPersistentStorage::<CurrentNetwork>::open(StorageMode::new_test(None)).unwrap());
