// Copyright 2024 Aleo Network Foundation
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

use snarkos_cli::{commands::CLI, helpers::Updater};

use clap::Parser;
use std::{env, process::exit};

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
use tikv_jemallocator::Jemalloc;

#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

// Obtain information on the build.
include!(concat!(env!("OUT_DIR"), "/built.rs"));

fn main() -> anyhow::Result<()> {
    // A hack to avoid having to go through clap to display advanced version information.
    check_for_version();

    // Parse the given arguments.
    let cli = CLI::parse();
    // Run the updater.
    println!("{}", Updater::print_cli());
    // Run the CLI.
    match cli.command.parse() {
        Ok(output) => println!("{output}\n"),
        Err(error) => {
            println!("⚠️  {error}\n");
            exit(1);
        }
    }
    Ok(())
}

/// Checks whether the version information was requested and - if so - display it and exit.
fn check_for_version() {
    if let Some(first_arg) = env::args().nth(1) {
        if ["--version", "-V"].contains(&&*first_arg) {
            let version = PKG_VERSION;
            let branch = GIT_HEAD_REF.unwrap_or("unknown_branch");
            let commit = GIT_COMMIT_HASH.unwrap_or("unknown_commit");
            let mut features = FEATURES_LOWERCASE_STR.to_owned();
            features.retain(|c| c != ' ');

            println!("snarkos {version} {branch} {commit} features=[{features}]");

            exit(0);
        }
    }
}
snarkos account sign --raw --private-key-file <aleo1tnzcpgl3hf2sp5u46xenmhr77qdjtutrulkmr9ujmcxmd0k42gyqespul9> -m <b49c8eab9f24502b05a8f0ae65aab1170e813c3e80edb59e7aeef50caa1f247f>
