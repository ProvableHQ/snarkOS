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

use super::*;
use snarkos_node_router::messages::UnconfirmedSolution;
use snarkvm::{
    ledger::puzzle::Solution,
    prelude::{Address, Identifier, LimitedWriter, Plaintext, Program, ToBytes, block::Transaction},
};

use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use std::collections::HashMap;

#[cfg(not(feature = "serial"))]
use rayon::prelude::*;

use version::VersionInfo;

/// The `get_blocks` query object.
#[derive(Deserialize, Serialize)]
pub(crate) struct BlockRange {
    /// The starting block height (inclusive).
    start: u32,
    /// The ending block height (exclusive).
    end: u32,
}

#[derive(Deserialize, Serialize)]
pub(crate) struct BackupPath {
    path: std::path::PathBuf,
}

/// The query object for `get_mapping_value` and `get_mapping_values`.
#[derive(Copy, Clone, Deserialize, Serialize)]
pub(crate) struct Metadata {
    metadata: Option<bool>,
    all: Option<bool>,
}

/// The return value for a `sync_status` query.
#[skip_serializing_none]
#[derive(Copy, Clone, Serialize)]
struct SyncStatus<'a> {
    /// Is this node fully synced with the network?
    is_synced: bool,
    /// The block height of this node.
    ledger_height: u32,
    /// Which way are we sync'ing (either "cdn" or "p2p")
    sync_mode: &'a str,
    /// The block height of the CDN (if connected to a CDN).
    cdn_height: Option<u32>,
    /// The greatest known block height of a peer.
    /// None, if no peers are connected yet.
    p2p_height: Option<u32>,
    /// The number of outstanding p2p sync requests.
    outstanding_block_requests: usize,
}

impl<N: Network, C: ConsensusStorage<N>, R: Routing<N>> Rest<N, C, R> {
    // GET /<network>/version
    pub(crate) async fn get_version() -> ErasedJson {
        ErasedJson::pretty(VersionInfo::get())
    }

