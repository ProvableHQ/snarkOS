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

#[allow(dead_code)]
mod common;
#[allow(dead_code)]
mod components;

use snarkos_account::Account;
use snarkos_node_bft::{
    helpers::{now, Storage},
    ledger_service::CoreLedgerService,
    storage_service::BFTMemoryService,
};
use snarkos_node_bft_ledger_service::LedgerService;
use snarkvm::{
    console::{
        account::{Address, PrivateKey},
        network::MainnetV0,
    },
    ledger::{
        narwhal::{BatchCertificate, BatchHeader, Data, Subdag, Transmission, TransmissionID},
        store::{helpers::memory::ConsensusMemory, ConsensusStore},
        Block,
        Execution,
        Fee,
        Input,
        Output,
        Transition,
    },
    prelude::{Field, Group, Ledger, ToBytes, Transaction, Value, VM},
    utilities::TestRng,
};

use aleo_std::StorageMode;
use indexmap::{IndexMap, IndexSet};
use itertools::Itertools;
use rand::Rng;
use std::{
    collections::{BTreeMap, HashMap},
    str::FromStr,
    sync::Arc,
};

type CurrentNetwork = MainnetV0;
type CurrentLedger = Ledger<CurrentNetwork, ConsensusMemory<CurrentNetwork>>;
type CurrentConsensusStore = ConsensusStore<CurrentNetwork, ConsensusMemory<CurrentNetwork>>;

#[ignore]
fn test_prepare_advance_to_next_quorum_block() -> anyhow::Result<()> {
    // Deserialize the block from a file.
    let genesis_bytes = std::fs::read("genesis.bin").unwrap();
    let genesis: Block<CurrentNetwork> = bincode::deserialize(&genesis_bytes).unwrap();
    // Initialize the ledger.
    let ledger = CurrentLedger::load(genesis.clone(), StorageMode::Production).unwrap();
    let core_ledger = Arc::new(CoreLedgerService::new(ledger.clone(), Default::default()));
    // Read subdag from file.
    let subdag_bytes = std::fs::read("subdag.bin").unwrap();
    let subdag: Subdag<CurrentNetwork> = bincode::deserialize(&subdag_bytes).unwrap();
    // Read transmissions from file.
    let transmissions_bytes = std::fs::read("transmissions.bin").unwrap();
    let transmissions: IndexMap<TransmissionID<CurrentNetwork>, Transmission<CurrentNetwork>> =
        bincode::deserialize(&transmissions_bytes).unwrap();
    // Generate block.
    let _block = core_ledger.prepare_advance_to_next_quorum_block(subdag, transmissions)?;
    return Ok(());
}

#[ignore]
fn test_check_next_block() -> anyhow::Result<()> {
    // Deserialize the block from a file.
    let genesis_bytes = std::fs::read("genesis.bin").unwrap();
    let genesis: Block<CurrentNetwork> = bincode::deserialize(&genesis_bytes).unwrap();
    // Initialize the ledger.
    let ledger = CurrentLedger::load(genesis.clone(), StorageMode::Production).unwrap();
    let core_ledger = Arc::new(CoreLedgerService::new(ledger.clone(), Default::default()));
    // Read block from file.
    let block_bytes = std::fs::read("block.bin").unwrap();
    let block: Block<CurrentNetwork> = bincode::deserialize(&block_bytes).unwrap();
    // Print the number of transactions in the block.
    let num_transactions = block.transactions().len();
    println!("Block contains {num_transactions} transactions.");
    // Check next block
    core_ledger.check_next_block(&block)?;
    return Ok(());
}

