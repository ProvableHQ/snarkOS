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

use crate::helpers::{args::network_id_parser, dev::*};

use snarkos_account::Account;
use snarkos_display::Display;
use snarkos_node::{
    Node,
    bft::MEMORY_POOL_PORT,
    rest::DEFAULT_REST_PORT,
    router::{DEFAULT_NODE_PORT, messages::NodeType},
};
use snarkvm::{
    console::{
        account::{Address, PrivateKey},
        algorithms::Hash,
        network::{CanaryV0, MainnetV0, Network, TestnetV0},
    },
    ledger::{
        block::Block,
        committee::{Committee, MIN_DELEGATOR_STAKE, MIN_VALIDATOR_STAKE},
        store::{ConsensusStore, helpers::memory::ConsensusMemory},
    },
    prelude::{FromBytes, ToBits, ToBytes},
    synthesizer::VM,
    utilities::to_bytes_le,
};

use aleo_std::StorageMode;
use anyhow::{Context, Result, anyhow, bail, ensure};
use base64::prelude::{BASE64_STANDARD, Engine};
use clap::Parser;
use colored::Colorize;
use core::str::FromStr;
use indexmap::IndexMap;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaChaRng;
use serde::{Deserialize, Serialize};
use std::{
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, ToSocketAddrs},
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};
use tokio::runtime::{self, Runtime};
use ureq::http;

/// The recommended minimum number of 'open files' limit for a validator.
/// Validators should be able to handle at least 1000 concurrent connections, each requiring 2 sockets.
#[cfg(target_family = "unix")]
const RECOMMENDED_MIN_NOFILES_LIMIT: u64 = 2048;

/// A mapping of `staker_address` to `(validator_address, withdrawal_address, amount)`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
pub struct BondedBalances(IndexMap<String, (String, String, u64)>);

impl FromStr for BondedBalances {
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_json::from_str(s)
    }
}

/// Starts the snarkOS node.
#[derive(Clone, Debug, Parser)]
#[command(
    // Use kebab-case for all arguments (e.g., use the `private-key` flag for the `private_key` field).
    // This is already the default, but we specify it in case clap's default changes in the future.
    rename_all = "kebab-case",

    // Ensure at most one node type is specified.
    group(clap::ArgGroup::new("node_type").required(false).multiple(false)
),

    // Ensure all other dev flags can only be set if `--dev` is set.
    group(clap::ArgGroup::new("dev_flags").required(false).multiple(true).requires("dev")
),
    // Ensure any rest flag (including `--rest`) cannot be set
    // if `--norest` is set.
    group(clap::ArgGroup::new("rest_flags").required(false).multiple(true).conflicts_with("norest")),

    // Ensure you cannot set --verbosity and --log-filter flags at the same time.
    group(clap::ArgGroup::new("log_flags").required(false).multiple(false)),
)]
pub struct Start {
    /// Specify the network ID of this node
    /// [options: 0 = mainnet, 1 = testnet, 2 = canary]
    #[clap(long, default_value_t=MainnetV0::ID, long, value_parser = network_id_parser())]
    pub network: u16,

    /// Start the node as a prover.
    #[clap(long, group = "node_type")]
    pub prover: bool,

    /// Start the node as a client (default).
    ///
    /// Client are "full nodes", i.e, validate and execute all blocks they receive, but they do not participate in AleoBFT consensus.
    #[clap(long, group = "node_type", verbatim_doc_comment)]
    pub client: bool,

    /// Start the node as a validator.
    ///
    /// Validators are "full nodes", like clients, but also participate in AleoBFT.
    #[clap(long, group = "node_type", verbatim_doc_comment)]
    pub validator: bool,

    /// Specify the account private key of the node
    #[clap(long)]
    pub private_key: Option<String>,

    /// Specify the path to a file containing the account private key of the node
    #[clap(long = "private-key-file")]
    pub private_key_file: Option<PathBuf>,

    /// Set the IP address and port used for P2P communication.
    #[clap(long)]
    pub node: Option<SocketAddr>,

    /// Set the IP address and port used for BFT communication.
    /// This argument is only allowed for validator nodes.
    #[clap(long, requires = "validator")]
    pub bft: Option<SocketAddr>,

    /// Specify the IP address and port of the peer(s) to connect to (as a comma-separated list).
    ///
    /// These peers will be set as "trusted", which means the node will not disconnect from them when performing peer rotation.
    ///
    /// Setting peers to "" has the same effect as not setting the flag at all, except when using `--dev`.
    #[clap(long, verbatim_doc_comment)]
    pub peers: Option<String>,

    /// Specify the IP address and port of the validator(s) to connect to.
    #[clap(long)]
    pub validators: Option<String>,

    /// Allow untrusted peers (not listed in `--peers`) to connect.
    ///
    /// The flag will be ignored by client and prover nodes, as tis behavior is always enabled for these types of nodes.
    #[clap(long, verbatim_doc_comment)]
    pub allow_external_peers: bool,

    /// If the flag is set, a client will periodically evict more external peers
    #[clap(long)]
    pub rotate_external_peers: bool,

    /// Specify the IP address and port for the REST server
    #[clap(long, group = "rest_flags")]
    pub rest: Option<SocketAddr>,

    /// Specify the requests per second (RPS) rate limit per IP for the REST server
    #[clap(long, default_value_t = 10, group = "rest_flags")]
    pub rest_rps: u32,

    /// Specify the JWT secret for the REST server (16B, base64-encoded).
    #[clap(long, group = "rest_flags")]
    pub jwt_secret: Option<String>,

    /// Specify the JWT creation timestamp; can be any time in the last 10 years.
    #[clap(long, group = "rest_flags")]
    pub jwt_timestamp: Option<i64>,

    /// If the flag is set, the node will not initialize the REST server.
    #[clap(long)]
    pub norest: bool,

    /// Write log message to stdout instead of showing a terminal UI.
    ///
    /// This is useful, for example, for running a node as a service instead of in the foreground or to pipe its output into a file.
    #[clap(long, verbatim_doc_comment)]
    pub nodisplay: bool,

