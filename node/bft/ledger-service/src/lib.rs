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

// See https://github.com/ProvableHQ/snarkVM/issues/2775
#![forbid(unsafe_code)]

#[macro_use]
extern crate async_trait;

#[cfg(feature = "metrics")]
extern crate snarkos_node_metrics as metrics;

mod helpers;
pub use helpers::*;

#[cfg(feature = "ledger")]
pub mod ledger;
#[cfg(feature = "ledger")]
pub use ledger::*;

#[cfg(feature = "mock")]
pub mod mock;
#[cfg(feature = "mock")]
pub use mock::*;

#[cfg(feature = "prover")]
pub mod prover;
#[cfg(feature = "prover")]
pub use prover::*;

#[cfg(feature = "translucent")]
pub mod translucent;
#[cfg(feature = "translucent")]
pub use translucent::*;

pub mod traits;
pub use traits::*;

/// Formats an ID into a truncated identifier (for logging purposes).
pub fn fmt_id(id: impl ToString) -> String {
    let id = id.to_string();
    let mut formatted_id = id.chars().take(16).collect::<String>();
    if id.chars().count() > 16 {
        formatted_id.push_str("..");
    }
    formatted_id
}

/// Records how long a blocking task waited for a thread in the `tokio` blocking pool.
///
/// Only called by [`spawn_blocking!`], which cannot reference the metrics crate directly because
/// its body is expanded in the calling crate.
#[doc(hidden)]
pub fn record_blocking_wait(submitted: std::time::Instant) {
    #[cfg(feature = "metrics")]
    metrics::histogram(metrics::cpu::BLOCKING_WAIT_SECS, submitted.elapsed().as_secs_f64());
    #[cfg(not(feature = "metrics"))]
    let _ = submitted;
}

/// A helper macro to spawn a blocking task.
#[macro_export]
macro_rules! spawn_blocking {
    ($expr:expr) => {{
        let submitted = ::std::time::Instant::now();
        match tokio::task::spawn_blocking(move || {
            $crate::record_blocking_wait(submitted);
            $expr
        })
        .await
        {
            Ok(value) => value,
            Err(error) => Err(snarkvm::prelude::anyhow!("[tokio::spawn_blocking] {error}")),
        }
    }};
}