#[ignore]
fn generate_blocks_for_profiling() -> anyhow::Result<()> {
    let rng = &mut TestRng::fixed(1);

    // Initialize the round parameters.
    let max_gc_rounds = BatchHeader::<CurrentNetwork>::MAX_GC_ROUNDS as u64;
    let commit_round = 2;
    let committee_size = 40u16;
    let num_transactions = committee_size as usize * 1000;
    let range_to_drain = 0..BatchHeader::<CurrentNetwork>::MAX_TRANSMISSIONS_PER_BATCH;

    // Initialize the store.
    let store = CurrentConsensusStore::open(None).unwrap();
    let account: Account<CurrentNetwork> = Account::new(rng)?;

    // Create a genesis block with a seeded RNG to reproduce the same genesis private keys.
    let seed: u64 = rng.gen();
    let genesis_rng = &mut TestRng::from_seed(seed);
    let genesis =
        VM::from(store).unwrap().large_genesis_beacon(account.private_key(), genesis_rng, committee_size).unwrap();
    // let genesis = VM::from(store).unwrap().genesis_beacon(account.private_key(), genesis_rng).unwrap();

    // Serialize the genesis block and write to disk.
    let genesis_bytes = bincode::serialize(&genesis).unwrap();
    std::fs::write("genesis.bin", &genesis_bytes).unwrap();

    // Deserialize the block from a file.
    // let genesis_bytes = std::fs::read("genesis.bin").unwrap();
    // let genesis: Block<CurrentNetwork> = bincode::deserialize(&genesis_bytes).unwrap();
    // Initialize the ledger.
    let ledger = CurrentLedger::load(genesis.clone(), StorageMode::Production).unwrap();
    let core_ledger = Arc::new(CoreLedgerService::new(ledger.clone(), Default::default()));

    // Extract the private keys from the genesis committee by using the same RNG to sample private keys.
    let genesis_rng = &mut TestRng::from_seed(seed);
    let private_keys = [Ok(*account.private_key())]
        .into_iter()
        .chain((1..committee_size).map(|_| PrivateKey::new(genesis_rng)))
        .collect::<Result<Vec<_>, _>>()?;

    // Initialize the storage.
    let storage = Storage::<CurrentNetwork>::new(core_ledger.clone(), Arc::new(BFTMemoryService::new()), max_gc_rounds);

    // TODO: can't import assert_storage.
    // Ensure the storage is empty.
    // assert_storage(&storage, &[], &[], &[], &Default::default());

    // Sample transactions.
    let transaction = sample_transaction(&ledger, &account, rng);
    let transactions = mutate_transaction(&transaction, num_transactions, rng);
    let transactions = transactions.into_iter().map(|tx| (tx.id(), tx)).collect::<IndexMap<_, _>>();
    let transactions_serialized = transactions
        .iter()
        .map(|(_, tx)| Data::<Transaction<CurrentNetwork>>::Buffer(tx.to_bytes_le().unwrap().into()))
        .collect::<Vec<_>>();
    let mut transmission_ids = transactions
        .iter()
        .map(|(id, tx)| {
            let checksum = Data::<Transaction<CurrentNetwork>>::Buffer(tx.to_bytes_le().unwrap().into())
                .to_checksum::<CurrentNetwork>()
                .unwrap();
            TransmissionID::Transaction(*id, checksum)
        })
        .collect::<IndexSet<_>>();
    let mut transmissions =
        transactions_serialized.into_iter().map(|tx| Transmission::Transaction(tx)).collect::<Vec<_>>();

    // Sample 5 rounds of batch certificates starting at the genesis round from a static set of 4 authors.
    let (round_to_certificates_map, committee) = {
        let committee = ledger.latest_committee().unwrap();

        // Initialize a mapping from the round number to the set of batch certificates in the round.
        type TransmissionsForCertificate =
            IndexMap<Field<CurrentNetwork>, HashMap<TransmissionID<CurrentNetwork>, Transmission<CurrentNetwork>>>;
        let mut round_to_certificates_map: HashMap<
            u64,
            (IndexSet<BatchCertificate<CurrentNetwork>>, TransmissionsForCertificate),
        > = HashMap::new();
        let mut previous_certificates: IndexSet<BatchCertificate<CurrentNetwork>> = IndexSet::with_capacity(4);

        for round in 0..=commit_round + 8 {
            // NOTE: we're not using most of these rounds
            // Sample enough rounds to create 3 blocks.
            let mut transmissions_and_ids_for_certificate: TransmissionsForCertificate = IndexMap::new();
            let mut current_certificates = IndexSet::new();
            let previous_certificate_ids: IndexSet<_> = if round == 0 || round == 1 {
                IndexSet::new()
            } else {
                previous_certificates.iter().map(|c| c.id()).collect()
            };
            let committee_id = committee.id();

            // Create a certificate for each validator.
            for (i, private_key_1) in private_keys.iter().enumerate() {
                let transmissions_for_round = transmissions.drain(range_to_drain.clone()).collect::<Vec<_>>();
                let ids_for_round = transmission_ids.drain(range_to_drain.clone()).collect::<IndexSet<_>>();
                let batch_header = BatchHeader::new(
                    private_key_1,
                    round,
                    now(),
                    committee_id,
                    ids_for_round.clone(),
                    // Default::default(),
                    previous_certificate_ids.clone(),
                    rng,
                )
                .unwrap();
                // Sign the batch header.
                let mut signatures = IndexSet::with_capacity(committee_size as usize);
                for (j, private_key_2) in private_keys.iter().enumerate() {
                    if i != j {
                        signatures.insert(private_key_2.sign(&[batch_header.batch_id()], rng).unwrap());
                    }
                }
                let certificate = BatchCertificate::from(batch_header, signatures).unwrap();

                let transmissions_and_ids =
                    ids_for_round.clone().into_iter().zip_eq(transmissions_for_round.clone().into_iter()).collect();
                transmissions_and_ids_for_certificate.insert(certificate.id(), transmissions_and_ids);

                current_certificates.insert(certificate);
            }
            // Update the map of certificates.
            round_to_certificates_map
                .insert(round, (current_certificates.clone(), transmissions_and_ids_for_certificate));
            previous_certificates = current_certificates.clone();
        }
        (round_to_certificates_map, committee)
    };

    // Insert certificates into storage.
    let mut all_transmissions = IndexMap::new();
    for i in 1..=commit_round + 8 {
        let (certs_for_round, transmissions_and_ids_for_certificate) =
            (*round_to_certificates_map.get(&i).unwrap()).clone();
        for certificate in certs_for_round.iter() {
            let transmissions_and_ids_for_cert = if let Some(transmissions_and_ids_for_cert) =
                transmissions_and_ids_for_certificate.get(&certificate.id())
            {
                transmissions_and_ids_for_cert.clone()
            } else {
                HashMap::new()
            };
            storage
                .insert_certificate(certificate.clone(), transmissions_and_ids_for_cert.clone(), Default::default())
                .unwrap();
            for (id, transmission) in transmissions_and_ids_for_cert.into_iter() {
                if all_transmissions.get(&id).is_none() {
                    all_transmissions.insert(id, transmission);
                }
            }
        }
    }

    // Create block 1.
    let leader_round_1 = commit_round;
    let leader_1 = committee.get_leader(leader_round_1).unwrap();
    let leader_certificate = storage.get_certificate_for_round_with_author(commit_round, leader_1).unwrap();
    let _block_1 = {
        let mut subdag_map: BTreeMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> = BTreeMap::new();
        let mut leader_cert_map = IndexSet::new();
        leader_cert_map.insert(leader_certificate.clone());
        let mut previous_cert_map = IndexSet::new();
        for cert in storage.get_certificates_for_round(commit_round - 1) {
            previous_cert_map.insert(cert);
        }
        subdag_map.insert(commit_round, leader_cert_map.clone());
        subdag_map.insert(commit_round - 1, previous_cert_map.clone());
        let subdag = Subdag::from(subdag_map.clone())?;

        let transmissions = subdag
            .transmission_ids()
            .map(|id| (*id, all_transmissions.get(id).unwrap().clone()))
            .collect::<IndexMap<_, _>>();
        println!("num transmissions: {}", transmissions.len());
        println!("num certificates: {}", subdag.certificate_ids().collect::<Vec<_>>().len());

        // Serialize the subdag.
        let subdag_bytes = bincode::serialize(&subdag).unwrap();
        std::fs::write("subdag.bin", &subdag_bytes).unwrap();

        // Serialize the transmissions.
        let transmissions_bytes = bincode::serialize(&transmissions).unwrap();
        std::fs::write("transmissions.bin", &transmissions_bytes).unwrap();

        // Generate block.
        let block = core_ledger.prepare_advance_to_next_quorum_block(subdag, transmissions)?;
        // Print the number of transactions in the block.
        let num_transactions = block.transactions().len();
        println!("Block contains {num_transactions} transactions.");
        // Check the block.
        core_ledger.check_next_block(&block)?;

        // Serialize the block and write to disk.
        let block_bytes = bincode::serialize(&block).unwrap();
        std::fs::write("block.bin", &block_bytes).unwrap();

        block
    };
    return Ok(()); // NOTE: we currently only generate a single block.
    // Insert block 1.
    // core_ledger.advance_to_next_block(&block_1)?;

    // // Create block 2.
    // let leader_round_2 = commit_round + 2;
    // let leader_2 = committee.get_leader(leader_round_2).unwrap();
    // let leader_certificate_2 = storage.get_certificate_for_round_with_author(leader_round_2, leader_2).unwrap();
    // let block_2 = {
    //     let mut subdag_map_2: BTreeMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> = BTreeMap::new();
    //     let mut leader_cert_map_2 = IndexSet::new();
    //     leader_cert_map_2.insert(leader_certificate_2.clone());
    //     let mut previous_cert_map_2 = IndexSet::new();
    //     for cert in storage.get_certificates_for_round(leader_round_2 - 1) {
    //         previous_cert_map_2.insert(cert);
    //     }
    //     subdag_map_2.insert(leader_round_2, leader_cert_map_2.clone());
    //     subdag_map_2.insert(leader_round_2 - 1, previous_cert_map_2.clone());
    //     let subdag_2 = Subdag::from(subdag_map_2.clone())?;
    //     core_ledger.prepare_advance_to_next_quorum_block(subdag_2, Default::default())?
    // };
    // // Insert block 2.
    // core_ledger.advance_to_next_block(&block_2)?;

    // // Create block 3
    // let leader_round_3 = commit_round + 4;
    // let leader_3 = committee.get_leader(leader_round_3).unwrap();
    // let leader_certificate_3 = storage.get_certificate_for_round_with_author(leader_round_3, leader_3).unwrap();
    // let block_3 = {
    //     let mut subdag_map_3: BTreeMap<u64, IndexSet<BatchCertificate<CurrentNetwork>>> = BTreeMap::new();
    //     let mut leader_cert_map_3 = IndexSet::new();
    //     leader_cert_map_3.insert(leader_certificate_3.clone());
    //     let mut previous_cert_map_3 = IndexSet::new();
    //     for cert in storage.get_certificates_for_round(leader_round_3 - 1) {
    //         previous_cert_map_3.insert(cert);
    //     }
    //     subdag_map_3.insert(leader_round_3, leader_cert_map_3.clone());
    //     subdag_map_3.insert(leader_round_3 - 1, previous_cert_map_3.clone());
    //     let subdag_3 = Subdag::from(subdag_map_3.clone())?;
    //     core_ledger.prepare_advance_to_next_quorum_block(subdag_3, Default::default())?
    // };
    // // Insert block 3.
    // core_ledger.advance_to_next_block(&block_3)?;
    //
    // Ok(())
}

