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

use crate::helpers::{args::parse_node_data_dir, logger::initialize_terminal_logger};

use snarkos_utilities::NodeDataDir;

use snarkvm::{
    console::network::{CanaryV0, MainnetV0, Network, TestnetV0},
    ledger::{
        Ledger,
        block::Block,
        store::{
            FinalizeStore,
            helpers::rocksdb::{ConsensusDB, FinalizeDB, RocksDB},
        },
    },
    prelude::FromBytes,
};

use aleo_std::StorageMode;
use anyhow::{Context, Result, bail, ensure};
use clap::Parser;
use colored::Colorize;
use std::path::{Path, PathBuf};

/// Cleans the snarkOS node storage.
#[derive(Debug, Parser)]
pub struct Clean {
    /// Specify the network to remove from storage (0 = mainnet, 1 = testnet, 2 = canary)
    #[clap(default_value_t=MainnetV0::ID, long = "network", value_parser = clap::value_parser!(u16).range((MainnetV0::ID as i64)..=(CanaryV0::ID as i64)))]
    pub network: u16,

    /// Enables development mode, specify the unique ID of the local node to clean.
    #[clap(long)]
    pub dev: Option<u16>,

    /// Specify the path to a directory containing the ledger. Overrides the default path (also for dev).
    #[clap(long, alias = "path")]
    pub ledger_storage: Option<PathBuf>,

    /// Keep the node data directory (disabled by default).
    #[clap(long)]
    pub keep_node_data: bool,

    /// Sets a custom path for the node configuration. Overrides the default path (also for dev).
    #[clap(long, alias = "node-data-path", conflicts_with = "keep_node_data")]
    pub node_data_storage: Option<PathBuf>,

    /// Delete the indexed history and leave the ledger and node data in place.
    ///
    /// This also deletes mapping history written by storage schema v0. A later
    /// `snarkos start --history` can record a different `--history-programs` pair only after this.
    /// Combined with `--history-json`, the index is deleted and then imported.
    #[clap(long)]
    pub history: bool,

    /// Import history from the per-block JSON files data-snarkVM writes, and leave the ledger and
    /// node data in place.
    ///
    /// `DIR` is the directory that holds `group-*`. This indexes the `credits.aleo` mappings
    /// `bonded`, `delegated`, `metadata`, `unbonding`, and `withdraw`, and staking rewards, through
    /// the local tip. The files must reach the tip. A later call continues from the history cursor.
    /// A stored history for a different scope fails until `snarkos clean --history`. This cannot
    /// be used with `--dev`.
    #[clap(long, value_name = "DIR")]
    pub history_json: Option<PathBuf>,
}

impl Clean {
    /// Cleans the snarkOS node storage.
    pub fn parse(self) -> Result<String> {
        if let Some(dir) = &self.history_json {
            return self.import_history_json(dir);
        }
        if self.history {
            return self.remove_history();
        }

        // Remove the specified node configuration from storage.
        if !self.keep_node_data {
            let node_data_dir = parse_node_data_dir(&self.node_data_storage, self.network, self.dev)?;
            println!("{}", Self::remove_node_data(&node_data_dir)?);
        }

        // Remove the specified ledger from storage.
        Self::remove_ledger(self.network, &self.storage_mode())
    }

    /// Returns the ledger storage selected by `--ledger-storage` and `--dev`.
    fn storage_mode(&self) -> StorageMode {
        match &self.ledger_storage {
            Some(path) => StorageMode::Custom(path.clone()),
            None => match self.dev {
                Some(id) => StorageMode::Development(id),
                None => StorageMode::Production,
            },
        }
    }

