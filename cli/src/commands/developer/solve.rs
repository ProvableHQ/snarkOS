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

use super::Developer;
use crate::helpers::args::{parse_private_key, prepare_endpoint};

use snarkvm::{
    console::network::Network,
    prelude::{Address, Block, puzzle::Puzzle},
};

use snarkvm_circuit_network::Aleo;
use snarkvm_ledger_puzzle_epoch::SynthesisPuzzle;

use anyhow::{Result, anyhow};
use clap::{Parser, builder::NonEmptyStringValueParser};
use colored::Colorize;
use rand::RngCore;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use std::sync::atomic::{AtomicBool, Ordering};
use ureq::http::Uri;
use zeroize::Zeroize;

/// Executes an Aleo program function.
#[derive(Debug, Parser)]
#[command(
    group(clap::ArgGroup::new("key").required(true).multiple(false))
)]
pub struct Solve {
    /// The private key used to generate the solution.
    #[clap(short = 'p', long, group = "key", value_parser=NonEmptyStringValueParser::default())]
    private_key: Option<String>,
    /// Specify the path to a file containing the account private key of the node
    #[clap(long, group = "key", value_parser=NonEmptyStringValueParser::default())]
    private_key_file: Option<String>,
    /// Use a developer validator key to generate the solution
    #[clap(long, group = "key")]
    dev_key: Option<u16>,
    /// The endpoint to query node state from and broadcast to.
    ///
    /// The given value is expected to be the base URL, e.g., "https://mynode.com", and will be extended automatically
    /// to fit the network type and query.
    #[clap(short, long, alias = "query", verbatim_doc_comment)]
    endpoint: Uri,
    /// Concurrency level for solution generation.
    #[clap(long, default_value_t = 1)]
    concurrency: u8,
    /// Check the broadcasted solution.
    #[clap(long)]
    check_solution: bool,
}

impl Drop for Solve {
    /// Zeroize the private key when the `Solve` struct goes out of scope.
    fn drop(&mut self) {
        if let Some(mut pk) = self.private_key.take() {
            pk.zeroize()
        }
    }
}

impl Solve {
    /// Generates a solution to the coinbase puzzle.
    pub fn parse<N: Network, A: Aleo<Network = N>>(self) -> Result<String> {
        let endpoint = prepare_endpoint(self.endpoint.clone())?;

        // Emit a warning if we did not build with feature "test_network".
        if !cfg!(feature = "test_network") {
            println!("⚠️  The 'test_network' feature is not enabled. Finding a solution may take a very long time.");
        }

        // Retrieve the private key.
        let private_key = parse_private_key(self.private_key.clone(), self.private_key_file.clone(), self.dev_key)?;
        let address: Address<N> = private_key.try_into()?;

        // Get the current block from the remote.
        let current_block: Block<N> = match Developer::http_get_json::<N, _>(&endpoint, "block/latest") {
            Ok(Some(s)) => Ok(s),
            Ok(None) => Err(anyhow!("Got unexpected 404 error")),
            Err(err) => Err(err),
        }?;

        // Get the block height, coinbase target and proof target.
        let block_height = current_block.height();
        let proof_target = current_block.proof_target();

        // Calculate the current epoch for the block height.
        let current_epoch = block_height.saturating_div(N::NUM_BLOCKS_PER_EPOCH);

        // Compute the epoch starting height (a multiple of `NUM_BLOCKS_PER_EPOCH`).
        let epoch_starting_height = (current_epoch).saturating_mul(N::NUM_BLOCKS_PER_EPOCH);

        // If the height is 0, return the default block hash.
        let epoch_hash = match epoch_starting_height {
            0 => N::BlockHash::default(),
            _ => {
                let epoch_hash_height = epoch_starting_height.saturating_sub(1);
                let epoch_block: Block<N> =
                    match Developer::http_get_json::<N, _>(&endpoint, &format!("block/{epoch_hash_height}")) {
                        Ok(Some(s)) => Ok(s),
                        Ok(None) => Err(anyhow!("Got unexpected 404 error")),
                        Err(err) => Err(err),
                    }?;
                epoch_block.hash()
            }
        };

        // Initialize an RNG and puzzle.
        let rng = &mut rand::thread_rng();
        let puzzle = Puzzle::<N>::new::<SynthesisPuzzle<N, A>>();

        let check_solution_query_parameter =
            if self.check_solution { "?check_solution=true" } else { Default::default() };

        // Run the core mining loop.
        loop {
            // Initialize a random range to search.
            let start = rng.next_u64();
            let end = start.saturating_add(self.concurrency as u64);

            let solution_found = AtomicBool::new(false);

            // Iterate over the counter range and generate solutions concurrently.
            (start..end).into_par_iter().for_each(|count| {
                // Generate a solution and attempt to broadcast it.
                let solution = puzzle.prove(epoch_hash, address, count, Some(proof_target));

                match solution {
                    Ok(solution) => {
                        let broadcast_endpoint = Developer::build_endpoint::<N>(
                            &endpoint,
                            &format!("solution/broadcast{check_solution_query_parameter}"),
                        )
                        .unwrap();

                        let response: String = match Developer::http_post_json(&broadcast_endpoint, &solution) {
                            Ok(Some(s)) => Ok(s),
                            Ok(None) => Err(anyhow!("Got unexpected 404 error")),
                            Err(err) => Err(err),
                        }
                        .unwrap();
                        println!("✅ Created solution for '{}' with id: {}", address.to_string().bold(), response);
                        solution_found.store(true, Ordering::Relaxed);
                    }
                    Err(e) => {
                        println!("Failed to generate solution that meets target: {e:?}");
                    }
                }
            });

            if solution_found.load(Ordering::Relaxed) {
                break;
            }
        }
        Ok("Solution found".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{CLI, Command, DeveloperCommand};
    use anyhow::bail;

    #[test]
    fn clap_snarkos_solve() -> Result<()> {
        let arg_vec = &["snarkos", "developer", "solve", "--private-key", "PRIVATE_KEY", "--endpoint=ENDPOINT"];
        let cli = CLI::try_parse_from(arg_vec)?;

        let Command::Developer(developer) = cli.command else {
            bail!("Unexpected result of clap parsing!");
        };
        let DeveloperCommand::Solve(solve) = developer.command else {
            bail!("Unexpected result of clap parsing!");
        };

        assert_eq!(developer.network, 0);
        assert_eq!(solve.private_key, Some("PRIVATE_KEY".to_string()));
        assert_eq!(solve.endpoint, "ENDPOINT");

        Ok(())
    }

    #[test]
    fn clap_snarkos_solve_pk_file() -> Result<()> {
        let arg_vec =
            &["snarkos", "developer", "solve", "--private-key-file", "PRIVATE_KEY_FILE", "--endpoint=ENDPOINT"];
        let cli = CLI::try_parse_from(arg_vec)?;

        let Command::Developer(developer) = cli.command else {
            bail!("Unexpected result of clap parsing!");
        };
        let DeveloperCommand::Solve(solve) = developer.command else {
            bail!("Unexpected result of clap parsing!");
        };

        assert_eq!(developer.network, 0);
        assert_eq!(solve.private_key_file, Some("PRIVATE_KEY_FILE".to_string()));
        assert_eq!(solve.endpoint, "ENDPOINT");

        Ok(())
    }
}
