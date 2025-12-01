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

use snarkvm::{
    console::network::{CanaryV0, MainnetV0},
    prelude::{Network, PrivateKey},
};

use anyhow::{Context, Result, anyhow};
use clap::builder::RangedU64ValueParser;
use std::{path::PathBuf, str::FromStr};
use ureq::http::{Uri, uri};

use crate::helpers::dev::get_development_key;

pub(crate) fn network_id_parser() -> RangedU64ValueParser<u16> {
    RangedU64ValueParser::<u16>::new().range((MainnetV0::ID as u64)..=(CanaryV0::ID as u64))
}

/// Make sure a proper scheme (http or https) is set for the endpoint.
pub(crate) fn prepare_endpoint(endpoint: Uri) -> Result<Uri> {
    let mut parts = endpoint.into_parts();

    if parts.scheme.is_none() {
        println!("No scheme given for endpoint. Defaulting to HTTP.");
        parts.scheme = Some(uri::Scheme::HTTP);
    }

    if parts.path_and_query.is_none() {
        // An empty path is fine for the base URL.
        parts.path_and_query = Some(uri::PathAndQuery::from_static(""));
    }

    // Given that the input URI is valid and the scheme we assign is valid, this should never fail.
    Uri::from_parts(parts).with_context(|| "Invalid endpoint URL")
}

/// Fetch the private key, either from the commandline or from a file.
pub(crate) fn parse_private_key<N: Network>(
    cmdline: Option<String>,
    file_name: Option<String>,
    dev_key: Option<u16>,
) -> Result<PrivateKey<N>> {
    if let Some(index) = dev_key {
        let private_key = get_development_key(index)?;
        println!("🔑 Using development private key for node {index} ({private_key})\n");
        return Ok(private_key);
    }

    let key_str = if let Some(keystr) = cmdline {
        keystr
    } else if let Some(file_name) = file_name {
        let path = file_name.parse::<PathBuf>().map_err(|e| anyhow!("Invalid path - {e}"))?;
        std::fs::read_to_string(path).with_context(|| "Failed to read private key from disk")?.trim().to_string()
    } else {
        unreachable!();
    };

    // Retrieve the private key.
    let private_key = PrivateKey::from_str(&key_str).with_context(|| "Failed to parse private key")?;

    Ok(private_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_endpoint() {
        let before = Uri::try_from("localhost:3030").unwrap();
        let after = prepare_endpoint(before).unwrap();
        assert_eq!(after.to_string(), "http://localhost:3030/");
    }
}
