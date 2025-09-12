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

use crate::{BFT, MAX_LEADER_CERTIFICATE_DELAY_IN_SECS, helpers::Storage, sync::SyncCallback};
use snarkos_account::Account;
use snarkos_node_bft_ledger_service::MockLedgerService;
use snarkos_node_bft_storage_service::BFTMemoryService;
use snarkos_node_sync::BlockSync;
use snarkvm::{
    console::account::{Address, PrivateKey},
    ledger::{
        committee::Committee,
        narwhal::batch_certificate::test_helpers::{sample_batch_certificate, sample_batch_certificate_for_round},
    },
    utilities::TestRng,
};

use aleo_std::StorageMode;
use anyhow::Result;
use indexmap::{IndexMap, IndexSet};
use std::sync::Arc;

type CurrentNetwork = snarkvm::console::network::MainnetV0;

/// Samples a new test instance, with an optional committee round and the given maximum GC rounds.
fn sample_test_instance(
    committee_round: Option<u64>,
    max_gc_rounds: u64,
    rng: &mut TestRng,
) -> (Committee<CurrentNetwork>, Account<CurrentNetwork>, Arc<MockLedgerService<CurrentNetwork>>, Storage<CurrentNetwork>)
{
    let committee = match committee_round {
        Some(round) => snarkvm::ledger::committee::test_helpers::sample_committee_for_round(round, rng),
        None => snarkvm::ledger::committee::test_helpers::sample_committee(rng),
    };
    let account = Account::new(rng).unwrap();
    let ledger = Arc::new(MockLedgerService::new(committee.clone()));
    let transmissions = Arc::new(BFTMemoryService::new());
    let storage = Storage::new(ledger.clone(), transmissions, max_gc_rounds);

    (committee, account, ledger, storage)
}

// Helper function to set up BFT for testing.
fn initialize_bft(
    account: Account<CurrentNetwork>,
    storage: Storage<CurrentNetwork>,
    ledger: Arc<MockLedgerService<CurrentNetwork>>,
) -> anyhow::Result<BFT<CurrentNetwork>> {
    // Create the block synchronization logic.
    let block_sync = Arc::new(BlockSync::new(ledger.clone()));
    // Initialize the BFT.
    BFT::new(account.clone(), storage.clone(), ledger.clone(), block_sync, None, &[], StorageMode::new_test(None), None)
}

#[test]
#[tracing_test::traced_test]
fn test_is_leader_quorum_odd() -> Result<()> {
    let rng = &mut TestRng::default();

    // Sample batch certificates.
    let mut certificates = IndexSet::new();
    certificates.insert(snarkvm::ledger::narwhal::batch_certificate::test_helpers::sample_batch_certificate_for_round_with_previous_certificate_ids(1, IndexSet::new(), rng));
    certificates.insert(snarkvm::ledger::narwhal::batch_certificate::test_helpers::sample_batch_certificate_for_round_with_previous_certificate_ids(1, IndexSet::new(), rng));
    certificates.insert(snarkvm::ledger::narwhal::batch_certificate::test_helpers::sample_batch_certificate_for_round_with_previous_certificate_ids(1, IndexSet::new(), rng));
    certificates.insert(snarkvm::ledger::narwhal::batch_certificate::test_helpers::sample_batch_certificate_for_round_with_previous_certificate_ids(1, IndexSet::new(), rng));

    // Initialize the committee.
    let committee = snarkvm::ledger::committee::test_helpers::sample_committee_for_round_and_members(
        1,
        vec![certificates[0].author(), certificates[1].author(), certificates[2].author(), certificates[3].author()],
        rng,
    );

    // Initialize the ledger.
    let ledger = Arc::new(MockLedgerService::new(committee.clone()));
    // Initialize the storage.
    let storage = Storage::new(ledger.clone(), Arc::new(BFTMemoryService::new()), 10);
    // Initialize the account.
    let account = Account::new(rng)?;
    // Initialize the BFT.
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;
    assert!(bft.is_timer_expired());
    // Ensure this call succeeds on an odd round.
    let result = bft.is_leader_quorum_or_nonleaders_available(1);
    // If timer has expired but quorum threshold is not reached, return 'false'.
    assert!(!result);
    // Insert certificates into storage.
    for certificate in certificates.iter() {
        storage.testing_only_insert_certificate_testing_only(certificate.clone());
    }
    // Ensure this call succeeds on an odd round.
    let result = bft.is_leader_quorum_or_nonleaders_available(1);
    assert!(result); // no previous leader certificate
    // Set the leader certificate.
    let leader_certificate = sample_batch_certificate(rng);
    *bft.leader_certificate.write() = Some(leader_certificate);
    // Ensure this call succeeds on an odd round.
    let result = bft.is_leader_quorum_or_nonleaders_available(1);
    assert!(result); // should now fall through to the end of function

    Ok(())
}