fn sample_transaction(
    ledger: &CurrentLedger,
    account: &Account<CurrentNetwork>,
    rng: &mut TestRng,
) -> Transaction<CurrentNetwork> {
    let recipient_private_key = PrivateKey::<CurrentNetwork>::new(rng).unwrap();
    let recipient_address = Address::try_from(recipient_private_key).unwrap();
    let inputs = [Value::from_str(&format!("{recipient_address}")).unwrap(), Value::from_str("1u64").unwrap()];
    ledger
        .vm()
        .execute(account.private_key(), ("credits.aleo", "transfer_public"), inputs.into_iter(), None, 0, None, rng)
        .unwrap()
}

// TODO: this can be cleaned up.
fn mutate_transaction(
    transaction: &Transaction<CurrentNetwork>,
    num_mutations: usize,
    _rng: &mut TestRng,
) -> Vec<Transaction<CurrentNetwork>> {
    let mut transactions = Vec::with_capacity(num_mutations);
    // Modify the transaction in-place
    if let Transaction::Execute(_, ref mut execution, ref mut fee) = transaction.clone() {
        // Access the first transition
        if let Some(transition) = execution.transitions().next() {
            // Extract the existing values
            let program_id = transition.program_id().clone();
            let function_name = transition.function_name().clone();
            let inputs = transition.inputs().to_vec();

            let outputs = transition.outputs().to_vec();
            let mut tpk = transition.tpk().clone();
            let mut fee_tpk = Group::<MainnetV0>::generator(); // Placeholder initialization

            let scm: Field<MainnetV0> = transition.scm().clone();

            for n in 0..num_mutations as u32 {
                // Increment the `tcm` field by `n`
                let new_transition_tcm = *transition.tcm() + Field::from_u32(n);

                tpk = tpk + Group::<MainnetV0>::generator();

                if n == 0 {
                    if let Some(fee_transition) = fee.as_ref().map(|f| f.transition().clone()) {
                        fee_tpk = fee_transition.tpk().clone();
                    }
                } else {
                    fee_tpk = fee_tpk + Group::<MainnetV0>::generator();
                }

                // create new input Vector and iterate over number of inputs. Add Field::from_u32(n) to the input id
                let mut new_inputs = Vec::new();
                for input in inputs.iter() {
                    let mut new_input = input.clone();
                    match new_input {
                        Input::Constant(ref mut id, _) => *id += Field::from_u32(n),
                        Input::Public(ref mut id, _) => *id += Field::from_u32(n),
                        Input::Private(ref mut id, _) => *id += Field::from_u32(n),
                        Input::Record(ref mut id, ref mut tag) => {
                            *id += Field::from_u32(n);
                            *tag += Field::from_u32(n);
                        }
                        Input::ExternalRecord(ref mut id) => *id += Field::from_u32(n),
                    }
                    new_inputs.push(new_input);
                }

                let mut new_outputs = Vec::new();
                for output in outputs.iter() {
                    let mut new_output = output.clone();
                    match new_output {
                        Output::Constant(ref mut id, _) => *id += Field::from_u32(n),
                        Output::Public(ref mut id, _) => *id += Field::from_u32(n),
                        Output::Private(ref mut id, _) => *id += Field::from_u32(n),
                        Output::Record(ref mut commitment, ref mut checksum, _) => {
                            *commitment += Field::from_u32(n);
                            *checksum += Field::from_u32(n);
                        }
                        Output::ExternalRecord(ref mut id) => *id += Field::from_u32(n),
                        Output::Future(ref mut id, _) => *id += Field::from_u32(n),
                    }
                    new_outputs.push(new_output);
                }

                let new_transition = Transition::new(
                    program_id.clone(),
                    function_name.clone(),
                    new_inputs.clone(),
                    new_outputs.clone(),
                    tpk.clone(),
                    new_transition_tcm,
                    scm.clone(),
                )
                .unwrap();

                // Create a new execution with the new transition
                let new_transitions = vec![new_transition];
                let global_state_root = execution.global_state_root();
                let proof = execution.proof().cloned();

                let mut updated_fee = None;

                // Handle the fee transition
                if let Some(old_fee) = fee.as_ref() {
                    if let Some(fee_transition) = Some(old_fee.transition()) {
                        let fee_program_id = *fee_transition.program_id();
                        let fee_function_name = fee_transition.function_name().clone();
                        let fee_inputs = fee_transition.inputs().to_vec();

                        let fee_outputs = fee_transition.outputs().to_vec();
                        let fee_scm: Field<MainnetV0> = *fee_transition.scm();
                        let new_fee_transition_tcm = *fee_transition.tcm() + Field::from_u32(n);

                        fee_tpk = fee_tpk + Group::<MainnetV0>::generator();

                        let mut new_fee_inputs = Vec::new();
                        for input in fee_inputs.iter() {
                            let mut new_input = input.clone();
                            match new_input {
                                Input::Constant(ref mut id, _) => *id += Field::from_u32(n),
                                Input::Public(ref mut id, _) => *id += Field::from_u32(n),
                                Input::Private(ref mut id, _) => *id += Field::from_u32(n),
                                Input::Record(ref mut id, ref mut tag) => {
                                    *id += Field::from_u32(n);
                                    *tag += Field::from_u32(n);
                                }
                                Input::ExternalRecord(ref mut id) => *id += Field::from_u32(n),
                            }
                            new_fee_inputs.push(new_input);
                        }

                        // Create new output vector and iterate over number of outputs. Add Field::from_u32(n) to the output id
                        let mut new_fee_outputs = Vec::new();
                        for output in fee_outputs.iter() {
                            let mut new_output = output.clone();
                            match new_output {
                                Output::Constant(ref mut id, _) => *id += Field::from_u32(n),
                                Output::Public(ref mut id, _) => *id += Field::from_u32(n),
                                Output::Private(ref mut id, _) => *id += Field::from_u32(n),
                                Output::Record(ref mut commitment, ref mut checksum, _) => {
                                    *commitment += Field::from_u32(n);
                                    *checksum += Field::from_u32(n);
                                }
                                Output::ExternalRecord(ref mut id) => *id += Field::from_u32(n),
                                Output::Future(ref mut id, _) => *id += Field::from_u32(n),
                            }
                            new_fee_outputs.push(new_output);
                        }

                        let new_fee_transition = Transition::new(
                            fee_program_id,
                            fee_function_name,
                            new_fee_inputs,
                            new_fee_outputs,
                            fee_tpk,
                            new_fee_transition_tcm,
                            fee_scm,
                        )
                        .unwrap();

                        // Update the fee with the new transition
                        let new_fee =
                            Fee::from(new_fee_transition, old_fee.global_state_root(), old_fee.proof().cloned())
                                .unwrap();

                        updated_fee = Some(new_fee);
                    }
                }

                let new_execution =
                    Execution::from(new_transitions.into_iter(), global_state_root.clone(), proof.clone()).unwrap();

                // Create a new transaction with the new execution and updated fee
                let tx = Transaction::from_execution(new_execution, updated_fee).unwrap();
                transactions.push(tx);
            }
        }
    }
    transactions
}
