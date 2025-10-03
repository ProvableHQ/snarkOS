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

pub mod signals;
pub use signals::*;

use std::backtrace::Backtrace;

/// Prints a message using `tracing::error` if logging is enabled, otherwise uses `eprintln`.
#[macro_export]
macro_rules! print_error {
    ($($arg:tt)*) => {
        if tracing::log::log_enabled!(tracing::log::Level::Error) {
            tracing::error!($($arg)*);
        } else {
            eprintln!($($arg)*);
        }
    };
}

#[track_caller]
#[inline]
pub fn show_panic(msg: &str, backtrace: Backtrace) {
    print_error!("⚠️ {msg}\n");

    // Always show backtraces.
    let mut msg = "Backtrace:\n".to_string();
    msg.push_str("      [...]\n");

    // Remove all the low level frames.
    // This can be done more cleanly once the `backtrace_frames` feature is stabilized.
    let backtrace = backtrace.to_string();
    let lines = backtrace.lines().skip_while(|line| !line.contains("core::panicking"));

    for line in lines {
        // Stop printing once we hit the panic handler.
        if line.contains("snarkos::main") {
            break;
        }

        msg.push_str(&format!("{line}\n"));
    }

    // Print the entire backtrace as a single log message.
    print_error!("{msg}");
}
