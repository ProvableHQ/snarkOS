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

/// A convenience macro that explicitly concatenates an error into an `anyhow::Error` message.
///
/// Use this instead of `anyhow!("... {err}")` to make the intent clear and satisfy the
/// build-time check that disallows silent error concatenation. Prefer `.context()` /
/// `.with_context()` when the original error should be preserved as a cause chain.
#[macro_export]
macro_rules! anyhow_concat {
    ($($arg:tt)*) => { ::anyhow::anyhow!($($arg)*) };
}

/// A convenience macro that explicitly concatenates an error into a `bail!` message.
///
/// Use this instead of `bail!("... {err}")` to make the intent clear and satisfy the
/// build-time check that disallows silent error concatenation. Prefer `.context()` /
/// `.with_context()` when the original error should be preserved as a cause chain.
#[macro_export]
macro_rules! bail_concat {
    ($($arg:tt)*) => { return Err(::anyhow::anyhow!($($arg)*).into()) };
}

/// Prepends a prefix to the message of the top-level `anyhow::Error` while keeping its source
/// chain intact. Unlike `.context()`, this does not add an extra wrapping layer — the prefix is
/// folded into the existing top-level message.
///
/// # Example
/// ```ignore
/// let err = some_fallible_call().map_err(|e| prefix_error("[BlockResponse]", e))?;
/// ```
pub fn prefix_error(prefix: &str, error: anyhow::Error) -> anyhow::Error {
    // Collect the source chain *before* consuming `error`.
    // We stop before the top-level message because we are replacing it.
    let causes: Vec<String> = {
        let mut chain = Vec::new();
        let mut src: Option<&dyn std::error::Error> = error.source();
        while let Some(cause) = src {
            chain.push(cause.to_string());
            src = cause.source();
        }
        chain
    };

    // Build the new top-level message.
    let new_msg = format!("[{prefix}] {error}");

    // If there are no causes we are done.
    if causes.is_empty() {
        return anyhow::anyhow!("{new_msg}");
    }

    // Rebuild from the deepest cause upward, then wrap with the new top message.
    // We use string-based reconstruction because `std::error::Error` sources are not
    // `Send + Sync + 'static` and cannot be re-owned generically.
    let mut rebuilt = anyhow::anyhow!("{}", causes.last().unwrap());
    for cause in causes.iter().rev().skip(1) {
        rebuilt = rebuilt.context(cause.clone());
    }
    rebuilt.context(new_msg)
}

/// Utilities for signal and shutdown handling.
pub mod signals;

pub use signals::*;

pub mod node_data;
pub use node_data::*;