#[test]
#[tracing_test::traced_test]
fn test_is_leader_quorum_even_out_of_sync() -> Result<()> {
    let rng = &mut TestRng::default();

    // Sample the test instance.
    let (committee, account, ledger, storage) = sample_test_instance(Some(1), 10, rng);
    assert_eq!(committee.starting_round(), 1);
    assert_eq!(storage.current_round(), 1);
    assert_eq!(storage.max_gc_rounds(), 10);

    // Set up the BFT logic.
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;
    assert!(bft.is_timer_expired());

    // Store is at round 1, and we are checking for round 2.
    // Ensure this call fails on an even round.
    let result = bft.is_leader_quorum_or_nonleaders_available(2);
    assert!(!result);
    Ok(())
}

#[test]
#[tracing_test::traced_test]
fn test_is_leader_quorum_even() -> Result<()> {
    let rng = &mut TestRng::default();

    // Sample the test instance.
    let (committee, account, ledger, storage) = sample_test_instance(Some(2), 10, rng);
    assert_eq!(committee.starting_round(), 2);
    assert_eq!(storage.current_round(), 2);
    assert_eq!(storage.max_gc_rounds(), 10);

    // Set up the BFT logic.
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;
    assert!(bft.is_timer_expired());

    // Ensure this call fails on an even round.
    let result = bft.is_leader_quorum_or_nonleaders_available(2);
    assert!(!result);
    Ok(())
}

#[test]
#[tracing_test::traced_test]
fn test_is_even_round_ready() -> Result<()> {
    let rng = &mut TestRng::default();

    // Sample batch certificates.
    let mut certificates = IndexSet::new();
    certificates.insert(sample_batch_certificate_for_round(2, rng));
    certificates.insert(sample_batch_certificate_for_round(2, rng));
    certificates.insert(sample_batch_certificate_for_round(2, rng));
    certificates.insert(sample_batch_certificate_for_round(2, rng));

    // Initialize the committee.
    let committee = snarkvm::ledger::committee::test_helpers::sample_committee_for_round_and_members(
        2,
        vec![certificates[0].author(), certificates[1].author(), certificates[2].author(), certificates[3].author()],
        rng,
    );

    // Initialize the ledger.
    let ledger = Arc::new(MockLedgerService::new(committee.clone()));
    // Initialize the storage.
    let storage = Storage::new(ledger.clone(), Arc::new(BFTMemoryService::new()), 10);
    // Initialize the account.
    let account = Account::new(rng)?;

    // Set up the BFT logic.
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;
    assert!(bft.is_timer_expired());

    // Set the leader certificate.
    let leader_certificate = sample_batch_certificate_for_round(2, rng);
    *bft.leader_certificate.write() = Some(leader_certificate);
    let result = bft.is_even_round_ready_for_next_round(IndexSet::new(), committee.clone(), 2);
    // If leader certificate is set but quorum threshold is not reached, we are not ready for the next round.
    assert!(!result);
    // Once quorum threshold is reached, we are ready for the next round.
    let result = bft.is_even_round_ready_for_next_round(certificates.clone(), committee.clone(), 2);
    assert!(result);

    // Initialize a new BFT.
    let bft_timer = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;
    // If the leader certificate is not set and the timer has not expired, we are not ready for the next round.
    let result = bft_timer.is_even_round_ready_for_next_round(certificates.clone(), committee.clone(), 2);
    if !bft_timer.is_timer_expired() {
        assert!(!result);
    }
    // Wait for the timer to expire.
    let leader_certificate_timeout =
        std::time::Duration::from_millis(MAX_LEADER_CERTIFICATE_DELAY_IN_SECS as u64 * 1000);
    std::thread::sleep(leader_certificate_timeout);
    // Once the leader certificate timer has expired and quorum threshold is reached, we are ready to advance to the next round.
    let result = bft_timer.is_even_round_ready_for_next_round(certificates.clone(), committee.clone(), 2);
    if bft_timer.is_timer_expired() {
        assert!(result);
    } else {
        assert!(!result);
    }

    Ok(())
}

#[test]
#[tracing_test::traced_test]
fn test_update_leader_certificate_odd() -> Result<()> {
    let rng = &mut TestRng::default();

    // Sample the test instance.
    let (_, account, ledger, storage) = sample_test_instance(None, 10, rng);
    assert_eq!(storage.max_gc_rounds(), 10);

    // Initialize the BFT.
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;
    assert!(bft.is_timer_expired());

    // Ensure this call fails on an odd round.
    let result = bft.update_leader_certificate_to_even_round(1);
    assert!(!result);
    Ok(())
}

#[test]
#[tracing_test::traced_test]
fn test_update_leader_certificate_bad_round() -> Result<()> {
    let rng = &mut TestRng::default();

    // Sample the test instance.
    let (_, account, ledger, storage) = sample_test_instance(None, 10, rng);
    assert_eq!(storage.max_gc_rounds(), 10);

    // Initialize the BFT.
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;

    // Ensure this call succeeds on an even round.
    let result = bft.update_leader_certificate_to_even_round(6);
    assert!(!result);
    Ok(())
}

