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

use super::*;
use snarkos_node_network::PeerPoolHandling;
use snarkos_node_router::messages::UnconfirmedSolution;
use snarkos_node_sync::BftSyncMode;
#[cfg(feature = "history-staking-rewards")]
use snarkvm::ledger::store::helpers::MapRead;
use snarkvm::{
    ledger::puzzle::Solution,
    prelude::{
        Address,
        ConsensusVersion,
        Identifier,
        LimitedWriter,
        Plaintext,
        Program,
        ToBytes,
        Value,
        block::Transaction,
    },
    synthesizer::program::{FinalizeGlobalState, StackTrait},
};

use axum::{Json, extract::rejection::JsonRejection};

use aleo_std::aleo_ledger_dir;
use anyhow::{Context, anyhow};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_with::skip_serializing_none;
use std::{collections::HashMap, fs};

#[cfg(not(feature = "serial"))]
use rayon::prelude::*;
use version::VersionInfo;

#[cfg(feature = "history")]
const MAX_KEYS_PER_REQUEST: usize = 1 << 7;
#[cfg(feature = "history")]
type HistoricalMappingKey<N> = (ProgramID<N>, Identifier<N>, Plaintext<N>, u32);
#[cfg(feature = "history")]
type HistoricalMappingRoute<N> = (ProgramID<N>, Identifier<N>, u32);
#[cfg(feature = "history")]
type ViewFunctionRoute<N> = (ProgramID<N>, Identifier<N>, u32);

