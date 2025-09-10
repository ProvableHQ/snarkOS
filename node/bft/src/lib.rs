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

#![forbid(unsafe_code)]
#![allow(clippy::blocks_in_conditions)]
#![allow(clippy::type_complexity)]

#[macro_use]
extern crate async_trait;
#[macro_use]
extern crate tracing;

#[cfg(feature = "metrics")]
extern crate snarkos_node_metrics as metrics;

pub use snarkos_node_bft_events as events;
pub use snarkos_node_bft_ledger_service as ledger_service;
pub use snarkos_node_bft_storage_service as storage_service;

pub mod helpers;

mod bft;
pub use bft::*;

mod gateway;
pub use gateway::*;

mod primary;
pub use primary::*;

mod sync;
pub use sync::*;

mod worker;
pub use worker::*;

pub const CONTEXT: &str = "[MemoryPool]";

/// The port on which the memory pool listens for incoming connections.
pub const MEMORY_POOL_PORT: u16 = 5000; // port

/// The maximum number of milliseconds to wait before proposing a batch.
pub const MAX_BATCH_DELAY_IN_MS: u64 = 2500; // ms
/// The minimum number of seconds to wait before proposing a batch.
pub const MIN_BATCH_DELAY_IN_SECS: u64 = 1; // seconds
/// The maximum number of milliseconds to wait before timing out on a fetch.
pub const MAX_FETCH_TIMEOUT_IN_MS: u64 = 3 * MAX_BATCH_DELAY_IN_MS; // ms
/// The maximum number of seconds allowed for the leader to send their certificate.
pub const MAX_LEADER_CERTIFICATE_DELAY_IN_SECS: i64 = 2 * MAX_BATCH_DELAY_IN_MS as i64 / 1000; // seconds
/// The maximum number of seconds before the timestamp is considered expired.
pub const MAX_TIMESTAMP_DELTA_IN_SECS: i64 = 10; // seconds
/// The maximum number of workers that can be spawned.
pub const MAX_WORKERS: u8 = 1; // worker(s)

/// The interval at which each primary broadcasts a ping to every other node.
/// Note: If this is updated, be sure to update `MAX_BLOCKS_BEHIND` to correspond properly.
pub const PRIMARY_PING_IN_MS: u64 = 2 * MAX_BATCH_DELAY_IN_MS; // ms
/// The interval at which each worker broadcasts a ping to every other node.
pub const WORKER_PING_IN_MS: u64 = 4 * MAX_BATCH_DELAY_IN_MS; // ms

/// Wrapper around `tokio::spawn_blocking` that awaits the future and propagates panics.
pub async fn execute_blocking<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    match tokio::task::spawn_blocking(f).await {
        Ok(inner) => inner,
        Err(err) => {
            if err.is_panic() {
                // Resume the panic on the main task
                std::panic::resume_unwind(err.into_panic());
            } else {
                panic!("Got unexpected tokio error: {err}");
            }
        }
    }
}

/// A helper macro to spawn a blocking task.
/// It is cleaner to use `execute_blocking` directly.
#[macro_export]
macro_rules! spawn_blocking {
    ($expr:expr) => {
        $crate::execute_blocking(move || $expr).await
    };
}

pub mod errors {
    use colored::Colorize;

    /// Prints an anyhow::Error
    /// Helper function for `log_error` and `log_warning`.
    /// TODO(kaimast): replace with similar logic in snarkvm
    #[inline]
    fn flatten_anyhow_error<E: std::borrow::Borrow<anyhow::Error>>(error: E) -> String {
        let error = error.borrow();
        let chain = error.chain().skip(1).map(|next| next.to_string()).collect::<Vec<String>>().join(" — ");
        format!("{error} — {}", chain.dimmed())
    }

    /// Logs `anyhow::Error`'s its error chain using the `ERROR` log level.
    ///
    /// This follows the existing convention in the codebase that joins errors using em dashes.
    /// For example, an error "Invalid transaction" with a cause "Proof failed"would be logged
    /// as "Invalid transaction — Proof failed".
    /// TODO(kaimast): replace with similar logic in snarkvm
    pub fn log_error<E: std::borrow::Borrow<anyhow::Error>>(error: E) {
        tracing::error!("{}", flatten_anyhow_error(error));
    }

    /// Logs `anyhow::Error`'s its error chain using the `WARN` log level.
    ///
    /// This follows the existing convention in the codebase that joins errors using em dashes.
    /// For example, an error "Invalid transaction" with a cause "Proof failed"would be logged
    /// as "Invalid transaction — Proof failed".
    pub fn log_warning<E: std::borrow::Borrow<anyhow::Error>>(error: E) {
        tracing::warn!("{}", flatten_anyhow_error(error));
    }

    /// Logs `anyhow::Error`'s its error chain using the `DEBUG` log level.
    pub fn log_debug<E: std::borrow::Borrow<anyhow::Error>>(error: E) {
        tracing::debug!("{}", flatten_anyhow_error(error));
    }
}