#[test]
#[tracing_test::traced_test]
fn test_update_leader_certificate_even() -> Result<()> {
    let rng = &mut TestRng::default();

    // Set the current round.
    let current_round = 3;

    // Sample the certificates.
    let (_, certificates) =
        snarkvm::ledger::narwhal::batch_certificate::test_helpers::sample_batch_certificate_with_previous_certificates(
            current_round,
            rng,
        );

    // Initialize the committee.
    let committee = snarkvm::ledger::committee::test_helpers::sample_committee_for_round_and_members(
        2,
        vec![certificates[0].author(), certificates[1].author(), certificates[2].author(), certificates[3].author()],
        rng,
    );

    // Initialize the ledger.
    let ledger = Arc::new(MockLedgerService::new(committee.clone()));

    // Initialize the storage.
    let transmissions = Arc::new(BFTMemoryService::new());
    let storage = Storage::new(ledger.clone(), transmissions, 10);
    storage.testing_only_insert_certificate_testing_only(certificates[0].clone());
    storage.testing_only_insert_certificate_testing_only(certificates[1].clone());
    storage.testing_only_insert_certificate_testing_only(certificates[2].clone());
    storage.testing_only_insert_certificate_testing_only(certificates[3].clone());
    assert_eq!(storage.current_round(), 2);

    // Retrieve the leader certificate.
    let leader = committee.get_leader(2).unwrap();
    let leader_certificate = storage.get_certificate_for_round_with_author(2, leader).unwrap();

    // Initialize the BFT.
    let account = Account::new(rng)?;
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;

    // Set the leader certificate.
    *bft.leader_certificate.write() = Some(leader_certificate);

    // Update the leader certificate.
    // Ensure this call succeeds on an even round.
    let result = bft.update_leader_certificate_to_even_round(2);
    assert!(result);

    Ok(())
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_order_dag_with_dfs() -> Result<()> {
    let rng = &mut TestRng::default();

    // Sample the test instance.
    let (_, account, ledger, _) = sample_test_instance(Some(1), 10, rng);

    // Initialize the round parameters.
    let previous_round = 2; // <- This must be an even number, for `BFT::update_dag` to behave correctly below.
    let current_round = previous_round + 1;

    // Sample the current certificate and previous certificates.
    let (certificate, previous_certificates) =
        snarkvm::ledger::narwhal::batch_certificate::test_helpers::sample_batch_certificate_with_previous_certificates(
            current_round,
            rng,
        );

    /* Test GC */

    // Ensure the function succeeds in returning only certificates above GC.
    {
        // Initialize the storage.
        let storage = Storage::new(ledger.clone(), Arc::new(BFTMemoryService::new()), 1);
        // Initialize the BFT.
        let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;

        // Insert a mock DAG in the BFT.
        *bft.dag.write() = crate::helpers::dag::test_helpers::mock_dag_with_modified_last_committed_round(3);

        // Insert the previous certificates into the BFT.
        for certificate in previous_certificates.clone() {
            assert!(bft.update_dag::<false, false>(certificate).await.is_ok());
        }

        // Ensure this call succeeds and returns all given certificates.
        let result = bft.order_dag_with_dfs::<false>(certificate.clone());
        assert!(result.is_ok());
        let candidate_certificates = result.unwrap().into_values().flatten().collect::<Vec<_>>();
        assert_eq!(candidate_certificates.len(), 1);
        let expected_certificates = vec![certificate.clone()];
        assert_eq!(
            candidate_certificates.iter().map(|c| c.id()).collect::<Vec<_>>(),
            expected_certificates.iter().map(|c| c.id()).collect::<Vec<_>>()
        );
        assert_eq!(candidate_certificates, expected_certificates);
    }

    /* Test normal case */

    // Ensure the function succeeds in returning all given certificates.
    {
        // Initialize the storage.
        let storage = Storage::new(ledger.clone(), Arc::new(BFTMemoryService::new()), 1);
        // Initialize the BFT.
        let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;

        // Insert a mock DAG in the BFT.
        *bft.dag.write() = crate::helpers::dag::test_helpers::mock_dag_with_modified_last_committed_round(2);

        // Insert the previous certificates into the BFT.
        for certificate in previous_certificates.clone() {
            assert!(bft.update_dag::<false, false>(certificate).await.is_ok());
        }

        // Ensure this call succeeds and returns all given certificates.
        let result = bft.order_dag_with_dfs::<false>(certificate.clone());
        assert!(result.is_ok());
        let candidate_certificates = result.unwrap().into_values().flatten().collect::<Vec<_>>();
        assert_eq!(candidate_certificates.len(), 5);
        let expected_certificates = vec![
            previous_certificates[0].clone(),
            previous_certificates[1].clone(),
            previous_certificates[2].clone(),
            previous_certificates[3].clone(),
            certificate,
        ];
        assert_eq!(
            candidate_certificates.iter().map(|c| c.id()).collect::<Vec<_>>(),
            expected_certificates.iter().map(|c| c.id()).collect::<Vec<_>>()
        );
        assert_eq!(candidate_certificates, expected_certificates);
    }

    Ok(())
}

#[test]
#[tracing_test::traced_test]
fn test_order_dag_with_dfs_fails_on_missing_previous_certificate() -> Result<()> {
    let rng = &mut TestRng::default();

    // Sample the test instance.
    let (committee, account, ledger, storage) = sample_test_instance(Some(1), 1, rng);
    assert_eq!(committee.starting_round(), 1);
    assert_eq!(storage.current_round(), 1);
    assert_eq!(storage.max_gc_rounds(), 1);

    // Initialize the round parameters.
    let previous_round = 2; // <- This must be an even number, for `BFT::update_dag` to behave correctly below.
    let current_round = previous_round + 1;

    // Sample the current certificate and previous certificates.
    let (certificate, previous_certificates) =
        snarkvm::ledger::narwhal::batch_certificate::test_helpers::sample_batch_certificate_with_previous_certificates(
            current_round,
            rng,
        );
    // Construct the previous certificate IDs.
    let previous_certificate_ids: IndexSet<_> = previous_certificates.iter().map(|c| c.id()).collect();

    /* Test missing previous certificate. */

    // Initialize the BFT.
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;

    // The expected error message.
    let error_msg = format!(
        "Missing previous certificate {} for round {previous_round}",
        crate::helpers::fmt_id(previous_certificate_ids[3]),
    );

    // Ensure this call fails on a missing previous certificate.
    let result = bft.order_dag_with_dfs::<false>(certificate);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), error_msg);
    Ok(())
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_bft_gc_on_commit() -> Result<()> {
    let rng = &mut TestRng::default();

    // Initialize the round parameters.
    let max_gc_rounds = 1;
    let committee_round = 0;
    let commit_round = 2;
    let current_round = commit_round + 1;

    // Sample the certificates.
    let (_, certificates) =
        snarkvm::ledger::narwhal::batch_certificate::test_helpers::sample_batch_certificate_with_previous_certificates(
            current_round,
            rng,
        );

    // Initialize the committee.
    let committee = snarkvm::ledger::committee::test_helpers::sample_committee_for_round_and_members(
        committee_round,
        vec![certificates[0].author(), certificates[1].author(), certificates[2].author(), certificates[3].author()],
        rng,
    );

    // Initialize the ledger.
    let ledger = Arc::new(MockLedgerService::new(committee.clone()));

    // Initialize the storage.
    let transmissions = Arc::new(BFTMemoryService::new());
    let storage = Storage::new(ledger.clone(), transmissions, max_gc_rounds);
    // Insert the certificates into the storage.
    for certificate in certificates.iter() {
        storage.testing_only_insert_certificate_testing_only(certificate.clone());
    }

    // Get the leader certificate.
    let leader = committee.get_leader(commit_round).unwrap();
    let leader_certificate = storage.get_certificate_for_round_with_author(commit_round, leader).unwrap();

    // Initialize the BFT.
    let account = Account::new(rng)?;
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;

    *bft.dag.write() = crate::helpers::dag::test_helpers::mock_dag_with_modified_last_committed_round(commit_round);

    // Ensure that the `gc_round` has not been updated yet.
    assert_eq!(bft.storage().gc_round(), committee_round.saturating_sub(max_gc_rounds));

    // Insert the certificates into the BFT.
    for certificate in certificates {
        assert!(bft.update_dag::<false, false>(certificate).await.is_ok());
    }

    // Commit the leader certificate.
    bft.commit_leader_certificate::<false, false>(leader_certificate).await.unwrap();

    // Ensure that the `gc_round` has been updated.
    assert_eq!(bft.storage().gc_round(), commit_round - max_gc_rounds);

    Ok(())
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_sync_bft_dag_at_bootup() -> Result<()> {
    let rng = &mut TestRng::default();

    // Initialize the round parameters.
    let max_gc_rounds = 1;
    let committee_round = 0;
    let commit_round = 2;
    let current_round = commit_round + 1;

    // Sample the current certificate and previous certificates.
    let (_, certificates) =
        snarkvm::ledger::narwhal::batch_certificate::test_helpers::sample_batch_certificate_with_previous_certificates(
            current_round,
            rng,
        );

    // Initialize the committee.
    let committee = snarkvm::ledger::committee::test_helpers::sample_committee_for_round_and_members(
        committee_round,
        vec![certificates[0].author(), certificates[1].author(), certificates[2].author(), certificates[3].author()],
        rng,
    );

    // Initialize the ledger.
    let ledger = Arc::new(MockLedgerService::new(committee.clone()));

    // Initialize the storage.
    let storage = Storage::new(ledger.clone(), Arc::new(BFTMemoryService::new()), max_gc_rounds);
    // Insert the certificates into the storage.
    for certificate in certificates.iter() {
        storage.testing_only_insert_certificate_testing_only(certificate.clone());
    }

    // Get the leader certificate.
    let leader = committee.get_leader(commit_round).unwrap();
    let leader_certificate = storage.get_certificate_for_round_with_author(commit_round, leader).unwrap();

    // Initialize the BFT.
    let account = Account::new(rng)?;
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;

    // Insert a mock DAG in the BFT.
    *bft.dag.write() = crate::helpers::dag::test_helpers::mock_dag_with_modified_last_committed_round(commit_round);

    // Insert the previous certificates into the BFT.
    for certificate in certificates.clone() {
        assert!(bft.update_dag::<false, false>(certificate).await.is_ok());
    }

    // Commit the leader certificate.
    bft.commit_leader_certificate::<false, false>(leader_certificate.clone()).await.unwrap();

    // Simulate a bootup of the BFT.

    // Initialize a new instance of storage.
    let storage_2 = Storage::new(ledger.clone(), Arc::new(BFTMemoryService::new()), max_gc_rounds);
    // Initialize a new instance of BFT.
    let bootup_bft = initialize_bft(account.clone(), storage_2, ledger)?;

    // Sync the BFT DAG at bootup.
    bootup_bft.sync_dag_at_bootup(certificates.clone()).await.unwrap();

    // Check that the BFT starts from the same last committed round.
    assert_eq!(bft.dag.read().last_committed_round(), bootup_bft.dag.read().last_committed_round());

    // Ensure that both BFTs have committed the leader certificate.
    assert!(bft.dag.read().is_recently_committed(leader_certificate.round(), leader_certificate.id()));
    assert!(bootup_bft.dag.read().is_recently_committed(leader_certificate.round(), leader_certificate.id()));

    // Check the state of the bootup BFT.
    for certificate in certificates {
        let certificate_round = certificate.round();
        let certificate_id = certificate.id();
        // Check that the bootup BFT has committed the certificates.
        assert!(bootup_bft.dag.read().is_recently_committed(certificate_round, certificate_id));
        // Check that the bootup BFT does not contain the certificates in its graph, because
        // it should not need to order them again in subsequent subdags.
        assert!(!bootup_bft.dag.read().contains_certificate_in_round(certificate_round, certificate_id));
    }

    Ok(())
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_sync_bft_dag_at_bootup_shutdown() -> Result<()> {
    /*
    1. Run one uninterrupted BFT on a set of certificates for 2 leader commits.
    2. Run a separate bootup BFT that syncs with a set of pre shutdown certificates, and then commits a second leader normally over a set of post shutdown certificates.
    3. Observe that the uninterrupted BFT and the bootup BFT end in the same state.
    */

    let rng = &mut TestRng::default();

    // Initialize the round parameters.
    let max_gc_rounds = snarkvm::ledger::narwhal::BatchHeader::<CurrentNetwork>::MAX_GC_ROUNDS as u64;
    let committee_round = 0;
    let commit_round = 2;
    let current_round = commit_round + 1;
    let next_round = current_round + 1;

    // Sample 5 rounds of batch certificates starting at the genesis round from a static set of 4 authors.
    let (round_to_certificates_map, committee) = {
        let private_keys = vec![
            PrivateKey::new(rng).unwrap(),
            PrivateKey::new(rng).unwrap(),
            PrivateKey::new(rng).unwrap(),
            PrivateKey::new(rng).unwrap(),
        ];
        let addresses = vec![
            Address::try_from(private_keys[0])?,
            Address::try_from(private_keys[1])?,
            Address::try_from(private_keys[2])?,
            Address::try_from(private_keys[3])?,
        ];
        let committee = snarkvm::ledger::committee::test_helpers::sample_committee_for_round_and_members(
            committee_round,
            addresses,
            rng,
        );
        // Initialize a mapping from the round number to the set of batch certificates in the round.
        let mut round_to_certificates_map: IndexMap<
            u64,
            IndexSet<snarkvm::ledger::narwhal::BatchCertificate<CurrentNetwork>>,
        > = IndexMap::new();
        let mut previous_certificates = IndexSet::with_capacity(4);
        // Initialize the genesis batch certificates.
        for _ in 0..4 {
            previous_certificates.insert(sample_batch_certificate(rng));
        }
        for round in 0..commit_round + 3 {
            let mut current_certificates = IndexSet::new();
            let previous_certificate_ids: IndexSet<_> = if round == 0 || round == 1 {
                IndexSet::new()
            } else {
                previous_certificates.iter().map(|c| c.id()).collect()
            };
            let transmission_ids =
                snarkvm::ledger::narwhal::transmission_id::test_helpers::sample_transmission_ids(rng)
                    .into_iter()
                    .collect::<IndexSet<_>>();
            let timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
            let committee_id = committee.id();
            for (i, private_key_1) in private_keys.iter().enumerate() {
                let batch_header = snarkvm::ledger::narwhal::BatchHeader::new(
                    private_key_1,
                    round,
                    timestamp,
                    committee_id,
                    transmission_ids.clone(),
                    previous_certificate_ids.clone(),
                    rng,
                )
                .unwrap();
                let mut signatures = IndexSet::with_capacity(4);
                for (j, private_key_2) in private_keys.iter().enumerate() {
                    if i != j {
                        signatures.insert(private_key_2.sign(&[batch_header.batch_id()], rng).unwrap());
                    }
                }
                let certificate = snarkvm::ledger::narwhal::BatchCertificate::from(batch_header, signatures).unwrap();
                current_certificates.insert(certificate);
            }
            // Update the mapping.
            round_to_certificates_map.insert(round, current_certificates.clone());
            previous_certificates = current_certificates.clone();
        }
        (round_to_certificates_map, committee)
    };

    // Initialize the ledger.
    let ledger = Arc::new(MockLedgerService::new(committee.clone()));
    // Initialize the storage.
    let storage = Storage::new(ledger.clone(), Arc::new(BFTMemoryService::new()), max_gc_rounds);
    // Get the leaders for the next 2 commit rounds.
    let leader = committee.get_leader(commit_round).unwrap();
    let next_leader = committee.get_leader(next_round).unwrap();
    // Insert the pre shutdown certificates into the storage.
    let mut pre_shutdown_certificates: Vec<snarkvm::ledger::narwhal::BatchCertificate<CurrentNetwork>> = Vec::new();
    for i in 1..=commit_round {
        let certificates = (*round_to_certificates_map.get(&i).unwrap()).clone();
        if i == commit_round {
            // Only insert the leader certificate for the commit round.
            let leader_certificate = certificates.iter().find(|certificate| certificate.author() == leader);
            if let Some(c) = leader_certificate {
                pre_shutdown_certificates.push(c.clone());
            }
            continue;
        }
        pre_shutdown_certificates.extend(certificates);
    }
    for certificate in pre_shutdown_certificates.iter() {
        storage.testing_only_insert_certificate_testing_only(certificate.clone());
    }
    // Insert the post shutdown certificates into the storage.
    let mut post_shutdown_certificates: Vec<snarkvm::ledger::narwhal::BatchCertificate<CurrentNetwork>> = Vec::new();
    for j in commit_round..=commit_round + 2 {
        let certificate = (*round_to_certificates_map.get(&j).unwrap()).clone();
        post_shutdown_certificates.extend(certificate);
    }
    for certificate in post_shutdown_certificates.iter() {
        storage.testing_only_insert_certificate_testing_only(certificate.clone());
    }
    // Get the leader certificates.
    let leader_certificate = storage.get_certificate_for_round_with_author(commit_round, leader).unwrap();
    let next_leader_certificate = storage.get_certificate_for_round_with_author(next_round, next_leader).unwrap();

    // Initialize the BFT without bootup.
    let account = Account::new(rng)?;
    let bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;

    // Insert a mock DAG in the BFT without bootup.
    *bft.dag.write() = crate::helpers::dag::test_helpers::mock_dag_with_modified_last_committed_round(0);

    // Insert the certificates into the BFT without bootup.
    for certificate in pre_shutdown_certificates.clone() {
        assert!(bft.update_dag::<false, false>(certificate).await.is_ok());
    }

    // Insert the post shutdown certificates into the BFT without bootup.
    for certificate in post_shutdown_certificates.clone() {
        assert!(bft.update_dag::<false, false>(certificate).await.is_ok());
    }
    // Commit the second leader certificate.
    let commit_subdag = bft.order_dag_with_dfs::<false>(next_leader_certificate.clone()).unwrap();
    let commit_subdag_metadata = commit_subdag.iter().map(|(round, c)| (*round, c.len())).collect::<Vec<_>>();
    bft.commit_leader_certificate::<false, false>(next_leader_certificate.clone()).await.unwrap();

    // Simulate a bootup of the BFT.

    // Initialize a new instance of storage.
    let bootup_storage = Storage::new(ledger.clone(), Arc::new(BFTMemoryService::new()), max_gc_rounds);

    // Initialize a new instance of BFT with bootup.
    let bootup_bft = initialize_bft(account.clone(), bootup_storage.clone(), ledger.clone())?;

    // Sync the BFT DAG at bootup.
    bootup_bft.sync_dag_at_bootup(pre_shutdown_certificates.clone()).await.unwrap();

    // Insert the post shutdown certificates to the storage and BFT with bootup.
    for certificate in post_shutdown_certificates.iter() {
        bootup_bft.storage().testing_only_insert_certificate_testing_only(certificate.clone());
    }
    for certificate in post_shutdown_certificates.clone() {
        assert!(bootup_bft.update_dag::<false, false>(certificate).await.is_ok());
    }
    // Commit the second leader certificate.
    let commit_subdag_bootup = bootup_bft.order_dag_with_dfs::<false>(next_leader_certificate.clone()).unwrap();
    let commit_subdag_metadata_bootup =
        commit_subdag_bootup.iter().map(|(round, c)| (*round, c.len())).collect::<Vec<_>>();
    let committed_certificates_bootup = commit_subdag_bootup.values().flatten();
    bootup_bft.commit_leader_certificate::<false, false>(next_leader_certificate.clone()).await.unwrap();

    // Check that the final state of both BFTs is the same.

    // Check that both BFTs start from the same last committed round.
    assert_eq!(bft.dag.read().last_committed_round(), bootup_bft.dag.read().last_committed_round());

    // Ensure that both BFTs have committed the leader certificates.
    assert!(bft.dag.read().is_recently_committed(leader_certificate.round(), leader_certificate.id()));
    assert!(bft.dag.read().is_recently_committed(next_leader_certificate.round(), next_leader_certificate.id()));
    assert!(bootup_bft.dag.read().is_recently_committed(leader_certificate.round(), leader_certificate.id()));
    assert!(bootup_bft.dag.read().is_recently_committed(next_leader_certificate.round(), next_leader_certificate.id()));

    // Check that the bootup BFT has committed the pre shutdown certificates.
    for certificate in pre_shutdown_certificates.clone() {
        let certificate_round = certificate.round();
        let certificate_id = certificate.id();
        // Check that both BFTs have committed the certificates.
        assert!(bft.dag.read().is_recently_committed(certificate_round, certificate_id));
        assert!(bootup_bft.dag.read().is_recently_committed(certificate_round, certificate_id));
        // Check that the bootup BFT does not contain the certificates in its graph, because
        // it should not need to order them again in subsequent subdags.
        assert!(!bft.dag.read().contains_certificate_in_round(certificate_round, certificate_id));
        assert!(!bootup_bft.dag.read().contains_certificate_in_round(certificate_round, certificate_id));
    }

    // Check that that the bootup BFT has committed the subdag stemming from the second leader certificate in consensus.
    for certificate in committed_certificates_bootup.clone() {
        let certificate_round = certificate.round();
        let certificate_id = certificate.id();
        // Check that the both BFTs have committed the certificates.
        assert!(bft.dag.read().is_recently_committed(certificate_round, certificate_id));
        assert!(bootup_bft.dag.read().is_recently_committed(certificate_round, certificate_id));
        // Check that the bootup BFT does not contain the certificates in its graph, because
        // it should not need to order them again in subsequent subdags.
        assert!(!bft.dag.read().contains_certificate_in_round(certificate_round, certificate_id));
        assert!(!bootup_bft.dag.read().contains_certificate_in_round(certificate_round, certificate_id));
    }

    // Check that the commit subdag metadata for the second leader is the same for both BFTs.
    assert_eq!(commit_subdag_metadata_bootup, commit_subdag_metadata);

    Ok(())
}

#[tokio::test]
#[tracing_test::traced_test]
async fn test_sync_bft_dag_at_bootup_dfs() -> Result<()> {
    /*
    1. Run a bootup BFT that syncs with a set of pre shutdown certificates.
    2. Add post shutdown certificates to the bootup BFT.
    2. Observe that in the commit subdag of the second leader certificate, there are no repeated vertices from the pre shutdown certificates.
    */

    let rng = &mut TestRng::default();

    // Initialize the round parameters.
    let max_gc_rounds = snarkvm::ledger::narwhal::BatchHeader::<CurrentNetwork>::MAX_GC_ROUNDS as u64;
    let committee_round = 0;
    let commit_round = 2;
    let current_round = commit_round + 1;
    let next_round = current_round + 1;

    // Sample 5 rounds of batch certificates starting at the genesis round from a static set of 4 authors.
    let (round_to_certificates_map, committee) = {
        let private_keys = vec![
            PrivateKey::new(rng).unwrap(),
            PrivateKey::new(rng).unwrap(),
            PrivateKey::new(rng).unwrap(),
            PrivateKey::new(rng).unwrap(),
        ];
        let addresses = vec![
            Address::try_from(private_keys[0])?,
            Address::try_from(private_keys[1])?,
            Address::try_from(private_keys[2])?,
            Address::try_from(private_keys[3])?,
        ];
        let committee = snarkvm::ledger::committee::test_helpers::sample_committee_for_round_and_members(
            committee_round,
            addresses,
            rng,
        );
        // Initialize a mapping from the round number to the set of batch certificates in the round.
        let mut round_to_certificates_map: IndexMap<
            u64,
            IndexSet<snarkvm::ledger::narwhal::BatchCertificate<CurrentNetwork>>,
        > = IndexMap::new();
        let mut previous_certificates = IndexSet::with_capacity(4);
        // Initialize the genesis batch certificates.
        for _ in 0..4 {
            previous_certificates.insert(sample_batch_certificate(rng));
        }
        for round in 0..=commit_round + 2 {
            let mut current_certificates = IndexSet::new();
            let previous_certificate_ids: IndexSet<_> = if round == 0 || round == 1 {
                IndexSet::new()
            } else {
                previous_certificates.iter().map(|c| c.id()).collect()
            };
            let transmission_ids =
                snarkvm::ledger::narwhal::transmission_id::test_helpers::sample_transmission_ids(rng)
                    .into_iter()
                    .collect::<IndexSet<_>>();
            let timestamp = time::OffsetDateTime::now_utc().unix_timestamp();
            let committee_id = committee.id();
            for (i, private_key_1) in private_keys.iter().enumerate() {
                let batch_header = snarkvm::ledger::narwhal::BatchHeader::new(
                    private_key_1,
                    round,
                    timestamp,
                    committee_id,
                    transmission_ids.clone(),
                    previous_certificate_ids.clone(),
                    rng,
                )
                .unwrap();
                let mut signatures = IndexSet::with_capacity(4);
                for (j, private_key_2) in private_keys.iter().enumerate() {
                    if i != j {
                        signatures.insert(private_key_2.sign(&[batch_header.batch_id()], rng).unwrap());
                    }
                }
                let certificate = snarkvm::ledger::narwhal::BatchCertificate::from(batch_header, signatures).unwrap();
                current_certificates.insert(certificate);
            }
            // Update the mapping.
            round_to_certificates_map.insert(round, current_certificates.clone());
            previous_certificates = current_certificates.clone();
        }
        (round_to_certificates_map, committee)
    };

    // Initialize the ledger.
    let ledger = Arc::new(MockLedgerService::new(committee.clone()));
    // Initialize the storage.
    let storage = Storage::new(ledger.clone(), Arc::new(BFTMemoryService::new()), max_gc_rounds);
    // Get the leaders for the next 2 commit rounds.
    let leader = committee.get_leader(commit_round).unwrap();
    let next_leader = committee.get_leader(next_round).unwrap();
    // Insert the pre shutdown certificates into the storage.
    let mut pre_shutdown_certificates: Vec<snarkvm::ledger::narwhal::BatchCertificate<CurrentNetwork>> = Vec::new();
    for i in 1..=commit_round {
        let certificates = (*round_to_certificates_map.get(&i).unwrap()).clone();
        if i == commit_round {
            // Only insert the leader certificate for the commit round.
            let leader_certificate = certificates.iter().find(|certificate| certificate.author() == leader);
            if let Some(c) = leader_certificate {
                pre_shutdown_certificates.push(c.clone());
            }
            continue;
        }
        pre_shutdown_certificates.extend(certificates);
    }
    for certificate in pre_shutdown_certificates.iter() {
        storage.testing_only_insert_certificate_testing_only(certificate.clone());
    }
    // Initialize the bootup BFT.
    let account = Account::new(rng)?;
    let bootup_bft = initialize_bft(account.clone(), storage.clone(), ledger.clone())?;

    // Insert a mock DAG in the BFT without bootup.
    *bootup_bft.dag.write() = crate::helpers::dag::test_helpers::mock_dag_with_modified_last_committed_round(0);
    // Sync the BFT DAG at bootup.
    bootup_bft.sync_dag_at_bootup(pre_shutdown_certificates.clone()).await.unwrap();

    // Insert the post shutdown certificates into the storage.
    let mut post_shutdown_certificates: Vec<snarkvm::ledger::narwhal::BatchCertificate<CurrentNetwork>> = Vec::new();
    for j in commit_round..=commit_round + 2 {
        let certificate = (*round_to_certificates_map.get(&j).unwrap()).clone();
        post_shutdown_certificates.extend(certificate);
    }
    for certificate in post_shutdown_certificates.iter() {
        storage.testing_only_insert_certificate_testing_only(certificate.clone());
    }

    // Insert the post shutdown certificates into the DAG.
    for certificate in post_shutdown_certificates.clone() {
        assert!(bootup_bft.update_dag::<false, false>(certificate).await.is_ok());
    }

    // Get the next leader certificate to commit.
    let next_leader_certificate = storage.get_certificate_for_round_with_author(next_round, next_leader).unwrap();
    let commit_subdag = bootup_bft.order_dag_with_dfs::<false>(next_leader_certificate).unwrap();
    let committed_certificates = commit_subdag.values().flatten();

    // Check that none of the certificates synced from the bootup appear in the subdag for the next commit round.
    for pre_shutdown_certificate in pre_shutdown_certificates.clone() {
        for committed_certificate in committed_certificates.clone() {
            assert_ne!(pre_shutdown_certificate.id(), committed_certificate.id());
        }
    }
    Ok(())
}
