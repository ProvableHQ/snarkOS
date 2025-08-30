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

mod account;
pub use account::*;

mod clean;
pub use clean::*;

mod developer;
pub use developer::*;

mod start;
pub use start::*;

mod update;
pub use update::*;

use anstyle::{AnsiColor, Color, Style};
use anyhow::Result;
use clap::{Parser, builder::Styles};

const HEADER_COLOR: Option<Color> = Some(Color::Ansi(AnsiColor::Yellow));
const LITERAL_COLOR: Option<Color> = Some(Color::Ansi(AnsiColor::Green));
const ERROR_COLOR: Option<Color> = Some(Color::Ansi(AnsiColor::Red));
const INVALID_COLOR: Option<Color> = Some(Color::Ansi(AnsiColor::Magenta));

const STYLES: Styles = Styles::plain()
    .header(Style::new().bold().fg_color(HEADER_COLOR))
    .usage(Style::new().bold().fg_color(HEADER_COLOR))
    .error(Style::new().bold().fg_color(ERROR_COLOR))
    .invalid(Style::new().fg_color(INVALID_COLOR))
    .valid(Style::new().bold().fg_color(LITERAL_COLOR))
    .literal(Style::new().bold().fg_color(LITERAL_COLOR));

// The top-level command-line argument.
//
// Metadata is sourced from Cargo.toml. However, the version will be overridden in the main module (snarkos/main.rs).
#[derive(Debug, Parser)]
#[clap(name = "snarkOS", author, about, styles = STYLES, version)]
pub struct CLI {
    /// Specify a subcommand.
    #[clap(subcommand)]
    pub command: Command,

    /// Disable checking for new versions at startup.
    #[clap(long, global = true)]
    pub noupdater: bool,
}

/// The subcommand passed after `snarkos`, e.g. `Start` corresponds to `snarkos start`.
#[derive(Debug, Parser)]
pub enum Command {
    #[clap(subcommand)]
    Account(Account),
    #[clap(name = "clean")]
    Clean(Clean),
    #[clap(name = "developer", alias = "dev")]
    Developer(Box<Developer>),
    #[clap(name = "start")]
    Start(Box<Start>),
    #[clap(name = "update")]
    Update(Update),
}

impl Command {
    /// Runs the given command.
    pub fn parse(self) -> Result<String> {
        match self {
            Self::Account(command) => command.parse(),
            Self::Clean(command) => command.parse(),
            Self::Developer(command) => command.parse(),
            Self::Start(command) => command.parse(),
            Self::Update(command) => command.parse(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // As per the official clap recommendation.
    #[test]
    fn verify_cli() {
        use clap::CommandFactory;
        CLI::command().debug_assert()
    }
}
