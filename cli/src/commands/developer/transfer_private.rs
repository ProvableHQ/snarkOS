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

use super::{DEFAULT_ENDPOINT, Developer};
use crate::{
    commands::StoreFormat,
    helpers::args::{parse_private_key, prepare_endpoint},
};
use snarkvm::{
    console::network::Network,
    ledger::store::helpers::memory::BlockMemory,
    prelude::{
        Address,
        Locator,
        VM,
        Value,
        query::Query,
        store::{ConsensusStore, helpers::memory::ConsensusMemory},
    },
};

use aleo_std::StorageMode;
use anyhow::Result;
use clap::{Parser, builder::NonEmptyStringValueParser};
use std::{path::PathBuf, str::FromStr};
use ureq::http::Uri;
use zeroize::Zeroize;

/// Executes the `transfer_private` function in the `credits.aleo` program.
#[derive(Debug, Parser)]
#[clap(
    group(clap::ArgGroup::new("mode").required(true).multiple(false))
)]
pub struct TransferPrivate {
    /// The input record used to craft the transfer.
    #[clap(long)]
    input_record: String,
    /// The recipient address.
    #[clap(long)]
    recipient: String,
    /// The number of microcredits to transfer.
    #[clap(long)]
    amount: u64,
    /// The private key used to generate the deployment.
    #[clap(short = 'p', long, group = "key", value_parser=NonEmptyStringValueParser::default())]
    private_key: Option<String>,
    /// Specify the path to a file containing the account private key of the node
    #[clap(long, group = "key", value_parser=NonEmptyStringValueParser::default())]
    private_key_file: Option<String>,
    /// Use a developer validator key tok generate the deployment.
    #[clap(long, group = "key")]
    dev_key: Option<u16>,
    /// The endpoint to query node state from and broadcast to (if set to broadcast).
    #[clap(short, long, default_value=DEFAULT_ENDPOINT)]
    endpoint: Uri,
    /// The priority fee in microcredits.
    #[clap(long)]
    priority_fee: u64,
    /// The record to spend the fee from.
    #[clap(long)]
    fee_record: String,
    /// Set the URL used to broadcast the transaction (if no value is given, the query endpoint is used).
    #[clap(short, long, group = "mode")]
    broadcast: Option<Option<Uri>>,
    /// Performs a dry-run of transaction generation.
    #[clap(short, long, group = "mode")]
    dry_run: bool,
    /// Store generated deployment transaction to a local file.
    #[clap(long, group = "mode")]
    store: Option<String>,
    /// If --store is specified, the format in which the transaction should be stored : string or
    /// bytes, by default : bytes.
    #[clap(long, value_enum, default_value_t = StoreFormat::Bytes, requires="store")]
    store_format: StoreFormat,
    /// Wait for the transaction to be accepted by the network. Requires --broadcast.
    #[clap(long, requires = "broadcast")]
    wait: bool,
    /// Timeout in seconds when waiting for transaction confirmation. Default is 60 seconds.
    #[clap(long, default_value_t = 60, requires = "wait")]
    timeout: u64,
    /// Specify the path to a directory containing the ledger. Overrides the default path.
    #[clap(long = "storage_path")]
    storage_path: Option<PathBuf>,
}

impl Drop for TransferPrivate {
    /// Zeroize the private key when the `TransferPrivate` struct goes out of scope.
    fn drop(&mut self) {
        self.private_key.zeroize();
    }
}

impl TransferPrivate {
    /// Creates an Aleo transfer with the provided inputs.
    pub fn parse<N: Network>(self) -> Result<String> {
        let endpoint = prepare_endpoint(self.endpoint.clone())?;

        // Specify the query
        let query = Query::<N, BlockMemory<N>>::from(endpoint.clone());

        // Retrieve the recipient.
        let recipient = Address::<N>::from_str(&self.recipient)?;

        // Retrieve the private key.
        let private_key = parse_private_key(self.private_key.clone(), self.private_key_file.clone(), self.dev_key)?;
        println!("📦 Creating private transfer of {} microcredits to {}...\n", self.amount, recipient);

        // Generate the transfer_private transaction.
        let transaction = {
            // Initialize an RNG.
            let rng = &mut rand::thread_rng();

            // Initialize the storage.
            let storage_mode = match &self.storage_path {
                Some(path) => StorageMode::Custom(path.clone()),
                None => StorageMode::Production,
            };
            let store = ConsensusStore::<N, ConsensusMemory<N>>::open(storage_mode)?;

            // Initialize the VM.
            let vm = VM::from(store)?;

            // Prepare the fee.
            let fee_record = Developer::parse_record(&private_key, &self.fee_record)?;
            let priority_fee = self.priority_fee;

            // Prepare the inputs for a transfer.
            let input_record = Developer::parse_record(&private_key, &self.input_record)?;
            let inputs = [
                Value::Record(input_record),
                Value::from_str(&format!("{recipient}"))?,
                Value::from_str(&format!("{}u64", self.amount))?,
            ];

            // Create a new transaction.
            vm.execute(
                &private_key,
                ("credits.aleo", "transfer_private"),
                inputs.iter(),
                Some(fee_record),
                priority_fee,
                Some(&query),
                rng,
            )?
        };
        let locator = Locator::<N>::from_str("credits.aleo/transfer_private")?;
        println!("✅ Created private transfer of {} microcredits to {}\n", &self.amount, recipient);

        // Determine if the transaction should be broadcast, stored, or displayed to the user.
        Developer::handle_transaction(
            &endpoint,
            &self.broadcast,
            self.dry_run,
            &self.store,
            self.store_format,
            self.wait,
            self.timeout,
            transaction,
            locator.to_string(),
        )
    }
}
