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

use clap::{Parser, error::ErrorKind};

use snarkos_cli::CLI;

#[test]
fn pass_help_argument() {
    // Test for top level...
    let result = CLI::try_parse_from(["snarkos", "--help"]);
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::DisplayHelp);

    // ...and for sub commands
    let result = CLI::try_parse_from(["snarkos", "account", "-h"]);
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::DisplayHelp);

    let result = CLI::try_parse_from(["snarkos", "account", "--help"]);
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::DisplayHelp);
}

#[test]
fn pass_valid_command() {
    let result = CLI::try_parse_from(["snarkos", "start"]);
    assert!(result.is_ok());
}

#[test]
fn pass_invalid_command() {
    let result = CLI::try_parse_from(["snarkos", "bisect"]);
    let err = result.unwrap_err();
    assert_eq!(err.kind(), ErrorKind::InvalidSubcommand);
}