    /// Do not show the Aleo banner and information about the node on startup.
    #[clap(long, hide = true)]
    pub nobanner: bool,

    /// Specify the log verbosity of the node.
    /// [options: 0 (lowest log level) to 6 (highest level)]
    #[clap(long, default_value_t = 1, group = "log_flags")]
    pub verbosity: u8,

    /// Set a custom log filtering scheme, e.g., "off,snarkos_bft=trace", to show all log messages of snarkos_bft but nothing else.
    #[clap(long, group = "log_flags")]
    pub log_filter: Option<String>,

    /// Specify the path to the file where logs will be stored
    #[clap(long, default_value_os_t = std::env::temp_dir().join("snarkos.log"))]
    pub logfile: PathBuf,

    /// Enable the metrics exporter
    #[cfg(feature = "metrics")]
    #[clap(long)]
    pub metrics: bool,

    /// Specify the IP address and port for the metrics exporter
    #[cfg(feature = "metrics")]
    #[clap(long, requires = "metrics")]
    pub metrics_ip: Option<SocketAddr>,

    /// Specify the path to a directory containing the storage database for the ledger.
    /// This flag overrides the default path, even when `--dev` is set.
    #[clap(long)]
    pub storage: Option<PathBuf>,

    /// Enables the node to prefetch initial blocks from a CDN
    #[clap(long, conflicts_with = "nocdn")]
    pub cdn: Option<http::Uri>,

    /// If the flag is set, the node will not prefetch from a CDN
    #[clap(long)]
    pub nocdn: bool,

    /// Enables development mode used to set up test networks.
    ///
    /// The purpose of this flag is to run multiple nodes on the same machine and in the same working directory.
    /// To do this, set the value to a unique ID within the test work. For example if there are four nodes in the network, pass `--dev 0` for the first node, `--dev 1` for the second, and so forth.
    ///
    /// If you do not explicitly set the `--peers` flag, this will also populate the set of trusted peers, so that the network is fully connected.
    /// Additionally, if you do not set the `--rest` or the `--norest` flags, it will also set the REST port to `3030` for the first node, `3031` for the second, and so forth.
    #[clap(long, verbatim_doc_comment)]
    pub dev: Option<u16>,

    /// If development mode is enabled, specify the number of genesis validator.
    #[clap(long, group = "dev-flags", default_value_t=DEVELOPMENT_MODE_NUM_GENESIS_COMMITTEE_MEMBERS)]
    pub dev_num_validators: u16,

    /// If development mode is enabled, specify whether node 0 should generate traffic to drive the network.
    #[clap(long, group = "dev-flag")]
    pub no_dev_txs: bool,

    /// If development mode is enabled, specify the custom bonded balances as a JSON object.
    #[clap(long, group = "dev-flags")]
    pub dev_bonded_balances: Option<BondedBalances>,
}

impl Start {
    /// Starts the snarkOS node.
    pub fn parse(self) -> Result<String> {
        // Prepare the shutdown flag.
        let shutdown: Arc<AtomicBool> = Default::default();

        // Initialize the logger.
        let log_receiver = crate::helpers::initialize_logger(
            self.verbosity,
            &self.log_filter,
            self.nodisplay,
            self.logfile.clone(),
            shutdown.clone(),
        )
        .with_context(|| "Failed to set up logger")?;

        // Initialize the runtime.
        Self::runtime().block_on(async move {
            // Error messages.
            let node_parse_error = || "Failed to parse node arguments";
            let display_start_error = || "Failed to initialize the display";

            // Clone the configurations.
            let mut cli = self.clone();
            // Parse the network.
            match cli.network {
                MainnetV0::ID => {
                    // Parse the node from the configurations.
                    let node = cli.parse_node::<MainnetV0>(shutdown.clone()).await.with_context(node_parse_error)?;
                    // If the display is enabled, render the display.
                    if !cli.nodisplay {
                        // Initialize the display.
                        Display::start(node, log_receiver).with_context(display_start_error)?;
                    }
                }
                TestnetV0::ID => {
                    // Parse the node from the configurations.
                    let node = cli.parse_node::<TestnetV0>(shutdown.clone()).await.with_context(node_parse_error)?;
                    // If the display is enabled, render the display.
                    if !cli.nodisplay {
                        // Initialize the display.
                        Display::start(node, log_receiver).with_context(display_start_error)?;
                    }
                }
                CanaryV0::ID => {
                    // Parse the node from the configurations.
                    let node = cli.parse_node::<CanaryV0>(shutdown.clone()).await.with_context(node_parse_error)?;
                    // If the display is enabled, render the display.
                    if !cli.nodisplay {
                        // Initialize the display.
                        Display::start(node, log_receiver).with_context(display_start_error)?;
                    }
                }
                _ => panic!("Invalid network ID specified"),
            };
            // Note: Do not move this. The pending await must be here otherwise
            // other snarkOS commands will not exit.
            std::future::pending::<()>().await;
            Ok(String::new())
        })
    }
}

impl Start {
    /// Returns the initial peer(s) to connect to, from the given configurations.
    fn parse_trusted_peers(&self) -> Result<Vec<SocketAddr>> {
        let Some(peers) = &self.peers else { return Ok(vec![]) };

        match peers.is_empty() {
            // Split on an empty string returns an empty string.
            true => Ok(vec![]),
            false => peers
                .split(',')
                .map(|ip_or_hostname| {
                    let trimmed = ip_or_hostname.trim();
                    match trimmed.to_socket_addrs() {
                        Ok(mut ip_iter) => {
                            // A hostname might resolve to multiple IP addresses. We will use only the first one,
                            // assuming this aligns with the user's expectations.
                            let Some(ip) = ip_iter.next() else {
                                return Err(anyhow!(
                                    "The hostname supplied to --peers ('{trimmed}') does not reference any ip."
                                ));
                            };
                            Ok(ip)
                        }
                        Err(e) => {
                            Err(anyhow!("The hostname or IP supplied to --peers ('{trimmed}') is malformed: {e}"))
                        }
                    }
                })
                .collect(),
        }
    }