    /// Deletes the history index in the selected ledger.
    ///
    /// The ledger blocks stay. A leftover history-replay directory from an earlier build is
    /// deleted when it is present.
    fn remove_history(&self) -> Result<String> {
        let storage_mode = self.storage_mode();
        let path = aleo_std::aleo_ledger_dir(self.network, &storage_mode);
        let path_string = format!("(in \"{}\")", path.display()).dimmed();

        let mut cleaned = false;
        if path.exists() {
            self.reset_history_store(&storage_mode)?;
            cleaned = true;
        }
        let replay_path = leftover_history_replay_path(&path);
        if replay_path.exists() {
            std::fs::remove_dir_all(&replay_path).with_context(|| {
                format!("Failed to delete the leftover history replay at {}", replay_path.display())
            })?;
            cleaned = true;
        }

        match cleaned {
            true => Ok(format!("✅ Cleaned the history index {path_string}")),
            false => Ok(format!("✅ No history index was found {path_string}")),
        }
    }

    /// Imports JSON history into the selected ledger through its tip.
    ///
    /// `--history` deletes the stored index first. The ledger blocks and node data stay.
    fn import_history_json(&self, dir: &Path) -> Result<String> {
        ensure!(self.dev.is_none(), "`--history-json` cannot be used with `--dev`");
        ensure!(dir.is_dir(), "The JSON history directory '{}' does not exist", dir.display());

        let storage_mode = self.storage_mode();
        let path = aleo_std::aleo_ledger_dir(self.network, &storage_mode);
        let path_string = format!("(in \"{}\")", path.display()).dimmed();
        ensure!(path.exists(), "No snarkOS ledger was found {path_string}");

        // Import progress is logged with `tracing`.
        initialize_terminal_logger(0)?;
        match self.network {
            MainnetV0::ID => Self::import_network_history::<MainnetV0>(&storage_mode, dir, self.history)?,
            TestnetV0::ID => Self::import_network_history::<TestnetV0>(&storage_mode, dir, self.history)?,
            CanaryV0::ID => Self::import_network_history::<CanaryV0>(&storage_mode, dir, self.history)?,
            network => bail!("Unsupported network ID {network}"),
        }
        Ok(format!("✅ Imported history from {} {path_string}", dir.display()))
    }

    /// Loads the ledger of `N` and indexes its history from the JSON files in `dir`.
    fn import_network_history<N: Network>(storage_mode: &StorageMode, dir: &Path, reset: bool) -> Result<()> {
        let genesis = Block::<N>::from_bytes_le(N::genesis_bytes())?;
        let ledger = Ledger::<N, ConsensusDB<N>>::load(genesis, storage_mode.clone())?;
        if reset {
            ledger.reset_history()?;
        }
        ledger.import_history_json(dir)?;
        println!("History is indexed before block {}.", ledger.history_synced_height());
        Ok(())
    }

    /// Deletes the history tables of the ledger at `storage_mode`.
    ///
    /// v0 mapping-history prefixes are deleted before the open, which refuses a ledger that still
    /// has them.
    fn reset_history_store(&self, storage_mode: &StorageMode) -> Result<()> {
        match self.network {
            MainnetV0::ID => Self::reset_network_history::<MainnetV0>(storage_mode),
            TestnetV0::ID => Self::reset_network_history::<TestnetV0>(storage_mode),
            CanaryV0::ID => Self::reset_network_history::<CanaryV0>(storage_mode),
            network => bail!("Unsupported network ID {network}"),
        }
    }

    /// Deletes v0 mapping-history prefixes, then the indexed history, of `N`.
    fn reset_network_history<N: Network>(storage_mode: &StorageMode) -> Result<()> {
        RocksDB::open_dropping_legacy_mapping_history(N::ID, storage_mode.clone())?;
        FinalizeStore::<N, FinalizeDB<N>>::open(storage_mode.clone())?.reset_history()
    }