#[cfg(feature = "history")]
fn parse_historical_mapping_keys<N: Network>(keys: &[String]) -> Result<Vec<Plaintext<N>>, RestError> {
    // Retrieve the number of keys.
    let num_keys = keys.len();
    // Return an error if no keys are provided.
    if num_keys == 0 {
        return Err(RestError::unprocessable_entity(anyhow!("No keys provided")));
    }
    // Return an error if the number of keys exceeds the maximum allowed.
    if num_keys > MAX_KEYS_PER_REQUEST {
        return Err(RestError::unprocessable_entity(anyhow!(
            "Too many keys provided (max: {MAX_KEYS_PER_REQUEST}, got: {num_keys})"
        )));
    }

    // Deserialize the keys from the query.
    keys.iter()
        .enumerate()
        .map(|(index, key)| {
            key.parse::<Plaintext<N>>().map_err(|err| {
                RestError::unprocessable_entity(err.context(format!("Invalid key at index {index}: {key}")))
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

/// Parses a list of strings into a `Vec<Value<N>>` for use as view function inputs.
fn parse_view_inputs<N: Network>(inputs: &[String]) -> Result<Vec<Value<N>>, RestError> {
    inputs
        .iter()
        .enumerate()
        .map(|(index, input)| {
            input.parse::<Value<N>>().map_err(|err| {
                RestError::unprocessable_entity(err.context(format!("Invalid input at index {index}: {input}")))
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

/// Deserialize a CSV string into a vector of strings.
fn de_csv<'de, D>(de: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(de)?;
    Ok(if s.trim().is_empty() { Vec::new() } else { s.split(',').map(|x| x.trim().to_string()).collect() })
}

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

/// The query object for `transaction_broadcast`.
#[derive(Copy, Clone, Deserialize, Serialize)]
pub(crate) struct CheckTransaction {
    check_transaction: Option<bool>,
}

/// The query object for `solution_broadcast`.
#[derive(Copy, Clone, Deserialize, Serialize)]
pub(crate) struct CheckSolution {
    check_solution: Option<bool>,
}

/// The query object for `get_state_paths_for_commitments`.
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct Commitments {
    #[serde(deserialize_with = "de_csv")]
    commitments: Vec<String>,
}

/// The query object for `get_history_batch`.
#[cfg(feature = "history")]
#[derive(Clone, Deserialize, Serialize)]
pub(crate) struct HistoricalKeys {
    #[serde(deserialize_with = "de_csv")]
    keys: Vec<String>,
}

/// The return value for a transaction metadata query.
#[skip_serializing_none]
#[derive(Serialize)]
pub(crate) struct TransactionWithMetadata<T: Serialize, N: Network> {
    transaction: T,
    block_hash: Option<N::BlockHash>,
    block_height: Option<u32>,
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
    /// Validators can either sync in "fast" mode, by fetching blocks similar to how clients do,
    /// or the can sync certificates in "dag" mode.
    bft_sync_mode: Option<&'a str>,
    /// The block height of the CDN (if connected to a CDN).
    cdn_height: Option<u32>,
    /// The greatest known block height of a peer.
    /// None, if no peers are connected yet.
    p2p_height: Option<u32>,
    /// The number of outstanding p2p sync requests.
    outstanding_block_requests: usize,
    /// The current sync speed in blocks per second.
    sync_speed_bps: f64,
}

impl<N: Network, C: ConsensusStorage<N>, R: Routing<N>> Rest<N, C, R> {
    /// GET /<network>/version
    pub(crate) async fn get_version() -> ErasedJson {
        ErasedJson::pretty(VersionInfo::get::<N>())
    }

    /// Get /<network>/consensus_version
    pub(crate) async fn get_consensus_version(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(N::CONSENSUS_VERSION(rest.ledger.latest_height())? as u16))
    }

    /// GET /<network>/block/height/latest
    pub(crate) async fn get_block_height_latest(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.ledger.latest_height())
    }

    /// GET /<network>/block/hash/latest
    pub(crate) async fn get_block_hash_latest(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.ledger.latest_hash())
    }

    /// GET /<network>/block/latest
    pub(crate) async fn get_block_latest(State(rest): State<Self>) -> ErasedJson {
        let block = rest.ledger.latest_block();
        let hash = block.hash();
        // When present, this is 3x faster than serializing the block from the ledger.
        rest.block_cache.lock().get_or_insert(hash, || ErasedJson::pretty(block)).clone()
    }

    /// GET /<network>/block/{height}
    /// GET /<network>/block/{blockHash}
    pub(crate) async fn get_block(
        State(rest): State<Self>,
        Path(height_or_hash): Path<String>,
    ) -> Result<ErasedJson, RestError> {
        // Manually parse the height or the height of the hash, axum doesn't support different types
        // for the same path param.
        let hash = if let Ok(height) = height_or_hash.parse::<u32>() {
            rest.ledger.get_hash(height).with_context(|| "Failed to get a block's hash")?
        } else if let Ok(hash) = height_or_hash.parse::<N::BlockHash>() {
            hash
        } else {
            return Err(RestError::bad_request(anyhow!("invalid input: neither a block height nor a block hash")));
        };

        // Attempt to find a serialized block in the cache.
        if let Some(json_block) = rest.block_cache.lock().get(&hash) {
            return Ok(json_block.clone());
        }

        // Retrieve the block from the database.
        let json_block = match tokio::task::spawn_blocking(move || match rest.ledger.try_get_block_by_hash(&hash) {
            Ok(Some(block)) => Some(ErasedJson::pretty(block)),
            Ok(None) => None,
            Err(e) => {
                error!("Couldn't find a block: {e}");
                None
            }
        })
        .await
        {
            Ok(Some(block)) => Ok(block),
            Ok(None) => Err(RestError::not_found(anyhow!("Couldn't find block {height_or_hash}"))),
            Err(e) => Err(RestError::internal_server_error(anyhow!("tokio error: {e}"))),
        }?;

        rest.block_cache.lock().put(hash, json_block.clone());

        Ok(json_block)
    }

    /// GET /<network>/blocks?start={start_height}&end={end_height}
    pub(crate) async fn get_blocks(
        State(rest): State<Self>,
        Query(block_range): Query<BlockRange>,
    ) -> Result<ErasedJson, RestError> {
        let start_height = block_range.start;
        let end_height = block_range.end;

        const MAX_BLOCK_RANGE: u32 = 50;

        // Ensure the end height is greater than the start height.
        if start_height > end_height {
            return Err(RestError::bad_request(anyhow!("Invalid block range")));
        }

        // Ensure the block range is bounded.
        if end_height - start_height > MAX_BLOCK_RANGE {
            return Err(RestError::bad_request(anyhow!(
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
            Err(err) => {
                let err: anyhow::Error = err.into();

                Err(RestError::internal_server_error(
                    err.context(format!("Failed to get blocks '{start_height}..{end_height}'")),
                ))
            }
        }
    }

    /// GET /<network>/sync/status
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

        let bft_sync_mode = rest.block_sync.get_bft_sync_mode().map(|mode| match mode {
            BftSyncMode::Fast => "fast",
            BftSyncMode::Dag => "dag",
        });

        Ok(ErasedJson::pretty(SyncStatus {
            sync_mode,
            bft_sync_mode,
            cdn_height,
            is_synced: !cdn_sync && rest.routing.is_block_synced(),
            ledger_height: rest.ledger.latest_height(),
            p2p_height: rest.block_sync.greatest_peer_block_height(),
            outstanding_block_requests: rest.block_sync.num_outstanding_block_requests(),
            sync_speed_bps: rest.block_sync.get_sync_speed(),
        }))
    }

    /// GET /<network>/sync/peers
    pub(crate) async fn get_sync_peers(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        let peers: HashMap<String, u32> =
            rest.block_sync.get_peer_heights().into_iter().map(|(addr, height)| (addr.to_string(), height)).collect();
        Ok(ErasedJson::pretty(peers))
    }

    /// GET /<network>/sync/requests
    pub(crate) async fn get_sync_requests_summary(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        let summary = rest.block_sync.get_block_requests_summary();
        Ok(ErasedJson::pretty(summary))
    }

    /// GET /<network>/sync/requests/list
    pub(crate) async fn get_sync_requests_list(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        let requests = rest.block_sync.get_block_requests_info();
        Ok(ErasedJson::pretty(requests))
    }

    /// GET /<network>/height/{blockHash}
    pub(crate) async fn get_height(
        State(rest): State<Self>,
        Path(hash): Path<N::BlockHash>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_height(&hash)?))
    }

    /// GET /<network>/block/{height}/header
    pub(crate) async fn get_block_header(
        State(rest): State<Self>,
        Path(height): Path<u32>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_header(height)?))
    }

    /// GET /<network>/block/{height}/transactions
    pub(crate) async fn get_block_transactions(
        State(rest): State<Self>,
        Path(height): Path<u32>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_transactions(height)?))
    }

    /// GET /<network>/transaction/{transactionID}
    /// GET /<network>/transaction/{transactionID}?metadata={true}
    pub(crate) async fn get_transaction(
        State(rest): State<Self>,
        Path(tx_id): Path<N::TransactionID>,
        metadata: Query<Metadata>,
    ) -> Result<ErasedJson, RestError> {
        // Ledger returns a generic anyhow::Error, so checking the message is the only way to parse it.
        let transaction = rest.ledger.get_transaction(tx_id).map_err(|err| {
            if err.to_string().contains("Missing") { RestError::not_found(err) } else { RestError::from(err) }
        })?;
        // Check if metadata is requested and return the transaction with metadata if so.
        if metadata.metadata.unwrap_or(false) {
            // Get the block hash and height for the transaction, if it exists.
            let block_hash = rest.ledger.find_block_hash(&tx_id).ok().flatten();
            let block_height = block_hash.and_then(|hash| rest.ledger.get_height(&hash).ok());
            Ok(ErasedJson::pretty(TransactionWithMetadata::<_, N> { transaction, block_hash, block_height }))
        } else {
            Ok(ErasedJson::pretty(transaction))
        }
    }

    /// GET /<network>/transaction/confirmed/{transactionID}
    /// GET /<network>/transaction/confirmed/{transactionID}?metadata={true}
    pub(crate) async fn get_confirmed_transaction(
        State(rest): State<Self>,
        Path(tx_id): Path<N::TransactionID>,
        metadata: Query<Metadata>,
    ) -> Result<ErasedJson, RestError> {
        // Ledger returns a generic anyhow::Error, so checking the message is the only way to parse it.
        let transaction = rest.ledger.get_confirmed_transaction(tx_id).map_err(|err| {
            if err.to_string().contains("Missing") { RestError::not_found(err) } else { RestError::from(err) }
        })?;
        // Check if metadata is requested and return the transaction with metadata if so.
        if metadata.metadata.unwrap_or(false) {
            // Get the block hash and height for the confirmed transaction.
            let block_hash = rest
                .ledger
                .find_block_hash(&tx_id)?
                .ok_or_else(|| anyhow!("Block hash not found for transaction {tx_id}"))?;
            let block_height = rest.ledger.get_height(&block_hash)?;
            Ok(ErasedJson::pretty(TransactionWithMetadata::<_, N> {
                transaction,
                block_hash: Some(block_hash),
                block_height: Some(block_height),
            }))
        } else {
            Ok(ErasedJson::pretty(transaction))
        }
    }

    /// GET /<network>/transaction/unconfirmed/{transactionID}
    pub(crate) async fn get_unconfirmed_transaction(
        State(rest): State<Self>,
        Path(tx_id): Path<N::TransactionID>,
    ) -> Result<ErasedJson, RestError> {
        // Ledger returns a generic anyhow::Error, so checking the message is the only way to parse it.
        Ok(ErasedJson::pretty(rest.ledger.get_unconfirmed_transaction(&tx_id).map_err(|err| {
            if err.to_string().contains("Missing") { RestError::not_found(err) } else { RestError::from(err) }
        })?))
    }

    /// GET /<network>/memoryPool/transmissions
    pub(crate) async fn get_memory_pool_transmissions(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        match rest.consensus {
            Some(consensus) => {
                Ok(ErasedJson::pretty(consensus.unconfirmed_transmissions().collect::<IndexMap<_, _>>()))
            }
            None => Err(RestError::service_unavailable(anyhow!("Route isn't available for this node type"))),
        }
    }

    /// GET /<network>/memoryPool/solutions
    pub(crate) async fn get_memory_pool_solutions(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        match rest.consensus {
            Some(consensus) => Ok(ErasedJson::pretty(consensus.unconfirmed_solutions().collect::<IndexMap<_, _>>())),
            None => Err(RestError::service_unavailable(anyhow!("Route isn't available for this node type"))),
        }
    }

    /// GET /<network>/memoryPool/transactions
    pub(crate) async fn get_memory_pool_transactions(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        match rest.consensus {
            Some(consensus) => Ok(ErasedJson::pretty(consensus.unconfirmed_transactions().collect::<IndexMap<_, _>>())),
            None => Err(RestError::service_unavailable(anyhow!("Route isn't available for this node type"))),
        }
    }

    /// GET /<network>/program/{programID}
    /// GET /<network>/program/{programID}?metadata={true}
    pub(crate) async fn get_program(
        State(rest): State<Self>,
        Path(id): Path<ProgramID<N>>,
        metadata: Query<Metadata>,
    ) -> Result<ErasedJson, RestError> {
        // Get the program from the ledger.
        let program = rest.ledger.get_program(id).with_context(|| format!("Failed to find program `{id}`"))?;
        // Check if metadata is requested and return the program with metadata if so.
        if metadata.metadata.unwrap_or(false) {
            // Get the edition of the program.
            let edition = rest.ledger.get_latest_edition_for_program(&id)?;
            return rest.return_program_with_metadata(program, edition);
        }
        // Return the program without metadata.
        Ok(ErasedJson::pretty(program))
    }

    /// GET /<network>/program/{programID}/{edition}
    /// GET /<network>/program/{programID}/{edition}?metadata={true}
    pub(crate) async fn get_program_for_edition(
        State(rest): State<Self>,
        Path((id, edition)): Path<(ProgramID<N>, u16)>,
        metadata: Query<Metadata>,
    ) -> Result<ErasedJson, RestError> {
        // Get the program from the ledger.
        match rest
            .ledger
            .try_get_program_for_edition(&id, edition)
            .with_context(|| format!("Failed get program `{id}` for edition {edition}"))?
        {
            Some(program) => {
                // Check if metadata is requested and return the program with metadata if so.
                if metadata.metadata.unwrap_or(false) {
                    rest.return_program_with_metadata(program, edition)
                } else {
                    Ok(ErasedJson::pretty(program))
                }
            }
            None => Err(RestError::not_found(anyhow!("No program `{id}` exists for edition {edition}"))),
        }
    }

    /// A helper function to return the program and its metadata.
    /// This function is used in the `get_program` and `get_program_for_edition` functions.
    fn return_program_with_metadata(&self, program: Program<N>, edition: u16) -> Result<ErasedJson, RestError> {
        let id = program.id();
        // Get the transaction ID associated with the program and edition.
        let tx_id = self.ledger.find_latest_transaction_id_from_program_id_and_edition(id, edition)?;
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
        // Get the amendment count for this program and edition.
        let amendment_count =
            self.ledger.vm().block_store().transaction_store().get_amendment_count(id, edition)?.unwrap_or(0);
        Ok(ErasedJson::pretty(json!({
            "program": program,
            "edition": edition,
            "transaction_id": tx_id,
            "program_owner": program_owner,
            "amendment_count": amendment_count,
        })))
    }

    /// GET /<network>/program/{programID}/latest_edition
    pub(crate) async fn get_latest_program_edition(
        State(rest): State<Self>,
        Path(id): Path<ProgramID<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_latest_edition_for_program(&id)?))
    }

    /// GET /<network>/program/{programID}/mappings
    pub(crate) async fn get_mapping_names(
        State(rest): State<Self>,
        Path(id): Path<ProgramID<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.vm().finalize_store().get_mapping_names_confirmed(&id)?))
    }

    /// GET /<network>/program/{programID}/mapping/{mappingName}/{mappingKey}
    /// GET /<network>/program/{programID}/mapping/{mappingName}/{mappingKey}?metadata={true}
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

    /// GET /<network>/program/{programID}/mapping/{mappingName}?all={true}&metadata={true}
    pub(crate) async fn get_mapping_values(
        State(rest): State<Self>,
        Path((id, name)): Path<(ProgramID<N>, Identifier<N>)>,
        metadata: Query<Metadata>,
    ) -> Result<ErasedJson, RestError> {
        // Return an error if the `all` query parameter is not set to `true`.
        if metadata.all != Some(true) {
            return Err(RestError::bad_request(anyhow!(
                "Invalid query parameter. At this time, 'all=true' must be included"
            )));
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
            Ok(Err(err)) => Err(RestError::internal_server_error(err.context("Unable to read mapping"))),
            Err(err) => Err(RestError::internal_server_error(anyhow!("Tokio error: {err}"))),
        }
    }

    /// GET /<network>/program/{programID}/amendment_count
    pub(crate) async fn get_program_amendment_count(
        State(rest): State<Self>,
        Path(id): Path<ProgramID<N>>,
    ) -> Result<ErasedJson, RestError> {
        // Get the latest edition.
        let edition = rest.ledger.get_latest_edition_for_program(&id)?;
        // Get the amendment count for this program and edition.
        let amendment_count =
            rest.ledger.vm().block_store().transaction_store().get_amendment_count(&id, edition)?.unwrap_or(0);

        Ok(ErasedJson::pretty(json!({
            "program_id": id,
            "edition": edition,
            "amendment_count": amendment_count,
        })))
    }

    /// GET /<network>/program/{programID}/{edition}/amendment_count
    pub(crate) async fn get_program_amendment_count_for_edition(
        State(rest): State<Self>,
        Path((id, edition)): Path<(ProgramID<N>, u16)>,
    ) -> Result<ErasedJson, RestError> {
        // Get the amendment count for this program and edition.
        let amendment_count =
            rest.ledger.vm().block_store().transaction_store().get_amendment_count(&id, edition)?.unwrap_or(0);

        Ok(ErasedJson::pretty(json!({
            "program_id": id,
            "edition": edition,
            "amendment_count": amendment_count,
        })))
    }

    /// GET /<network>/statePath/{commitment}
    pub(crate) async fn get_state_path_for_commitment(
        State(rest): State<Self>,
        Path(commitment): Path<Field<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_state_path_for_commitment(&commitment)?))
    }

    /// GET /<network>/statePaths?commitments=cm1,cm2,...
    pub(crate) async fn get_state_paths_for_commitments(
        State(rest): State<Self>,
        Query(commitments): Query<Commitments>,
    ) -> Result<ErasedJson, RestError> {
        // Retrieve the number of commitments.
        let num_commitments = commitments.commitments.len();
        // Return an error if no commitments are provided.
        if num_commitments == 0 {
            return Err(RestError::unprocessable_entity(anyhow!("No commitments provided")));
        }
        // Return an error if the number of commitments exceeds the maximum allowed.
        if num_commitments > N::MAX_INPUTS {
            return Err(RestError::unprocessable_entity(anyhow!(format!(
                "Too many commitments provided (max: {}, got: {num_commitments})",
                N::MAX_INPUTS
            ))));
        }

        // Deserialize the commitments from the query.
        let commitments = match tokio::task::spawn_blocking(move || {
            commitments
                .commitments
                .iter()
                .map(|s| {
                    s.parse::<Field<N>>()
                        .map_err(|err| RestError::unprocessable_entity(err.context(format!("Invalid commitment: {s}"))))
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .await
        {
            Ok(Ok(commitments)) => commitments,
            Ok(Err(err)) => {
                return Err(RestError::internal_server_error(anyhow!(err).context("Unable to parse commitments")));
            }
            Err(err) => return Err(RestError::internal_server_error(anyhow!(err).context("Tokio error"))),
        };

        Ok(ErasedJson::pretty(rest.ledger.get_state_paths_for_commitments(&commitments)?))
    }

    /// GET /<network>/stateRoot/latest
    pub(crate) async fn get_state_root_latest(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.ledger.latest_state_root())
    }

    /// GET /<network>/stateRoot/{height}
    pub(crate) async fn get_state_root(
        State(rest): State<Self>,
        Path(height): Path<u32>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_state_root(height)?))
    }

    /// GET /<network>/committee/latest
    pub(crate) async fn get_committee_latest(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.latest_committee()?))
    }

    /// GET /<network>/committee/{height}
    pub(crate) async fn get_committee(
        State(rest): State<Self>,
        Path(height): Path<u32>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.get_committee(height)?))
    }

    /// GET /<network>/delegators/{validator}
    pub(crate) async fn get_delegators_for_validator(
        State(rest): State<Self>,
        Path(validator): Path<Address<N>>,
    ) -> Result<ErasedJson, RestError> {
        // Do not process the request if the node is too far behind to avoid sending outdated data.
        if !rest.routing.is_within_sync_leniency() {
            return Err(RestError::service_unavailable(anyhow!("Unable to request delegators (node is syncing)")));
        }

        // Return the delegators for the given validator.
        match tokio::task::spawn_blocking(move || rest.ledger.get_delegators_for_validator(&validator)).await {
            Ok(Ok(delegators)) => Ok(ErasedJson::pretty(delegators)),
            Ok(Err(err)) => Err(RestError::internal_server_error(err.context("Unable to request delegators"))),
            Err(err) => Err(RestError::internal_server_error(anyhow!(err).context("Tokio error"))),
        }
    }

    /// GET /<network>/peers/count (alias: /connections/p2p/count)
    pub(crate) async fn get_peers_count(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.routing.router().number_of_connected_peers())
    }

    /// GET /<network>/peers/all (alias: /connections/p2p/all)
    pub(crate) async fn get_peers_all(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.routing.router().connected_peers())
    }

    /// GET /<network>/peers/all/metrics (alias: /connections/p2p/all/metrics)
    pub(crate) async fn get_peers_all_metrics(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.routing.router().connected_metrics())
    }

    /// GET /<network>/connections/bft/count
    pub(crate) async fn get_bft_connections_count(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        match rest.consensus {
            Some(consensus) => Ok(ErasedJson::pretty(consensus.bft().primary().gateway().number_of_connected_peers())),
            None => Err(RestError::service_unavailable(anyhow!("Route isn't available for this node type"))),
        }
    }

    /// GET /<network>/connections/bft/all
    pub(crate) async fn get_bft_connections_all(State(rest): State<Self>) -> Result<ErasedJson, RestError> {
        match rest.consensus {
            Some(consensus) => Ok(ErasedJson::pretty(consensus.bft().primary().gateway().connected_peers())),
            None => Err(RestError::service_unavailable(anyhow!("Route isn't available for this node type"))),
        }
    }

    /// GET /<network>/node/address
    pub(crate) async fn get_node_address(State(rest): State<Self>) -> ErasedJson {
        ErasedJson::pretty(rest.routing.router().address())
    }

    /// GET /<network>/find/blockHash/{transactionID}
    pub(crate) async fn find_block_hash(
        State(rest): State<Self>,
        Path(tx_id): Path<N::TransactionID>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_block_hash(&tx_id)?))
    }

    /// GET /<network>/find/blockHeight/{stateRoot}
    pub(crate) async fn find_block_height_from_state_root(
        State(rest): State<Self>,
        Path(state_root): Path<N::StateRoot>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_block_height_from_state_root(state_root)?))
    }

    /// GET /<network>/find/transactionID/deployment/{programID}
    pub(crate) async fn find_latest_transaction_id_from_program_id(
        State(rest): State<Self>,
        Path(program_id): Path<ProgramID<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_latest_transaction_id_from_program_id(&program_id)?))
    }

    /// GET /<network>/find/transactionID/deployment/{programID}/{edition}
    pub(crate) async fn find_latest_transaction_id_from_program_id_and_edition(
        State(rest): State<Self>,
        Path((program_id, edition)): Path<(ProgramID<N>, u16)>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(
            rest.ledger.find_latest_transaction_id_from_program_id_and_edition(&program_id, edition)?,
        ))
    }

    /// GET /<network>/find/transactionID/deployment/{programID}/{edition}/original
    /// Finds the transaction ID for the original deployment (not an amendment).
    pub(crate) async fn find_original_deployment_transaction_id(
        State(rest): State<Self>,
        Path((program_id, edition)): Path<(ProgramID<N>, u16)>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(
            rest.ledger.find_original_transaction_id_from_program_id_and_edition(&program_id, edition)?,
        ))
    }

    /// GET /<network>/find/transactionID/deployment/{programID}/{edition}/{amendment}
    /// Finds the transaction ID for an amendment deployment at the specified index.
    pub(crate) async fn find_transaction_id_from_program_id_edition_and_amendment(
        State(rest): State<Self>,
        Path((program_id, edition, amendment)): Path<(ProgramID<N>, u16, u64)>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_transaction_id_from_program_id_edition_and_amendment(
            &program_id,
            edition,
            amendment,
        )?))
    }

    /// GET /<network>/find/transactionID/{transitionID}
    pub(crate) async fn find_transaction_id_from_transition_id(
        State(rest): State<Self>,
        Path(transition_id): Path<N::TransitionID>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_transaction_id_from_transition_id(&transition_id)?))
    }

    /// GET /<network>/find/transitionID/{inputOrOutputID}
    pub(crate) async fn find_transition_id(
        State(rest): State<Self>,
        Path(input_or_output_id): Path<Field<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(rest.ledger.find_transition_id(&input_or_output_id)?))
    }

    /// POST /<network>/transaction/broadcast
    /// POST /<network>/transaction/broadcast?check_transaction={true}
    ///
    /// Transaction Broadcast Flow
    ///
    /// /transaction/broadcast
    ///         |
    ///    +----+---------------------------+
    ///    |                               |
    ///    v                               v
    /// Without Query Params        With Query Param
    ///                                check_transaction=true
    ///    |                               |
    ///    +---------+                     +---------+
    ///    |         |                     |         |
    ///    v         v                     v         v
    /// Synced   Not Synced            Synced   Not Synced
    ///    |         |                     |         |
    ///    v         v                     v         v
    ///   200       200        check_transaction  check_transaction
    ///                           +---------+        +---------+
    ///                           |         |        |         |
    ///                           v         v        v         v
    ///                          200       422      203       503
    pub(crate) async fn transaction_broadcast(
        State(rest): State<Self>,
        check_transaction: Query<CheckTransaction>,
        json_result: Result<Json<Transaction<N>>, JsonRejection>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        let Json(tx) = match json_result {
            Ok(json) => json,
            Err(JsonRejection::JsonDataError(err)) => {
                // For JsonDataError, return 422 to let transaction validation handle it
                return Err(RestError::unprocessable_entity(anyhow!("Invalid transaction data: {err}")));
            }
            Err(other_rejection) => return Err(other_rejection.into()),
        };

        // If the transaction exceeds the transaction size limit, return an error.
        // The buffer is initially roughly sized to hold a `transfer_public`,
        // most transactions will be smaller and this reduces unnecessary allocations.
        // TODO: Should this be a blocking task?
        let buffer = Vec::with_capacity(3000);
        if tx.write_le(LimitedWriter::new(buffer, N::LATEST_MAX_TRANSACTION_SIZE())).is_err() {
            return Err(RestError::bad_request(anyhow!("Transaction size exceeds the byte limit")));
        }

        // Prepare the unconfirmed transaction message.
        let tx_id = tx.id();
        let message = Message::UnconfirmedTransaction(UnconfirmedTransaction {
            transaction_id: tx_id,
            transaction: Data::Object(tx.clone()),
        });

        // Check if the node is within sync leniency.
        let is_within_sync_leniency = rest.routing.is_within_sync_leniency();

        // Determine if we need to check the transaction.
        let check_transaction = check_transaction.check_transaction.unwrap_or(false);

        if check_transaction {
            // Select the semaphore based on the transaction type.
            let (slot, err_msg) = if tx.is_execute() {
                (rest.num_verifying_executions.acquire().await, "Too many execution verifications in progress")
            } else {
                (rest.num_verifying_deploys.acquire().await, "Too many deploy verifications in progress")
            };

            if slot.is_err() {
                return Err(RestError::too_many_requests(anyhow!("{err_msg}")));
            }

            // Perform the check.
            let res = rest.ledger.check_transaction_basic(&tx, None, &mut rand::rng()).map_err(|err| {
                match is_within_sync_leniency {
                    // The transaction failed to verify.
                    true => RestError::unprocessable_entity(err.context("Invalid transaction")),
                    // The node is out of sync and may not be able to properly validate the transaction.
                    false => {
                        RestError::service_unavailable(err.context("Unable to validate transaction (node is syncing)"))
                    }
                }
            });
            // Propagate error if any.
            res?;
        }

        // If the consensus module is enabled, add the unconfirmed transaction to the memory pool.
        if let Some(consensus) = rest.consensus {
            // Add the unconfirmed transaction to the memory pool.
            consensus.add_unconfirmed_transaction(tx.clone()).await?;
        }

        // Broadcast the transaction.
        rest.routing.propagate(message, &[]);

        // Determine if the node is synced and if the transaction was checked.
        match !is_within_sync_leniency && check_transaction {
            // If the node is not synced and we validated the transaction, return a 203.
            true => Ok((StatusCode::NON_AUTHORITATIVE_INFORMATION, ErasedJson::pretty(tx_id))),
            // Otherwise, return a 200.
            false => Ok((StatusCode::OK, ErasedJson::pretty(tx_id))),
        }
    }

    /// POST /<network>/solution/broadcast
    /// POST /<network>/solution/broadcast?check_solution={true}
    ///
    /// Solution Broadcast Flow
    ///
    /// /solution/broadcast
    ///         |
    ///    +----+---------------------------+
    ///    |                               |
    ///    v                               v
    /// Without Query Params        With Query Param
    ///                                check_solution=true
    ///    |                               |
    ///    +---------+                     +---------+
    ///    |         |                     |         |
    ///    v         v                     v         v
    /// Synced   Not Synced            Synced   Not Synced
    ///    |         |                     |         |
    ///    v         v                     v         v
    ///   200       200        check_solution        check_solution
    ///                           +---------+        +---------+
    ///                           |         |        |         |
    ///                           v         v        v         v
    ///                          200       422      203       503
    pub(crate) async fn solution_broadcast(
        State(rest): State<Self>,
        check_solution: Query<CheckSolution>,
        Json(solution): Json<Solution<N>>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        // Check if the node is within sync leniency.
        let is_within_sync_leniency = rest.routing.is_within_sync_leniency();
        // Determine if we need to check the solution.
        let check_solution = check_solution.check_solution.unwrap_or(false);
        // Check if the prover has reached their solution limit.
        // While snarkVM will ultimately abort any excess solutions for safety, performing this check
        // here prevents the to-be aborted solutions from propagating through the network.
        let prover_address = solution.address();
        if rest.ledger.is_solution_limit_reached(&prover_address, 0) {
            return Err(RestError::unprocessable_entity(anyhow!(
                "Invalid solution '{}' - Prover '{prover_address}' has reached their solution limit for the current epoch",
                fmt_id(solution.id())
            )));
        }

        if check_solution {
            // Try to acquire a slot.
            let slot = rest.num_verifying_solutions.acquire().await;
            if slot.is_err() {
                return Err(RestError::too_many_requests(anyhow!("Too many solution verifications in progress")));
            }

            // Compute the current epoch hash.
            let epoch_hash = rest.ledger.latest_epoch_hash()?;
            // Retrieve the current proof target.
            let proof_target = rest.ledger.latest_proof_target();
            // Ensure that the solution is valid for the given epoch.
            let puzzle = rest.ledger.puzzle().clone();
            // Verify the solution in a blocking task.
            let res: Result<(), anyhow::Error> =
                match tokio::task::spawn_blocking(move || puzzle.check_solution(&solution, epoch_hash, proof_target))
                    .await
                {
                    Ok(Ok(())) => Ok(()),
                    Ok(Err(err)) => {
                        return match is_within_sync_leniency {
                            // The solution failed to verify.
                            true => Err(RestError::unprocessable_entity(
                                err.context(format!("Invalid solution '{}'", fmt_id(solution.id()))),
                            )),
                            // The node is out of sync and may not be able to properly validate the solution.
                            false => Err(RestError::service_unavailable(anyhow!(
                                "Unable to validate solution '{}' (node is syncing)",
                                fmt_id(solution.id())
                            ))),
                        };
                    }
                    Err(err) => {
                        return Err(RestError::internal_server_error(anyhow!("Tokio error: {err}")));
                    }
                };
            // Propagate error if any.
            res?;
        }

        // If the consensus module is enabled, add the unconfirmed solution to the memory pool.
        if let Some(consensus) = rest.consensus {
            // Add the unconfirmed solution to the memory pool.
            let _ = consensus.add_unconfirmed_solution(solution).await;
        }

        let solution_id = solution.id();
        // Prepare the unconfirmed solution message.
        let message =
            Message::UnconfirmedSolution(UnconfirmedSolution { solution_id, solution: Data::Object(solution) });

        // Broadcast the unconfirmed solution message.
        rest.routing.propagate(message, &[]);

        // Determine if the node is synced and if the solution was checked.
        match !is_within_sync_leniency && check_solution {
            // If the node is not synced and we validated the solution, return a 203.
            true => Ok((StatusCode::NON_AUTHORITATIVE_INFORMATION, ErasedJson::pretty(solution_id))),
            // Otherwise, return a 200.
            false => Ok((StatusCode::OK, ErasedJson::pretty(solution_id))),
        }
    }

    /// POST /{network}/db_backup?path=new_fs_path
    pub(crate) async fn db_backup(
        State(rest): State<Self>,
        backup_path: Query<BackupPath>,
    ) -> Result<ErasedJson, RestError> {
        // Create a checkpoint at the given location.
        let mut backup_path = backup_path.path.clone();
        rest.ledger.backup_database(&backup_path)?;

        // Dump the block tree.
        let ret = ErasedJson::pretty(());
        if let Err(e) = rest.ledger.cache_block_tree() {
            warn!("Couldn't cache the block tree for a ledger checkpoint: {e}");
            return Ok(ret);
        }

        // Copy the block tree file to the new checkpoint.
        let mut block_tree_path = aleo_ledger_dir(N::ID, rest.ledger.vm().block_store().storage_mode());
        block_tree_path.push("block_tree");
        backup_path.push("block_tree");
        if let Err(e) = fs::copy(block_tree_path, backup_path) {
            warn!("Couldn't copy the block tree file to a ledger checkpoint: {e}");
        }

        Ok(ret)
    }

    /// GET /<network>/solution/limits/{prover_address}
    pub(crate) async fn get_solution_limits_for_prover(
        State(rest): State<Self>,
        Path(prover_address): Path<Address<N>>,
    ) -> Result<ErasedJson, RestError> {
        Ok(ErasedJson::pretty(json!({
            "is_limit_reached": rest.ledger.is_solution_limit_reached(&prover_address, 0),
            "num_remaining_solutions": rest.ledger.num_remaining_solutions(&prover_address, 0),
            "latest_epoch_hash": rest.ledger.latest_epoch_hash()?,
            "blocks_until_next_epoch": N::NUM_BLOCKS_PER_EPOCH.saturating_sub(rest.ledger.latest_height() % N::NUM_BLOCKS_PER_EPOCH),
        })))
    }

    /// GET /{network}/program/{id}/mapping/{name}/{key}/history/{height}
    #[cfg(feature = "history")]
    pub(crate) async fn get_history(
        State(rest): State<Self>,
        Path((program_id, mapping_name, mapping_key, height)): Path<HistoricalMappingKey<N>>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        // Retrieve the history for the given block height and variant.
        let value = rest.ledger.vm().finalize_store().get_historical_mapping_value(program_id, mapping_name, mapping_key.clone(), height)
            .map_err(|err| {
                RestError::not_found(err.context(format!("Could not load mapping '{mapping_name}/{mapping_key}' for program '{program_id}' from block '{height}'")))
            })?;

        Ok((StatusCode::OK, ErasedJson::pretty(value)))
    }

    /// GET /{network}/program/{id}/mapping/{name}/history/{height}?keys=key1,key2,...
    #[cfg(feature = "history")]
    pub(crate) async fn get_history_batch(
        State(rest): State<Self>,
        Path((program_id, mapping_name, height)): Path<HistoricalMappingRoute<N>>,
        Query(historical_keys): Query<HistoricalKeys>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        let mapping_keys = parse_historical_mapping_keys::<N>(&historical_keys.keys)?;

        let values = match tokio::task::spawn_blocking(move || cfg_into_iter!(historical_keys
            .keys)
            .zip(mapping_keys)
            .map(|(key, mapping_key)| {
                let value = rest
                    .ledger
                    .vm()
                    .finalize_store()
                    .get_historical_mapping_value(program_id, mapping_name, mapping_key, height)
                    .map_err(|err| {
                        RestError::not_found(err.context(format!(
                            "Could not load mapping '{mapping_name}/{key}' for program '{program_id}' from block '{height}'"
                        )))
                    })?;

                Ok(json!({ "key": key, "value": value }))
            })
            .collect::<Result<Vec<_>, RestError>>())
            .await {
                Ok(Ok(values)) => values,
                Ok(Err(err)) => return Err(RestError::internal_server_error(anyhow!(err).context("Unable to get historical mapping values"))),
                Err(err) => return Err(RestError::internal_server_error(anyhow!("Tokio error: {err}"))),
            };

        Ok((StatusCode::OK, ErasedJson::pretty(values)))
    }

    /// POST /{network}/program/{id}/view/{functionName}/{height}
    ///
    /// Evaluates a view function against the ledger state at the given block `height`.
    /// The request body must be a JSON array of string-encoded inputs, e.g.:
    ///
    /// ```json
    /// ["aleo1...", "10u64"]
    /// ```
    ///
    /// Returns the outputs as a JSON array of string-encoded values.
    #[cfg(feature = "history")]
    pub(crate) async fn evaluate_view(
        State(rest): State<Self>,
        Path((program_id, view_name, height)): Path<ViewFunctionRoute<N>>,
        json_result: Result<Json<Vec<String>>, JsonRejection>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        // Parse the inputs from the request body.
        let Json(raw_inputs) = match json_result {
            Ok(json) => json,
            Err(err) => return Err(RestError::unprocessable_entity(anyhow!("Invalid request body: {err}"))),
        };

        // Parse the inputs into `Value<N>`.
        let inputs = parse_view_inputs::<N>(&raw_inputs)?;

        // Evaluate the view function in a blocking task.
        let outputs = match tokio::task::spawn_blocking(move || {
            rest.ledger.vm().evaluate_view_at_height(program_id, view_name, inputs, height)
        })
        .await
        {
            Ok(Ok(outputs)) => outputs,
            Ok(Err(err)) => {
                return Err(RestError::bad_request(
                    err.context(format!("Failed to evaluate view '{view_name}' for '{program_id}' at height {height}")),
                ));
            }
            Err(err) => return Err(RestError::internal_server_error(anyhow!("Tokio error: {err}"))),
        };

        // Encode each output as a string.
        let output_strings: Vec<String> = outputs.iter().map(|v| v.to_string()).collect();

        Ok((StatusCode::OK, ErasedJson::pretty(output_strings)))
    }

    /// POST /{network}/program/{id}/view/{functionName}
    ///
    /// Evaluates a view function against the ledger state at the latest block height.
    /// The request body must be a JSON array of string-encoded inputs, e.g.:
    ///
    /// ```json
    /// ["aleo1...", "10u64"]
    /// ```
    ///
    /// Returns the outputs as a JSON array of string-encoded values.
    /// Optionally, append `?metadata=true` to also return the block height at which the
    /// view was evaluated (same semantics as the mapping-read endpoints).
    pub(crate) async fn evaluate_view_latest(
        State(rest): State<Self>,
        Path((program_id, view_name)): Path<(ProgramID<N>, Identifier<N>)>,
        metadata: Query<Metadata>,
        json_result: Result<Json<Vec<String>>, JsonRejection>,
    ) -> Result<ErasedJson, RestError> {
        // Parse the inputs from the request body.
        let Json(raw_inputs) = match json_result {
            Ok(json) => json,
            Err(err) => return Err(RestError::unprocessable_entity(anyhow!("Invalid request body: {err}"))),
        };

        // Parse the inputs into `Value<N>`.
        let inputs = parse_view_inputs::<N>(&raw_inputs)?;

        // Evaluate the view function in a blocking task.
        // The latest block's state is captured inside the task to minimise the window
        // between state sampling and evaluation.
        let (outputs, height) = match tokio::task::spawn_blocking(move || {
            // Capture the latest block to build a consistent `FinalizeGlobalState`.
            let block = rest.ledger.latest_block();
            let height = block.height();

            // Reconstruct the `FinalizeGlobalState` for the latest block. The block timestamp
            // is only included from `ConsensusVersion::V12` onward, matching the consensus path.
            let block_timestamp =
                (height >= N::CONSENSUS_HEIGHT(ConsensusVersion::V12).unwrap_or_default()).then_some(block.timestamp());
            let state = FinalizeGlobalState::new::<N>(
                block.round(),
                height,
                block_timestamp,
                block.cumulative_weight(),
                block.cumulative_proof_target(),
                block.previous_hash(),
                None,
            )?;

            // Get the current (latest-edition) stack for the program.
            let stack = rest.ledger.vm().process().get_stack(program_id)?;

            // Evaluate the view against the current finalize store.
            let outputs = stack.evaluate_view(state, rest.ledger.vm().finalize_store(), &view_name, inputs)?;

            Ok::<_, anyhow::Error>((outputs, height))
        })
        .await
        {
            Ok(Ok(result)) => result,
            Ok(Err(err)) => {
                return Err(RestError::bad_request(err.context(format!(
                    "Failed to evaluate view '{view_name}' for '{program_id}' at the latest height"
                ))));
            }
            Err(err) => return Err(RestError::internal_server_error(anyhow!("Tokio error: {err}"))),
        };

        // Encode each output as a string.
        let output_strings: Vec<String> = outputs.iter().map(|v| v.to_string()).collect();

        // Check if metadata is requested and return the outputs with the sampled height if so.
        if metadata.metadata.unwrap_or(false) {
            return Ok(ErasedJson::pretty(json!({
                "data": output_strings,
                "height": height,
            })));
        }

        Ok(ErasedJson::pretty(output_strings))
    }

    /// GET /{network}/staking/rewards/{address}/{height}
    #[cfg(feature = "history-staking-rewards")]
    pub(crate) async fn get_staking_reward(
        State(rest): State<Self>,
        Path((address, height)): Path<(Address<N>, u32)>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        // Retrieve the history for the given block height and variant.
        let value = rest.ledger.vm().finalize_store().staking_rewards_map().get_confirmed(&(address, height)).map_err(
            |err| {
                RestError::not_found(
                    err.context(format!("Could not load the staking reward for {address} from block '{height}'")),
                )
            },
        )?;

        Ok((StatusCode::OK, ErasedJson::pretty(value)))
    }

    /// GET /{network}/validators/participation
    /// GET /{network}/validators/participation?metadata={true}
    #[cfg(feature = "telemetry")]
    pub(crate) async fn get_validator_participation_scores(
        State(rest): State<Self>,
        metadata: Query<Metadata>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        match rest.consensus {
            Some(consensus) => {
                // Retrieve the committee lookback for the latest round.
                let latest_round = rest.ledger.latest_round();
                let committee_lookback = rest
                    .ledger
                    .get_committee_lookback_for_round(latest_round)?
                    .ok_or_else(|| RestError::not_found(anyhow!("No committee found for round {latest_round}")))?;
                // Retrieve the latest participation scores, combining certificate and signature scores.
                let participation_scores: IndexMap<_, _> = consensus
                    .bft()
                    .primary()
                    .gateway()
                    .validator_telemetry()
                    .get_participation_scores(&committee_lookback)
                    .into_iter()
                    .map(|(address, (cert_score, sig_score))| {
                        let combined = ((0.9 * cert_score + 0.1 * sig_score) * 100.0).round() / 100.0;
                        (address, combined)
                    })
                    .collect();

                // Check if metadata is requested and return the participation scores with metadata if so.
                if metadata.metadata.unwrap_or(false) {
                    return Ok(ErasedJson::pretty(json!({
                        "participation_scores": participation_scores,
                        "height": rest.ledger.latest_height(),
                    })));
                }

                Ok(ErasedJson::pretty(participation_scores))
            }
            None => Err(RestError::service_unavailable(anyhow!("Route isn't available for this node type"))),
        }
    }

    /// GET /{network}/slipstream/plugins
    #[cfg(feature = "slipstream-plugins")]
    pub(crate) async fn slipstream_list_plugins(
        State(rest): State<Self>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        use snarkvm::slipstream_plugin_manager::slipstream_manager::SlipstreamPluginManagerError;

        let mgr_arc = rest.ledger.vm().finalize_store().slipstream_plugin_manager();
        let mgr_guard = mgr_arc.read();
        let manager = mgr_guard
            .as_ref()
            .ok_or_else(|| RestError::service_unavailable(anyhow!("No Slipstream plugin manager is installed")))?;
        let plugins = manager
            .list_plugins()
            .map_err(|e: SlipstreamPluginManagerError| RestError::internal_server_error(anyhow!(e)))?;
        Ok((StatusCode::OK, ErasedJson::pretty(plugins)))
    }

    /// POST /{network}/slipstream/plugins
    #[cfg(feature = "slipstream-plugins")]
    pub(crate) async fn slipstream_load_plugin(
        State(rest): State<Self>,
        Json(body): Json<serde_json::Value>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        use snarkvm::slipstream_plugin_manager::slipstream_manager::SlipstreamPluginManagerError;

        let config_file = body
            .get("config_file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| RestError::bad_request(anyhow!("Missing required field: config_file")))?
            .to_owned();
        let mgr_arc = rest.ledger.vm().finalize_store().slipstream_plugin_manager();
        if mgr_arc.read().is_none() {
            return Err(RestError::service_unavailable(anyhow!("No Slipstream plugin manager is installed")));
        }
        let name = tokio::task::spawn_blocking(move || -> Result<String, SlipstreamPluginManagerError> {
            // Safety: manager is set exactly once and never cleared; verified Some above.
            mgr_arc.write().as_mut().expect("plugin manager verified present").load_plugin(&config_file)
        })
        .await
        .map_err(|e| RestError::internal_server_error(anyhow!("Task join error: {e}")))?
        .map_err(|e| match e {
            SlipstreamPluginManagerError::PluginAlreadyLoaded(_) => RestError::unprocessable_entity(anyhow!("{e}")),
            other => RestError::internal_server_error(anyhow!("{other}")),
        })?;
        Ok((StatusCode::OK, ErasedJson::pretty(serde_json::json!({ "loaded": name }))))
    }

    /// DELETE /{network}/slipstream/plugins/{name}
    #[cfg(feature = "slipstream-plugins")]
    pub(crate) async fn slipstream_unload_plugin(
        State(rest): State<Self>,
        Path(name): Path<String>,
    ) -> Result<impl axum::response::IntoResponse, RestError> {
        use snarkvm::slipstream_plugin_manager::slipstream_manager::SlipstreamPluginManagerError;

        let mgr_arc = rest.ledger.vm().finalize_store().slipstream_plugin_manager();
        if mgr_arc.read().is_none() {
            return Err(RestError::service_unavailable(anyhow!("No Slipstream plugin manager is installed")));
        }
        tokio::task::spawn_blocking(move || -> anyhow::Result<()> {
            // Safety: manager is set exactly once and never cleared; verified Some above.
            mgr_arc.write().as_mut().expect("plugin manager verified present").unload_plugin(&name).map_err(
                |e: SlipstreamPluginManagerError| match e {
                    SlipstreamPluginManagerError::PluginNotLoaded(_) => anyhow!("404: {e}"),
                    other => anyhow!("{other}"),
                },
            )
        })
        .await
        .map_err(|e| RestError::internal_server_error(anyhow!("Task join error: {e}")))?
        .map_err(|e| {
            let msg = e.to_string();
            if let Some(stripped) = msg.strip_prefix("404: ") {
                RestError::not_found(anyhow!("{stripped}"))
            } else {
                RestError::internal_server_error(e)
            }
        })?;
        Ok((StatusCode::OK, ErasedJson::pretty(serde_json::json!({ "unloaded": true }))))
    }

    // TODO: PUT /{network}/slipstream/plugins/{name} (reload) is not yet implemented.
}

#[cfg(all(test, feature = "history"))]
mod tests {
    use super::*;
    use snarkvm::prelude::MainnetV0;

    #[test]
    fn parse_historical_mapping_keys_rejects_empty() {
        let err = parse_historical_mapping_keys::<MainnetV0>(&[]).unwrap_err();
        assert_eq!(err, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn parse_historical_mapping_keys_rejects_too_many() {
        let keys = vec![String::from("1field"); MAX_KEYS_PER_REQUEST + 1];
        let err = parse_historical_mapping_keys::<MainnetV0>(&keys).unwrap_err();
        assert_eq!(err, StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[test]
    fn parse_historical_mapping_keys_rejects_invalid_key_with_index() {
        let keys = vec![String::from("1field"), String::from("not_a_plaintext")];
        let err = parse_historical_mapping_keys::<MainnetV0>(&keys).unwrap_err();
        assert_eq!(err, StatusCode::UNPROCESSABLE_ENTITY);
        assert!(err.to_string().contains("Invalid key at index 1"));
    }
}