    /// Returns the initial validator(s) to connect to, from the given configurations.
    fn parse_trusted_validators(&self) -> Result<Vec<SocketAddr>> {
        let Some(validators) = &self.validators else { return Ok(vec![]) };

        // Split on an empty string returns an empty string.
        if validators.is_empty() {
            return Ok(vec![]);
        }

        let mut result = vec![];
        for ip in validators.split(',') {
            match ip.parse::<SocketAddr>() {
                Ok(ip) => result.push(ip),
                Err(err) => bail!("An address supplied to --validators ('{ip}') is malformed: {err}"),
            }
        }

        Ok(result)
    }

    /// Returns the CDN to prefetch initial blocks from, from the given configurations.
    fn parse_cdn<N: Network>(&self) -> Result<Option<http::Uri>> {
        // Determine if the node type is not declared.
        let is_no_node_type = !(self.validator || self.prover || self.client);

        // Disable CDN if:
        //  1. The node is in development mode.
        //  2. The user has explicitly disabled CDN.
        //  3. The node is a prover (no need to sync).
        //  4. The node type is not declared (defaults to client) (no need to sync).
        if self.dev.is_some() || self.nocdn || self.prover || is_no_node_type {
            Ok(None)
        }
        // Enable the CDN otherwise.
        else {
            // Determine the CDN URL.
            match &self.cdn {
                // Use the provided CDN URL if it is not empty.
                Some(cdn) => match cdn.to_string().is_empty() {
                    true => Ok(None),
                    false => Ok(Some(cdn.clone())),
                },
                // If no CDN URL is provided, determine the CDN URL based on the network ID.
                None => {
                    let uri = format!("{}/{}", snarkos_node_cdn::CDN_BASE_URL, N::SHORT_NAME);
                    Ok(Some(http::Uri::try_from(&uri).with_context(|| "Unexpected error")?))
                }
            }
        }
    }

    /// Read the private key directly from an argument or from a filesystem location,
    /// returning the Aleo account.
    fn parse_private_key<N: Network>(&self) -> Result<Account<N>> {
        match self.dev {
            None => match (&self.private_key, &self.private_key_file) {
                // Parse the private key directly.
                (Some(private_key), None) => Account::from_str(private_key.trim()),
                // Parse the private key from a file.
                (None, Some(path)) => {
                    check_permissions(path)?;
                    Account::from_str(std::fs::read_to_string(path)?.trim())
                }
                // Ensure the private key is provided to the CLI, except for clients or nodes in development mode.
                (None, None) => match self.client {
                    true => Account::new(&mut rand::thread_rng()),
                    false => bail!("Missing the '--private-key' or '--private-key-file' argument"),
                },
                // Ensure only one private key flag is provided to the CLI.
                (Some(_), Some(_)) => {
                    bail!("Cannot use '--private-key' and '--private-key-file' simultaneously, please use only one")
                }
            },
            Some(index) => {
                let private_key = get_development_key(index)?;
                if !self.nobanner {
                    println!(
                        "🔑 Your development private key for node {index} is {}.\n",
                        private_key.to_string().bold()
                    );
                }
                Account::try_from(private_key)
            }
        }
    }