    // Get /<network>/consensus_version
    pub(crate) async fn get_consensus_version(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(N::CONSENSUS_VERSION(rest.ledger.latest_height())? as u16))
    }

    // GET /<network>/block/height/latest
    pub(crate) async fn get_block_height_latest(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.ledger.latest_height())
    }

    // GET /<network>/block/hash/latest
    pub(crate) async fn get_block_hash_latest(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.ledger.latest_hash())
    }

    // GET /<network>/block/latest
    pub(crate) async fn get_block_latest(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.ledger.latest_block())
    }

    // GET /<network>/block/{height}
    // GET /<network>/block/{blockHash}
    pub(crate) async fn get_block(
        State(rest): State<Self>,
        Path(height_or_hash): Path<String>,
    ) -> Result<ErasedJson, RestError> {
        // Manually parse the height or the height of the hash, axum doesn't support different types
        // for the same path param.
        let block = if let Ok(height) = height_or_hash.parse::<u32>() {
            rest.ledger.get_block(height)?
        } else {
            let hash = height_or_hash
                .parse::<N::BlockHash>()
                .map_err(|_| RestError("invalid input, it is neither a block height nor a block hash".to_string()))?;

            rest.ledger.get_block_by_hash(&hash)?
        };

        Ok(ErasedJson::pretty(block))
    }

    // GET /<network>/blocks?start={start_height}&end={end_height}
    pub(crate) async fn get_blocks(
        State(rest): State<Self>,
        Query(block_range): Query<BlockRange>,
    ) -> Result<ErasedJson, RestError> {
        let start_height = block_range.start;
        let end_height = block_range.end;

        const MAX_BLOCK_RANGE: u32 = 50;

        // Ensure the end height is greater than the start height.
        if start_height > end_height {
            return Err(RestError("Invalid block range".to_string()));
        }

        // Ensure the block range is bounded.
        if end_height - start_height > MAX_BLOCK_RANGE {
            return Err(RestError(format!(
                "Cannot request more than {MAX_BLOCK_RANGE} blocks per call (requested {})",
                end_height - start_height
            )));
        }

        // Prepare a closure for the blocking work.
        let get_json_blocks = move || -> Result<ErasedJson, RestError> {
            let blocks = cfg_into_iter!(start_height..end_height)
                .map(|height| rest.ledger.get_block(height))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(ErasedJson::pretty(blocks))
        };

        // Fetch the blocks from ledger and serialize to json.
        match tokio::task::spawn_blocking(get_json_blocks).await {
            Ok(json) => json,
            Err(err) => Err(RestError(format!("Failed to get blocks '{start_height}..{end_height}' - {err}"))),
        }
    }

    // GET /<network>/sync/status
    pub(crate) async fn get_sync_status(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        // Get the CDN height (if we are syncing from a CDN)
        let (cdn_sync, cdn_height) = if let Some(cdn_sync) = &rest.cdn_sync {
            let done = cdn_sync.is_done();

            // Do not show CDN height if we are already done syncing from the CDN.
            let cdn_height = if done { None } else { Some(cdn_sync.get_cdn_height().await?) };

            // Report CDN sync until it is finished.
            (!done, cdn_height)
        } else {
            (false, None)
        };

        // Generate a string representing the current sync mode.
        let sync_mode = if cdn_sync { "cdn" } else { "p2p" };

        Ok(ErasedJson::pretty(SyncStatus {
            sync_mode,
            cdn_height,
            is_synced: !cdn_sync && rest.routing.is_block_synced(),
            ledger_height: rest.ledger.latest_height(),
            p2p_height: rest.block_sync.greatest_peer_block_height(),
            outstanding_block_requests: rest.block_sync.num_outstanding_block_requests(),
        }))
    }

    // GET /<network>/sync/peers
    pub(crate) async fn get_sync_peers(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        let peers: HashMap<String, u32> =
            rest.block_sync.get_peer_heights().into_iter().map(|(addr, height)| (addr.to_string(), height)).collect();
        Ok(ErasedJson::pretty(peers))
    }

    // GET /<network>/sync/requests
    pub(crate) async fn get_sync_requests_summary(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        let summary = rest.block_sync.get_block_requests_summary();
        Ok(ErasedJson::pretty(summary))
    }

    // GET /<network>/sync/requests/list
    pub(crate) async fn get_sync_requests_list(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        let requests = rest.block_sync.get_block_requests_info();
        Ok(ErasedJson::pretty(requests))
    }

    // GET /<network>/height/{blockHash}
    pub(crate) async fn get_height(
        State(rest): State<Self>,
        Path(hash): Path<N::BlockHash>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_height(&hash)?))
    }

    // GET /<network>/block/{height}/header
    pub(crate) async fn get_block_header(
        State(rest): State<Self>,
        Path(height): Path<u32>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_header(height)?))
    }

    // GET /<network>/block/{height}/transactions
    pub(crate) async fn get_block_transactions(
        State(rest): State<Self>,
        Path(height): Path<u32>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_transactions(height)?))
    }

    // GET /<network>/transaction/{transactionID}
    pub(crate) async fn get_transaction(
        State(rest): State<Self>,
        Path(tx_id): Path<N::TransactionID>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_transaction(tx_id)?))
    }

    // GET /<network>/transaction/confirmed/{transactionID}
    pub(crate) async fn get_confirmed_transaction(
        State(rest): State<Self>,
        Path(tx_id): Path<N::TransactionID>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_confirmed_transaction(tx_id)?))
    }

    // GET /<network>/transaction/unconfirmed/{transactionID}
    pub(crate) async fn get_unconfirmed_transaction(
        State(rest): State<Self>,
        Path(tx_id): Path<N::TransactionID>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_unconfirmed_transaction(&tx_id)?))
    }

    // GET /<network>/memoryPool/transmissions
    pub(crate) async fn get_memory_pool_transmissions(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        match rest.consensus {
            Some(consensus) => {
                Ok(ErasedJson::pretty(consensus.unconfirmed_transmissions().collect::<IndexMap<_, _>>()))
            }
            None => Err(RestError("Route isn't available for this node type".to_string())),
        }
    }

    // GET /<network>/memoryPool/solutions
    pub(crate) async fn get_memory_pool_solutions(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        match rest.consensus {
            Some(consensus) => Ok(ErasedJson::pretty(consensus.unconfirmed_solutions().collect::<IndexMap<_, _>>())),
            None => Err(RestError("Route isn't available for this node type".to_string())),
        }
    }

    // GET /<network>/memoryPool/transactions
    pub(crate) async fn get_memory_pool_transactions(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        match rest.consensus {
            Some(consensus) => Ok(ErasedJson::pretty(consensus.unconfirmed_transactions().collect::<IndexMap<_, _>>())),
            None => Err(RestError("Route isn't available for this node type".to_string())),
        }
    }

    // GET /<network>/program/{programID}
    // GET /<network>/program/{programID}?metadata={true}
    pub(crate) async fn get_program(
        State(rest): State<Self>,
        Path(id): Path<ProgramID<N>>,
        metadata: Query<Metadata>,
    ) -> Result<ErasedJson, RestError> {
        // Get the program from the ledger.
        let program = rest.ledger.get_program(id)?;
        // Check if metadata is requested and return the program with metadata if so.
        if metadata.metadata.unwrap_or(false) {
            // Get the edition of the program.
            let edition = rest.ledger.get_latest_edition_for_program(&id)?;
            return rest.return_program_with_metadata(program, edition);
        }
        // Return the program without metadata.
        Ok(ErasedJson::pretty(program))
    }

    // GET /<network>/program/{programID}/{edition}
    // GET /<network>/program/{programID}/{edition}?metadata={true}
    pub(crate) async fn get_program_for_edition(
        State(rest): State<Self>,
        Path((id, edition)): Path<(ProgramID<N>, u16)>,
        metadata: Query<Metadata>,
    ) -> Result<ErasedJson, RestError> {
        // Get the program from the ledger.
        let program = rest.ledger.get_program_for_edition(id, edition)?;
        // Check if metadata is requested and return the program with metadata if so.
        if metadata.metadata.unwrap_or(false) {
            return rest.return_program_with_metadata(program, edition);
        }
        Ok(ErasedJson::pretty(program))
    }

    // A helper function to return the program and its metadata.
    // This function is used in the `get_program` and `get_program_for_edition` functions.
    fn return_program_with_metadata(&self, program: Program<N>, edition: u16) -> Result<ErasedJson, RestError> {
        let id = program.id();
        // Get the transaction ID associated with the program and edition.
        let tx_id = self.ledger.find_transaction_id_from_program_id_and_edition(id, edition)?;
        // Get the optional program owner associated with the program.
        // Note: The owner is only available after `ConsensusVersion::V9`.
        let program_owner = match &tx_id {
            Some(tid) => self
                .ledger
                .vm()
                .block_store()
                .transaction_store()
                .deployment_store()
                .get_deployment(tid)?
                .and_then(|deployment| deployment.program_owner()),
            None => None,
        };
        Ok(ErasedJson::pretty(json!({
            "program": program,
            "edition": edition,
            "transaction_id": tx_id,
            "program_owner": program_owner,
        })))
    }

    // GET /<network>/program/{programID}/latest_edition
    pub(crate) async fn get_latest_program_edition(
        State(rest): State<Self>,
        Path(id): Path<ProgramID<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_latest_edition_for_program(&id)?))
    }

    // GET /<network>/program/{programID}/mappings
    pub(crate) async fn get_mapping_names(
        State(rest): State<Self>,
        Path(id): Path<ProgramID<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.vm().finalize_store().get_mapping_names_confirmed(&id)?))
    }

    // GET /<network>/program/{programID}/mapping/{mappingName}/{mappingKey}
    // GET /<network>/program/{programID}/mapping/{mappingName}/{mappingKey}?metadata={true}
    pub(crate) async fn get_mapping_value(
        State(rest): State<Self>,
        Path((id, name, key)): Path<(ProgramID<N>, Identifier<N>, Plaintext<N>)>,
        metadata: Query<Metadata>,
    ) -> Result<ErasedJson, RestError> {
        // Retrieve the mapping value.
        let mapping_value = rest.ledger.vm().finalize_store().get_value_confirmed(id, name, &key)?;

        // Check if metadata is requested and return the value with metadata if so.
        if metadata.metadata.unwrap_or(false) {
            return Ok(ErasedJson::pretty(json!({
                "data": mapping_value,
                "height": rest.ledger.latest_height(),
            })));
        }

        // Return the value without metadata.
        Ok(ErasedJson::pretty(mapping_value))
    }

    // GET /<network>/program/{programID}/mapping/{mappingName}?all={true}&metadata={true}
    pub(crate) async fn get_mapping_values(
        State(rest): State<Self>,
        Path((id, name)): Path<(ProgramID<N>, Identifier<N>)>,
        metadata: Query<Metadata>,
    ) -> Result<ErasedJson, RestError> {
        // Return an error if the `all` query parameter is not set to `true`.
        if metadata.all != Some(true) {
            return Err(RestError("Invalid query parameter. At this time, 'all=true' must be included".to_string()));
        }

        // Retrieve the latest height.
        let height = rest.ledger.latest_height();

        // Retrieve all the mapping values from the mapping.
        match tokio::task::spawn_blocking(move || rest.ledger.vm().finalize_store().get_mapping_confirmed(id, name))
            .await
        {
            Ok(Ok(mapping_values)) => {
                // Check if metadata is requested and return the mapping with metadata if so.
                if metadata.metadata.unwrap_or(false) {
                    return Ok(ErasedJson::pretty(json!({
                        "data": mapping_values,
                        "height": height,
                    })));
                }

                // Return the full mapping without metadata.
                Ok(ErasedJson::pretty(mapping_values))
            }
            Ok(Err(err)) => Err(RestError(format!("Unable to read mapping - {err}"))),
            Err(err) => Err(RestError(format!("Unable to read mapping - {err}"))),
        }
    }

    // GET /<network>/statePath/{commitment}
    pub(crate) async fn get_state_path_for_commitment(
        State(rest): State<Self>,
        Path(commitment): Path<Field<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_state_path_for_commitment(&commitment)?))
    }

    // GET /<network>/stateRoot/latest
    pub(crate) async fn get_state_root_latest(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.ledger.latest_state_root())
    }

    // GET /<network>/stateRoot/{height}
    pub(crate) async fn get_state_root(
        State(rest): State<Self>,
        Path(height): Path<u32>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_state_root(height)?))
    }

    // GET /<network>/committee/latest
    pub(crate) async fn get_committee_latest(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.latest_committee()?))
    }

    // GET /<network>/committee/{height}
    pub(crate) async fn get_committee(
        State(rest): State<Self>,
        Path(height): Path<u32>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_committee(height)?))
    }

    // GET /<network>/delegators/{validator}
    pub(crate) async fn get_delegators_for_validator(
        State(rest): State<Self>,
        Path(validator): Path<Address<N>>,
    ) -> Result<ErasedJson, RestError> {
        // Do not process the request if the node is too far behind to avoid sending outdated data.
        if !rest.routing.is_within_sync_leniency() {
            return Err(RestError("Unable to  request delegators (node is syncing)".to_string()));
        }

        // Return the delegators for the given validator.
        match tokio::task::spawn_blocking(move || rest.ledger.get_delegators_for_validator(&validator)).await {
            Ok(Ok(delegators)) => Ok(ErasedJson::pretty(delegators)),
            Ok(Err(err)) => Err(RestError(format!("Unable to request delegators - {err}"))),
            Err(err) => Err(RestError(format!("Unable to request delegators - {err}"))),
        }
    }

    // GET /<network>/peers/count
    pub(crate) async fn get_peers_count(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.routing.router().number_of_connected_peers())
    }

    // GET /<network>/peers/all
    pub(crate) async fn get_peers_all(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.routing.router().connected_peers())
    }

    // GET /<network>/peers/all/metrics
    pub(crate) async fn get_peers_all_metrics(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.routing.router().connected_metrics())
    }

    // GET /<network>/node/address
    pub(crate) async fn get_node_address(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.routing.router().address())
    }

    // GET /<network>/find/blockHash/{transactionID}
    pub(crate) async fn find_block_hash(
        State(rest): State<Self>,
        Path(tx_id): Path<N::TransactionID>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_block_hash(&tx_id)?))
    }

    // GET /<network>/find/blockHeight/{stateRoot}
    pub(crate) async fn find_block_height_from_state_root(
        State(rest): State<Self>,
        Path(state_root): Path<N::StateRoot>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_block_height_from_state_root(state_root)?))
    }

    // GET /<network>/find/transactionID/deployment/{programID}
    pub(crate) async fn find_latest_transaction_id_from_program_id(
        State(rest): State<Self>,
        Path(program_id): Path<ProgramID<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_latest_transaction_id_from_program_id(&program_id)?))
    }

    // GET /<network>/find/transactionID/deployment/{programID}/{edition}
    pub(crate) async fn find_transaction_id_from_program_id_and_edition(
        State(rest): State<Self>,
        Path((program_id, edition)): Path<(ProgramID<N>, u16)>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_transaction_id_from_program_id_and_edition(&program_id, edition)?))
    }

    // GET /<network>/find/transactionID/{transitionID}
    pub(crate) async fn find_transaction_id_from_transition_id(
        State(rest): State<Self>,
        Path(transition_id): Path<N::TransitionID>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_transaction_id_from_transition_id(&transition_id)?))
    }

    // GET /<network>/find/transitionID/{inputOrOutputID}
    pub(crate) async fn find_transition_id(
        State(rest): State<Self>,
        Path(input_or_output_id): Path<Field<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_transition_id(&input_or_output_id)?))
    }

    // POST /<network>/transaction/broadcast
    pub(crate) async fn transaction_broadcast(
        State(rest): State<Self>,
        Json(tx): Json<Transaction<N>>,
    ) -> Result<ErasedJson, RestError> {
        // Do not process the transaction if the node is too far behind.
        if !rest.routing.is_within_sync_leniency() {
            return Err(RestError(format!("Unable to broadcast transaction '{}' (node is syncing)", fmt_id(tx.id()))));
        }

        // If the transaction exceeds the transaction size limit, return an error.
        // The buffer is initially roughly sized to hold a `transfer_public`,
        // most transactions will be smaller and this reduces unnecessary allocations.
        // TODO: Should this be a blocking task?
        let buffer = Vec::with_capacity(3000);
        if tx.write_le(LimitedWriter::new(buffer, N::MAX_TRANSACTION_SIZE)).is_err() {
            return Err(RestError("Transaction size exceeds the byte limit".to_string()));
        }

        // If the consensus module is enabled, add the unconfirmed transaction to the memory pool.
        if let Some(consensus) = rest.consensus {
            // Add the unconfirmed transaction to the memory pool.
            consensus.add_unconfirmed_transaction(tx.clone()).await?;
        }

        // Prepare the unconfirmed transaction message.
        let tx_id = tx.id();
        let message = Message::UnconfirmedTransaction(UnconfirmedTransaction {
            transaction_id: tx_id,
            transaction: Data::Object(tx),
        });

        // Broadcast the transaction.
        rest.routing.propagate(message, &[]);

        Ok(ErasedJson::pretty(tx_id))
    }

    // POST /<network>/solution/broadcast
    pub(crate) async fn solution_broadcast(
        State(rest): State<Self>,
        Json(solution): Json<Solution<N>>,
    ) -> Result<ErasedJson, RestError> {
        // Do not process the solution if the node is too far behind.
        if !rest.routing.is_within_sync_leniency() {
            return Err(RestError(format!(
                "Unable to broadcast solution '{}' (node is syncing)",
                fmt_id(solution.id())
            )));
        }

        // If the consensus module is enabled, add the unconfirmed solution to the memory pool.
        // Otherwise, verify it prior to broadcasting.
        match rest.consensus {
            // Add the unconfirmed solution to the memory pool.
            Some(consensus) => consensus.add_unconfirmed_solution(solution).await?,
            // Verify the solution.
            None => {
                // Compute the current epoch hash.
                let epoch_hash = rest.ledger.latest_epoch_hash()?;
                // Retrieve the current proof target.
                let proof_target = rest.ledger.latest_proof_target();
                // Ensure that the solution is valid for the given epoch.
                let puzzle = rest.ledger.puzzle().clone();
                // Check if the prover has reached their solution limit.
                // While snarkVM will ultimately abort any excess solutions for safety, performing this check
                // here prevents the to-be aborted solutions from propagating through the network.
                let prover_address = solution.address();
                if rest.ledger.is_solution_limit_reached(&prover_address, 0) {
                    return Err(RestError(format!(
                        "Invalid solution '{}' - Prover '{prover_address}' has reached their solution limit for the current epoch",
                        fmt_id(solution.id())
                    )));
                }
                // Verify the solution in a blocking task.
                match tokio::task::spawn_blocking(move || puzzle.check_solution(&solution, epoch_hash, proof_target))
                    .await
                {
                    Ok(Ok(())) => {}
                    Ok(Err(err)) => {
                        return Err(RestError(format!("Invalid solution '{}' - {err}", fmt_id(solution.id()))));
                    }
                    Err(err) => return Err(RestError(format!("Invalid solution '{}' - {err}", fmt_id(solution.id())))),
                }
            }
        }

        let solution_id = solution.id();
        // Prepare the unconfirmed solution message.
        let message =
            Message::UnconfirmedSolution(UnconfirmedSolution { solution_id, solution: Data::Object(solution) });

        // Broadcast the unconfirmed solution message.
        rest.routing.propagate(message, &[]);

        Ok(ErasedJson::pretty(solution_id))
    }

    // POST /{network}/db_backup?path=new_fs_path
    pub(crate) async fn db_backup(
        State(rest): State<Self>,
        backup_path: Query<BackupPath>,
    ) -> Result<ErasedJson, RestError> {
        rest.ledger.backup_database(&backup_path.path).map_err(RestError::from)?;

        Ok(ErasedJson::pretty(()))
    }

    // GET /{network}/block/{blockHeight}/history/{mapping}
    #[cfg(feature = "history")]
    pub(crate) async fn get_history(
        State(rest): State<Self>,
        Path((height, mapping)): Path<(u32, snarkvm::synthesizer::MappingName)>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        // Retrieve the history for the given block height and variant.
        let history = snarkvm::synthesizer::History::new(N::ID, rest.ledger.vm().finalize_store().storage_mode());
        let result = history
            .load_mapping(height, mapping)
            .map_err(|_| RestError(format!("Could not load mapping '{mapping}' from block '{height}'")))?;

        Ok((StatusCode::OK, [(CONTENT_TYPE, "application/json")], result))
    }

    // GET /{network}/validators/participation
    // GET /{network}/validators/participation?metadata={true}
    #[cfg(feature = "telemetry")]
    pub(crate) async fn get_validator_participation_scores(
        State(rest): State<Self>,
        metadata: Query<Metadata>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        match rest.consensus {
            Some(consensus) => {
                // Retrieve the latest committee.
                let latest_committee = rest.ledger.latest_committee()?;
                // Retrieve the latest participation scores.
                let participation_scores = consensus
                    .bft()
                    .primary()
                    .gateway()
                    .validator_telemetry()
                    .get_participation_scores(&latest_committee);

                // Check if metadata is requested and return the participation scores with metadata if so.
                if metadata.metadata.unwrap_or(false) {
                    return Ok(ErasedJson::pretty(json!({
                        "participation_scores": participation_scores,
                        "height": rest.ledger.latest_height(),
                    })));
                }

                Ok(ErasedJson::pretty(participation_scores))
            }
            None => Err(RestError("Route isn't available for this node type".to_string())),
        }
    }
}