    /// Removes the specified node configuration from storage.
    fn remove_node_data(node_data_dir: &NodeDataDir) -> Result<String> {
        // With the new layout, we can remove the entire folder.
        let data_path = node_data_dir.path();

        // Prepare the path string.
        let path_string = format!("(in \"{}\")", data_path.display()).dimmed();

        if data_path.exists() {
            std::fs::remove_dir_all(data_path).with_context(|| format!("Failed to remove node data {path_string}"))?;
            Ok(format!("✅ Cleaned up node data {path_string}"))
        } else {
            Ok(format!("✅ No node data was found {path_string}"))
        }
    }

    /// Removes the specified ledger from storage.
    pub(crate) fn remove_ledger(network: u16, mode: &StorageMode) -> Result<String> {
        // Construct the path to the ledger in storage.
        let path = aleo_std::aleo_ledger_dir(network, mode);

        // Prepare the path string.
        let path_string = format!("(in \"{}\")", path.display()).dimmed();

        // Check if the path to the ledger exists in storage.
        if path.exists() {
            // Remove the ledger files from storage.
            std::fs::remove_dir_all(&path)
                .with_context(|| format!("Failed to remove the snarkOS ledger {path_string}"))?;
            Ok(format!("✅ Cleaned the snarkOS ledger {path_string}"))
        } else {
            Ok(format!("✅ No snarkOS ledger was found {path_string}"))
        }
    }
}

/// Returns the history-replay directory an earlier build kept beside `ledger`.
fn leftover_history_replay_path(ledger: &Path) -> PathBuf {
    let name = ledger.file_name().and_then(|name| name.to_str()).unwrap_or("ledger");
    let mut replay_path = ledger.to_path_buf();
    replay_path.set_file_name(format!("{name}-history-replay"));
    replay_path
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::{console::program::ProgramID, ledger::store::HistoryScope};
    use std::str::FromStr;

    use indexmap::IndexMap;

    #[test]
    fn history_flag_parses() {
        let clean = Clean::try_parse_from(["snarkos", "--history"]).unwrap();
        assert!(clean.history);
        assert!(clean.history_json.is_none());
        assert_eq!(clean.network, MainnetV0::ID);

        let clean = Clean::try_parse_from(["snarkos", "--history-json", "/data/history-0"]).unwrap();
        assert_eq!(clean.history_json, Some(PathBuf::from("/data/history-0")));
        assert!(!clean.history);

        let clean = Clean::try_parse_from(["snarkos", "--history", "--history-json", "/data/history-0"]).unwrap();
        assert!(clean.history);
        assert_eq!(clean.history_json, Some(PathBuf::from("/data/history-0")));
    }

    #[test]
    fn clean_history_resets_the_index_and_keeps_the_ledger_directory() {
        let dir = tempfile::tempdir().unwrap();
        let mode = StorageMode::Custom(dir.path().to_path_buf());
        let replay = leftover_history_replay_path(dir.path());
        std::fs::create_dir(&replay).unwrap();

        {
            let store = FinalizeStore::<MainnetV0, FinalizeDB<MainnetV0>>::open(mode.clone()).unwrap();
            let credits = ProgramID::<MainnetV0>::from_str("credits.aleo").unwrap();
            store.store_history_scope(&HistoryScope::programs(IndexMap::from([(credits, 10)]))).unwrap();
            store.set_history_synced_height(4).unwrap();
        }

        let clean = Clean {
            network: MainnetV0::ID,
            dev: None,
            ledger_storage: Some(dir.path().to_path_buf()),
            keep_node_data: false,
            node_data_storage: None,
            history: true,
            history_json: None,
        };
        let message = clean.parse().unwrap();
        assert!(message.contains("Cleaned the history index"), "{message}");
        assert!(dir.path().exists(), "the ledger directory stays");
        assert!(!replay.exists(), "a leftover history replay is removed");

        let store = FinalizeStore::<MainnetV0, FinalizeDB<MainnetV0>>::open(mode).unwrap();
        assert_eq!(store.history_synced_height(), 0);
        assert!(store.stored_history_scope().unwrap().is_none());
    }
}