    /// Updates the configurations if the node is in development mode.
    fn parse_development(&mut self, trusted_peers: &mut Vec<SocketAddr>, trusted_validators: &mut Vec<SocketAddr>) {
        // If `--dev` is set, assume the dev nodes are initialized from 0 to `dev`,
        // and add each of them to the trusted peers. In addition, set the node IP to `4130 + dev`,
        // and the REST port to `3030 + dev`.

        if let Some(dev) = self.dev {
            // Add the dev nodes to the trusted peers.
            if trusted_peers.is_empty() {
                for i in 0..dev {
                    trusted_peers.push(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, DEFAULT_NODE_PORT + i)));
                }
            }
            // Add the dev nodes to the trusted validators.
            if trusted_validators.is_empty() {
                // To avoid ambiguity, we define the first few nodes to be the trusted validators to connect to.
                for i in 0..2 {
                    // Don't connect to yourself.
                    if i == dev {
                        continue;
                    }

                    trusted_validators
                        .push(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, MEMORY_POOL_PORT + i)));
                }
            }
            // Set the node IP to `4130 + dev`.
            //
            // Note: the `node` flag is an option to detect remote devnet testing.
            if self.node.is_none() {
                self.node = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DEFAULT_NODE_PORT + dev)));
            }

            // If the `norest` flag is not set and the REST IP is not already specified set the REST IP to `3030 + dev`.
            if !self.norest && self.rest.is_none() {
                self.rest = Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DEFAULT_REST_PORT + dev)));
            }
        }
    }

    /// Returns an alternative genesis block if the node is in development mode.
    /// Otherwise, returns the actual genesis block.
    fn parse_genesis<N: Network>(&self) -> Result<Block<N>> {
        if self.dev.is_some() {
            // Determine the number of genesis committee members.
            let num_committee_members = self.dev_num_validators;
            ensure!(
                num_committee_members >= DEVELOPMENT_MODE_NUM_GENESIS_COMMITTEE_MEMBERS,
                "Number of genesis committee members is too low"
            );

            // Initialize the (fixed) RNG.
            let mut rng = ChaChaRng::seed_from_u64(DEVELOPMENT_MODE_RNG_SEED);
            // Initialize the development private keys.
            let dev_keys =
                (0..num_committee_members).map(|_| PrivateKey::<N>::new(&mut rng)).collect::<Result<Vec<_>>>()?;
            // Initialize the development addresses.
            let development_addresses = dev_keys.iter().map(Address::<N>::try_from).collect::<Result<Vec<_>>>()?;

            // Construct the committee based on the state of the bonded balances.
            let (committee, bonded_balances) = match &self.dev_bonded_balances {
                Some(bonded_balances) => {
                    // Parse the bonded balances.
                    let bonded_balances = bonded_balances
                        .0
                        .iter()
                        .map(|(staker_address, (validator_address, withdrawal_address, amount))| {
                            let staker_addr = Address::<N>::from_str(staker_address)?;
                            let validator_addr = Address::<N>::from_str(validator_address)?;
                            let withdrawal_addr = Address::<N>::from_str(withdrawal_address)?;
                            Ok((staker_addr, (validator_addr, withdrawal_addr, *amount)))
                        })
                        .collect::<Result<IndexMap<_, _>>>()?;

                    // Construct the committee members.
                    let mut members = IndexMap::new();
                    for (staker_address, (validator_address, _, amount)) in bonded_balances.iter() {
                        // Ensure that the staking amount is sufficient.
                        match staker_address == validator_address {
                            true => ensure!(amount >= &MIN_VALIDATOR_STAKE, "Validator stake is too low"),
                            false => ensure!(amount >= &MIN_DELEGATOR_STAKE, "Delegator stake is too low"),
                        }

                        // Ensure that the validator address is included in the list of development addresses.
                        ensure!(
                            development_addresses.contains(validator_address),
                            "Validator address {validator_address} is not included in the list of development addresses"
                        );

                        // Add or update the validator entry in the list of members
                        members.entry(*validator_address).and_modify(|(stake, _, _)| *stake += amount).or_insert((
                            *amount,
                            true,
                            rng.gen_range(0..100),
                        ));
                    }
                    // Construct the committee.
                    let committee = Committee::<N>::new(0u64, members)?;
                    (committee, bonded_balances)
                }
                None => {
                    // Calculate the committee stake per member.
                    let stake_per_member =
                        N::STARTING_SUPPLY.saturating_div(2).saturating_div(num_committee_members as u64);
                    ensure!(stake_per_member >= MIN_VALIDATOR_STAKE, "Committee stake per member is too low");

                    // Construct the committee members and distribute stakes evenly among committee members.
                    let members = development_addresses
                        .iter()
                        .map(|address| (*address, (stake_per_member, true, rng.gen_range(0..100))))
                        .collect::<IndexMap<_, _>>();

                    // Construct the bonded balances.
                    // Note: The withdrawal address is set to the staker address.
                    let bonded_balances = members
                        .iter()
                        .map(|(address, (stake, _, _))| (*address, (*address, *address, *stake)))
                        .collect::<IndexMap<_, _>>();
                    // Construct the committee.
                    let committee = Committee::<N>::new(0u64, members)?;

                    (committee, bonded_balances)
                }
            };

            // Ensure that the number of committee members is correct.
            ensure!(
                committee.members().len() == num_committee_members as usize,
                "Number of committee members {} does not match the expected number of members {num_committee_members}",
                committee.members().len()
            );

            // Calculate the public balance per validator.
            let remaining_balance = N::STARTING_SUPPLY.saturating_sub(committee.total_stake());
            let public_balance_per_validator = remaining_balance.saturating_div(num_committee_members as u64);

            // Construct the public balances with fairly equal distribution.
            let mut public_balances = dev_keys
                .iter()
                .map(|private_key| Ok((Address::try_from(private_key)?, public_balance_per_validator)))
                .collect::<Result<indexmap::IndexMap<_, _>>>()?;

            // If there is some leftover balance, add it to the 0-th validator.
            let leftover =
                remaining_balance.saturating_sub(public_balance_per_validator * num_committee_members as u64);
            if leftover > 0 {
                let (_, balance) = public_balances.get_index_mut(0).unwrap();
                *balance += leftover;
            }

            // Check if the sum of committee stakes and public balances equals the total starting supply.
            let public_balances_sum: u64 = public_balances.values().copied().sum();
            if committee.total_stake() + public_balances_sum != N::STARTING_SUPPLY {
                bail!("Sum of committee stakes and public balances does not equal total starting supply.");
            }

            // Construct the genesis block.
            load_or_compute_genesis(dev_keys[0], committee, public_balances, bonded_balances, &mut rng)
        } else {
            Block::from_bytes_le(N::genesis_bytes())
        }
    }

    /// Returns the node type specified in the command-line arguments.
    /// This will return `NodeType::Client` if no node type was specified by the user.
    const fn parse_node_type(&self) -> NodeType {
        if self.validator {
            NodeType::Validator
        } else if self.prover {
            NodeType::Prover
        } else {
            NodeType::Client
        }
    }

    /// Returns the node type corresponding to the given configurations.
    #[rustfmt::skip]
    async fn parse_node<N: Network>(&mut self, shutdown: Arc<AtomicBool>) -> Result<Node<N>> {
        if !self.nobanner {
            // Print the welcome banner.
            println!("{}", crate::helpers::welcome_message());
        }

        // Check if we are running with the lower coinbase and proof targets. This should only be
        // allowed in --dev mode and should not be allowed in mainnet mode.
        if cfg!(feature = "test_network") && self.dev.is_none() {
            bail!("The 'test_network' feature is enabled, but the '--dev' flag is not set");
        }

        // Parse the trusted peers to connect to.
        let mut trusted_peers = self.parse_trusted_peers()?;
        // Parse the trusted validators to connect to.
        let mut trusted_validators = self.parse_trusted_validators()?;
        // Parse the development configurations.
        self.parse_development(&mut trusted_peers, &mut trusted_validators);

        // Parse the CDN.
        let cdn = self.parse_cdn::<N>().with_context(|| "Failed to parse given CDN URL")?;

        // Parse the genesis block.
        let genesis = self.parse_genesis::<N>()?;
        // Parse the private key of the node.
        let account = self.parse_private_key::<N>()?;
        // Parse the node type.
        let node_type = self.parse_node_type();

        // Parse the node IP or use the default IP/port.
        let node_ip = self.node.unwrap_or(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DEFAULT_NODE_PORT)));

        // Parse the REST IP.
        let rest_ip = match self.norest {
            true => None,
            false => self.rest.or_else(|| Some("0.0.0.0:3030".parse().unwrap())),
        };

        if !self.nobanner {
            // Print the Aleo address.
            println!("👛 Your Aleo address is {}.\n", account.address().to_string().bold());
            // Print the node type and network.
            println!(
                "🧭 Starting {} on {} at {}.\n",
                node_type.description().bold(),
                N::NAME.bold(),
                node_ip.to_string().bold()
            );

            // If the node is running a REST server, print the REST IP and JWT.
            if node_type.is_validator() || node_type.is_client() {
                if let Some(rest_ip) = rest_ip {
                    println!("🌐 Starting the REST server at {}.\n", rest_ip.to_string().bold());

                    let jwt_secret = if let Some(jwt_b64) = &self.jwt_secret {
                        if self.jwt_timestamp.is_none() {
                            bail!("The '--jwt-timestamp' flag must be set if the '--jwt-secret' flag is set");
                        }
                        let jwt_bytes = BASE64_STANDARD.decode(jwt_b64).map_err(|_| anyhow::anyhow!("Invalid JWT secret"))?;
                        if jwt_bytes.len() != 16 {
                            bail!("The JWT secret must be 16 bytes long");
                        }
                        Some(jwt_bytes)
                    } else {
                        None
                    };

                    if let Ok(jwt_token) = snarkos_node_rest::Claims::new(account.address(), jwt_secret, self.jwt_timestamp).to_jwt_string() {
                        println!("🔑 Your one-time JWT token is {}\n", jwt_token.dimmed());
                    }
                }
            }
        }

        // If the node is a validator, check if the open files limit is lower than recommended.
        #[cfg(target_family = "unix")]
        if node_type.is_validator() {
            crate::helpers::check_open_files_limit(RECOMMENDED_MIN_NOFILES_LIMIT);
        }
        // Check if the machine meets the minimum requirements for a validator.
        crate::helpers::check_validator_machine(node_type);

        // Initialize the metrics.
        #[cfg(feature = "metrics")]
        if self.metrics {
            metrics::initialize_metrics(self.metrics_ip);
        }

        // Initialize the storage mode.
        let storage_mode = match &self.storage {
            Some(path) => StorageMode::Custom(path.clone()),
            None => match self.dev {
                Some(id) => StorageMode::Development(id),
                None => StorageMode::Production,
            },
        };

        // Determine whether to generate background transactions in dev mode.
        let dev_txs = match self.dev {
            Some(_) => !self.no_dev_txs,
            None => {
                // If the `no_dev_txs` flag is set, inform the user that it is ignored.
                if self.no_dev_txs {
                    eprintln!("The '--no-dev-txs' flag is ignored because '--dev' is not set");
                }
                false
            }
        };


        // TODO(kaimast): start the display earlier and show sync progress.
        if !self.nodisplay && !self.nocdn {
            println!("🪧 The terminal UI will not start until the node has finished syncing from the CDN. If this step takes too long, consider restarting with `--nodisplay`.");
        }

        // Initialize the node.
        match node_type {
            NodeType::Validator => Node::new_validator(node_ip, self.bft, rest_ip, self.rest_rps, account, &trusted_peers, &trusted_validators, genesis, cdn, storage_mode, self.allow_external_peers, dev_txs, self.dev, shutdown.clone()).await,
            NodeType::Prover => Node::new_prover(node_ip, account, &trusted_peers, genesis, self.dev, shutdown.clone()).await,
            NodeType::Client => Node::new_client(node_ip, rest_ip, self.rest_rps, account, &trusted_peers, genesis, cdn, storage_mode, self.rotate_external_peers, self.dev, shutdown).await
        }
    }

    /// Returns a runtime for the node.
    fn runtime() -> Runtime {
        // Retrieve the number of cores.
        let num_cores = num_cpus::get();

        // Initialize the number of tokio worker threads, max tokio blocking threads, and rayon cores.
        // Note: We intentionally set the number of tokio worker threads and number of rayon cores to be
        // more than the number of physical cores, because the node is expected to be I/O-bound.
        let (num_tokio_worker_threads, max_tokio_blocking_threads, num_rayon_cores_global) =
            (2 * num_cores, 512, num_cores);

        // Initialize the parallelization parameters.
        rayon::ThreadPoolBuilder::new()
            .stack_size(8 * 1024 * 1024)
            .num_threads(num_rayon_cores_global)
            .build_global()
            .unwrap();

        // Initialize the runtime configuration.
        runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_stack_size(8 * 1024 * 1024)
            .worker_threads(num_tokio_worker_threads)
            .max_blocking_threads(max_tokio_blocking_threads)
            .build()
            .expect("Failed to initialize a runtime for the router")
    }
}

fn check_permissions(path: &PathBuf) -> Result<(), snarkvm::prelude::Error> {
    #[cfg(target_family = "unix")]
    {
        use std::os::unix::fs::PermissionsExt;
        ensure!(path.exists(), "The file '{:?}' does not exist", path);
        crate::check_parent_permissions(path)?;
        let permissions = path.metadata()?.permissions().mode();
        ensure!(permissions & 0o777 == 0o600, "The file {:?} must be readable only by the owner (0600)", path);
    }
    Ok(())
}

/// Loads or computes the genesis block.
fn load_or_compute_genesis<N: Network>(
    genesis_private_key: PrivateKey<N>,
    committee: Committee<N>,
    public_balances: indexmap::IndexMap<Address<N>, u64>,
    bonded_balances: indexmap::IndexMap<Address<N>, (Address<N>, Address<N>, u64)>,
    rng: &mut ChaChaRng,
) -> Result<Block<N>> {
    // Construct the preimage.
    let mut preimage = Vec::new();

    // Input the network ID.
    preimage.extend(&N::ID.to_le_bytes());
    // Input the genesis coinbase target.
    preimage.extend(&to_bytes_le![N::GENESIS_COINBASE_TARGET]?);
    // Input the genesis proof target.
    preimage.extend(&to_bytes_le![N::GENESIS_PROOF_TARGET]?);

    // Input the genesis private key, committee, and public balances.
    preimage.extend(genesis_private_key.to_bytes_le()?);
    preimage.extend(committee.to_bytes_le()?);
    preimage.extend(&to_bytes_le![public_balances.iter().collect::<Vec<(_, _)>>()]?);
    preimage.extend(&to_bytes_le![
        bonded_balances
            .iter()
            .flat_map(|(staker, (validator, withdrawal, amount))| to_bytes_le![staker, validator, withdrawal, amount])
            .collect::<Vec<_>>()
    ]?);

    // Input the parameters' metadata based on network
    match N::ID {
        snarkvm::console::network::MainnetV0::ID => {
            preimage.extend(snarkvm::parameters::mainnet::BondValidatorVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::BondPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::UnbondPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::ClaimUnbondPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::SetValidatorStateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::TransferPrivateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::TransferPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::TransferPrivateToPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::TransferPublicToPrivateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::FeePrivateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::FeePublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::mainnet::InclusionVerifier::METADATA.as_bytes());
        }
        snarkvm::console::network::TestnetV0::ID => {
            preimage.extend(snarkvm::parameters::testnet::BondValidatorVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::BondPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::UnbondPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::ClaimUnbondPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::SetValidatorStateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::TransferPrivateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::TransferPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::TransferPrivateToPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::TransferPublicToPrivateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::FeePrivateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::FeePublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::testnet::InclusionVerifier::METADATA.as_bytes());
        }
        snarkvm::console::network::CanaryV0::ID => {
            preimage.extend(snarkvm::parameters::canary::BondValidatorVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::BondPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::UnbondPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::ClaimUnbondPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::SetValidatorStateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::TransferPrivateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::TransferPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::TransferPrivateToPublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::TransferPublicToPrivateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::FeePrivateVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::FeePublicVerifier::METADATA.as_bytes());
            preimage.extend(snarkvm::parameters::canary::InclusionVerifier::METADATA.as_bytes());
        }
        _ => {
            // Unrecognized Network ID
            bail!("Unrecognized Network ID: {}", N::ID);
        }
    }

    // Initialize the hasher.
    let hasher = snarkvm::console::algorithms::BHP256::<N>::setup("aleo.dev.block")?;
    // Compute the hash.
    // NOTE: this is a fast-to-compute but *IMPERFECT* identifier for the genesis block;
    //       to know the actual genesis block hash, you need to compute the block itself.
    let hash = hasher.hash(&preimage.to_bits_le())?.to_string();

    // A closure to load the block.
    let load_block = |file_path| -> Result<Block<N>> {
        // Attempts to load the genesis block file locally.
        let buffer = std::fs::read(file_path)?;
        // Return the genesis block.
        Block::from_bytes_le(&buffer)
    };

    // Construct the file path.
    let file_path = std::env::temp_dir().join(hash);
    // Check if the genesis block exists.
    if file_path.exists() {
        // If the block loads successfully, return it.
        if let Ok(block) = load_block(&file_path) {
            return Ok(block);
        }
    }

    /* Otherwise, compute the genesis block and store it. */

    // Initialize a new VM.
    let vm = VM::from(ConsensusStore::<N, ConsensusMemory<N>>::open(StorageMode::new_test(None))?)?;
    // Initialize the genesis block.
    let block = vm.genesis_quorum(&genesis_private_key, committee, public_balances, bonded_balances, rng)?;
    // Write the genesis block to the file.
    std::fs::write(&file_path, block.to_bytes_le()?)?;
    // Return the genesis block.
    Ok(block)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{CLI, Command};
    use snarkvm::prelude::MainnetV0;

    use ureq::http;

    type CurrentNetwork = MainnetV0;

    #[test]
    fn test_parse_trusted_peers() {
        let config = Start::try_parse_from(["snarkos", "--peers", ""].iter()).unwrap();
        assert!(config.parse_trusted_peers().is_ok());
        assert!(config.parse_trusted_peers().unwrap().is_empty());

        let config = Start::try_parse_from(["snarkos", "--peers", "1.2.3.4:5"].iter()).unwrap();
        assert!(config.parse_trusted_peers().is_ok());
        assert_eq!(config.parse_trusted_peers().unwrap(), vec![SocketAddr::from_str("1.2.3.4:5").unwrap()]);

        let config = Start::try_parse_from(["snarkos", "--peers", "1.2.3.4:5,6.7.8.9:0"].iter()).unwrap();
        assert!(config.parse_trusted_peers().is_ok());
        assert_eq!(config.parse_trusted_peers().unwrap(), vec![
            SocketAddr::from_str("1.2.3.4:5").unwrap(),
            SocketAddr::from_str("6.7.8.9:0").unwrap()
        ]);
    }

    #[test]
    fn test_parse_trusted_validators() {
        let config = Start::try_parse_from(["snarkos", "--validators", ""].iter()).unwrap();
        assert!(config.parse_trusted_validators().is_ok());
        assert!(config.parse_trusted_validators().unwrap().is_empty());

        let config = Start::try_parse_from(["snarkos", "--validators", "1.2.3.4:5"].iter()).unwrap();
        assert!(config.parse_trusted_validators().is_ok());
        assert_eq!(config.parse_trusted_validators().unwrap(), vec![SocketAddr::from_str("1.2.3.4:5").unwrap()]);

        let config = Start::try_parse_from(["snarkos", "--validators", "1.2.3.4:5,6.7.8.9:0"].iter()).unwrap();
        assert!(config.parse_trusted_validators().is_ok());
        assert_eq!(config.parse_trusted_validators().unwrap(), vec![
            SocketAddr::from_str("1.2.3.4:5").unwrap(),
            SocketAddr::from_str("6.7.8.9:0").unwrap()
        ]);
    }

    #[test]
    fn test_parse_log_filter() {
        // Ensure we cannot set, both, log-filter and verbosity
        let result = Start::try_parse_from(["snarkos", "--verbosity=5", "--log-filter=warn"].iter());
        assert!(result.is_err(), "Must not be able to set log-filter and verbosity at the same time");

        // Ensure the values are set correctly.
        let config = Start::try_parse_from(["snarkos", "--verbosity=5"].iter()).unwrap();
        assert_eq!(config.verbosity, 5);
        let config = Start::try_parse_from(["snarkos", "--log-filter=snarkos=warn"].iter()).unwrap();
        assert_eq!(config.log_filter, Some("snarkos=warn".to_string()));
    }

    #[test]
    fn test_parse_cdn() -> Result<()> {
        // Validator (Prod)
        let config = Start::try_parse_from(["snarkos", "--validator", "--private-key", "aleo1xx"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_some());
        let config =
            Start::try_parse_from(["snarkos", "--validator", "--private-key", "aleo1xx", "--cdn", "url"].iter())
                .unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_some());
        let config = Start::try_parse_from(["snarkos", "--validator", "--private-key", "aleo1xx", "--nocdn"].iter())?;
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());

        // Validator (Dev)
        let config =
            Start::try_parse_from(["snarkos", "--dev", "0", "--validator", "--private-key", "aleo1xx"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config = Start::try_parse_from(
            ["snarkos", "--dev", "0", "--validator", "--private-key", "aleo1xx", "--cdn", "url"].iter(),
        )
        .unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config = Start::try_parse_from(
            ["snarkos", "--dev", "0", "--validator", "--private-key", "aleo1xx", "--nocdn"].iter(),
        )?;
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());

        // Prover (Prod)
        let config = Start::try_parse_from(["snarkos", "--prover", "--private-key", "aleo1xx"].iter())?;
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config = Start::try_parse_from(["snarkos", "--prover", "--private-key", "aleo1xx", "--cdn", "url"].iter())?;
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config = Start::try_parse_from(["snarkos", "--prover", "--private-key", "aleo1xx", "--nocdn"].iter())?;
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());

        // Prover (Dev)
        let config =
            Start::try_parse_from(["snarkos", "--dev", "0", "--prover", "--private-key", "aleo1xx"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config = Start::try_parse_from(
            ["snarkos", "--dev", "0", "--prover", "--private-key", "aleo1xx", "--cdn", "url"].iter(),
        )
        .unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config =
            Start::try_parse_from(["snarkos", "--dev", "0", "--prover", "--private-key", "aleo1xx", "--nocdn"].iter())
                .unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());

        // Client (Prod)
        let config = Start::try_parse_from(["snarkos", "--client", "--private-key", "aleo1xx"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_some());
        let config =
            Start::try_parse_from(["snarkos", "--client", "--private-key", "aleo1xx", "--cdn", "url"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_some());
        let config =
            Start::try_parse_from(["snarkos", "--client", "--private-key", "aleo1xx", "--nocdn"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());

        // Client (Dev)
        let config =
            Start::try_parse_from(["snarkos", "--dev", "0", "--client", "--private-key", "aleo1xx"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config = Start::try_parse_from(
            ["snarkos", "--dev", "0", "--client", "--private-key", "aleo1xx", "--cdn", "url"].iter(),
        )
        .unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config =
            Start::try_parse_from(["snarkos", "--dev", "0", "--client", "--private-key", "aleo1xx", "--nocdn"].iter())
                .unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());

        // Default (Prod)
        let config = Start::try_parse_from(["snarkos"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config = Start::try_parse_from(["snarkos", "--cdn", "url"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config = Start::try_parse_from(["snarkos", "--nocdn"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());

        // Default (Dev)
        let config = Start::try_parse_from(["snarkos", "--dev", "0"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config = Start::try_parse_from(["snarkos", "--dev", "0", "--cdn", "url"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());
        let config = Start::try_parse_from(["snarkos", "--dev", "0", "--nocdn"].iter()).unwrap();
        assert!(config.parse_cdn::<CurrentNetwork>()?.is_none());

        Ok(())
    }

    #[test]
    fn test_parse_development_and_genesis() {
        let prod_genesis = Block::from_bytes_le(CurrentNetwork::genesis_bytes()).unwrap();

        let mut trusted_peers = vec![];
        let mut trusted_validators = vec![];
        let mut config = Start::try_parse_from(["snarkos"].iter()).unwrap();
        config.parse_development(&mut trusted_peers, &mut trusted_validators);
        let candidate_genesis = config.parse_genesis::<CurrentNetwork>().unwrap();
        assert_eq!(trusted_peers.len(), 0);
        assert_eq!(trusted_validators.len(), 0);
        assert_eq!(candidate_genesis, prod_genesis);

        let _config = Start::try_parse_from(["snarkos", "--dev", ""].iter()).unwrap_err();

        let mut trusted_peers = vec![];
        let mut trusted_validators = vec![];
        let mut config = Start::try_parse_from(["snarkos", "--dev", "1"].iter()).unwrap();
        config.parse_development(&mut trusted_peers, &mut trusted_validators);
        assert_eq!(config.rest, Some(SocketAddr::from_str("0.0.0.0:3031").unwrap()));

        let mut trusted_peers = vec![];
        let mut trusted_validators = vec![];
        let mut config = Start::try_parse_from(["snarkos", "--dev", "1", "--rest", "127.0.0.1:8080"].iter()).unwrap();
        config.parse_development(&mut trusted_peers, &mut trusted_validators);
        assert_eq!(config.rest, Some(SocketAddr::from_str("127.0.0.1:8080").unwrap()));

        let mut trusted_peers = vec![];
        let mut trusted_validators = vec![];
        let mut config = Start::try_parse_from(["snarkos", "--dev", "1", "--norest"].iter()).unwrap();
        config.parse_development(&mut trusted_peers, &mut trusted_validators);
        assert!(config.rest.is_none());

        let mut trusted_peers = vec![];
        let mut trusted_validators = vec![];
        let mut config = Start::try_parse_from(["snarkos", "--dev", "0"].iter()).unwrap();
        config.parse_development(&mut trusted_peers, &mut trusted_validators);
        let expected_genesis = config.parse_genesis::<CurrentNetwork>().unwrap();
        assert_eq!(config.node, Some(SocketAddr::from_str("0.0.0.0:4130").unwrap()));
        assert_eq!(config.rest, Some(SocketAddr::from_str("0.0.0.0:3030").unwrap()));
        assert_eq!(trusted_peers.len(), 0);
        assert_eq!(trusted_validators.len(), 1);
        assert!(!config.validator);
        assert!(!config.prover);
        assert!(!config.client);
        assert_ne!(expected_genesis, prod_genesis);

        let mut trusted_peers = vec![];
        let mut trusted_validators = vec![];
        let mut config =
            Start::try_parse_from(["snarkos", "--dev", "1", "--validator", "--private-key", ""].iter()).unwrap();
        config.parse_development(&mut trusted_peers, &mut trusted_validators);
        let genesis = config.parse_genesis::<CurrentNetwork>().unwrap();
        assert_eq!(config.node, Some(SocketAddr::from_str("0.0.0.0:4131").unwrap()));
        assert_eq!(config.rest, Some(SocketAddr::from_str("0.0.0.0:3031").unwrap()));
        assert_eq!(trusted_peers.len(), 1);
        assert_eq!(trusted_validators.len(), 1);
        assert!(config.validator);
        assert!(!config.prover);
        assert!(!config.client);
        assert_eq!(genesis, expected_genesis);

        let mut trusted_peers = vec![];
        let mut trusted_validators = vec![];
        let mut config =
            Start::try_parse_from(["snarkos", "--dev", "2", "--prover", "--private-key", ""].iter()).unwrap();
        config.parse_development(&mut trusted_peers, &mut trusted_validators);
        let genesis = config.parse_genesis::<CurrentNetwork>().unwrap();
        assert_eq!(config.node, Some(SocketAddr::from_str("0.0.0.0:4132").unwrap()));
        assert_eq!(config.rest, Some(SocketAddr::from_str("0.0.0.0:3032").unwrap()));
        assert_eq!(trusted_peers.len(), 2);
        assert_eq!(trusted_validators.len(), 2);
        assert!(!config.validator);
        assert!(config.prover);
        assert!(!config.client);
        assert_eq!(genesis, expected_genesis);

        let mut trusted_peers = vec![];
        let mut trusted_validators = vec![];
        let mut config =
            Start::try_parse_from(["snarkos", "--dev", "3", "--client", "--private-key", ""].iter()).unwrap();
        config.parse_development(&mut trusted_peers, &mut trusted_validators);
        let genesis = config.parse_genesis::<CurrentNetwork>().unwrap();
        assert_eq!(config.node, Some(SocketAddr::from_str("0.0.0.0:4133").unwrap()));
        assert_eq!(config.rest, Some(SocketAddr::from_str("0.0.0.0:3033").unwrap()));
        assert_eq!(trusted_peers.len(), 3);
        assert_eq!(trusted_validators.len(), 2);
        assert!(!config.validator);
        assert!(!config.prover);
        assert!(config.client);
        assert_eq!(genesis, expected_genesis);
    }

    #[test]
    fn clap_snarkos_start() {
        let arg_vec = vec![
            "snarkos",
            "start",
            "--nodisplay",
            "--dev",
            "2",
            "--validator",
            "--private-key",
            "PRIVATE_KEY",
            "--cdn",
            "CDN",
            "--peers",
            "IP1,IP2,IP3",
            "--validators",
            "IP1,IP2,IP3",
            "--rest",
            "127.0.0.1:3030",
        ];
        let cli = CLI::parse_from(arg_vec);

        let Command::Start(start) = cli.command else {
            panic!("Unexpected result of clap parsing!");
        };

        assert!(start.nodisplay);
        assert_eq!(start.dev, Some(2));
        assert!(start.validator);
        assert_eq!(start.private_key.as_deref(), Some("PRIVATE_KEY"));
        assert_eq!(start.cdn, Some(http::Uri::try_from("CDN").unwrap()));
        assert_eq!(start.rest, Some("127.0.0.1:3030".parse().unwrap()));
        assert_eq!(start.network, 0);
        assert_eq!(start.peers, Some("IP1,IP2,IP3".to_string()));
        assert_eq!(start.validators, Some("IP1,IP2,IP3".to_string()));
    }

    #[test]
    fn parse_peers_when_ips() {
        let arg_vec = vec!["snarkos", "start", "--peers", "127.0.0.1:3030,127.0.0.2:3030"];
        let cli = CLI::parse_from(arg_vec);

        if let Command::Start(start) = cli.command {
            let peers = start.parse_trusted_peers();
            assert!(peers.is_ok());
            assert_eq!(peers.unwrap().len(), 2, "Expected two peers");
        } else {
            panic!("Unexpected result of clap parsing!");
        }
    }

    #[test]
    fn parse_peers_when_hostnames() {
        let arg_vec = vec!["snarkos", "start", "--peers", "www.example.com:4130,www.google.com:4130"];
        let cli = CLI::parse_from(arg_vec);

        if let Command::Start(start) = cli.command {
            let peers = start.parse_trusted_peers();
            assert!(peers.is_ok());
            assert_eq!(peers.unwrap().len(), 2, "Expected two peers");
        } else {
            panic!("Unexpected result of clap parsing!");
        }
    }

    #[test]
    fn parse_peers_when_mixed_and_with_whitespaces() {
        let arg_vec = vec!["snarkos", "start", "--peers", "  127.0.0.1:3030,  www.google.com:4130 "];
        let cli = CLI::parse_from(arg_vec);

        if let Command::Start(start) = cli.command {
            let peers = start.parse_trusted_peers();
            assert!(peers.is_ok());
            assert_eq!(peers.unwrap().len(), 2, "Expected two peers");
        } else {
            panic!("Unexpected result of clap parsing!");
        }
    }

    #[test]
    fn parse_peers_when_unknown_hostname_gracefully() {
        let arg_vec = vec!["snarkos", "start", "--peers", "banana.cake.eafafdaeefasdfasd.com"];
        let cli = CLI::parse_from(arg_vec);

        if let Command::Start(start) = cli.command {
            assert!(start.parse_trusted_peers().is_err());
        } else {
            panic!("Unexpected result of clap parsing!");
        }
    }
}
