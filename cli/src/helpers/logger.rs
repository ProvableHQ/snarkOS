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

use crate::helpers::{DynamicFormatter, LogWriter};

use anyhow::{Result, bail};

use anyhow::Context;
use crossterm::tty::IsTty;
use std::{
    fs::File,
    io,
    path::Path,
    str::FromStr,
    sync::{Arc, atomic::AtomicBool},
};
use time::{UtcDateTime, macros::format_description};
use tokio::sync::mpsc;
use tracing_subscriber::{
    EnvFilter,
    layer::{Layer, SubscriberExt},
    util::SubscriberInitExt,
};

#[cfg(feature = "flamegraph")]
use tracing_flame::FlameLayer;

fn parse_log_verbosity(verbosity: u8) -> Result<EnvFilter> {
    // First, set default log verbosity.
    // Note, that this must not be prefixed with `RUST_LOG=`.
    let default_log_str = match verbosity {
        0 => "info",
        1 => "debug",
        2.. => "trace",
    };
    let filter = EnvFilter::from_str(default_log_str).unwrap();

    // Now, set rules for specific crates.
    let filter = if verbosity >= 2 {
        filter.add_directive("snarkos_node_sync=trace".parse().unwrap())
    } else {
        filter.add_directive("snarkos_node_sync=debug".parse().unwrap())
    };

    let filter = if verbosity >= 3 {
        filter
            .add_directive("snarkos_node_bft=trace".parse().unwrap())
            .add_directive("snarkos_node_bft::gateway=debug".parse().unwrap())
    } else {
        filter.add_directive("snarkos_node_bft=debug".parse().unwrap())
    };

    let filter = if verbosity >= 4 {
        let filter = filter.add_directive("snarkos_node_bft::gateway=trace".parse().unwrap());

        // At high log levels, also show warnings of third-party crates.
        filter
            .add_directive("mio=warn".parse().unwrap())
            .add_directive("tokio_util=warn".parse().unwrap())
            .add_directive("hyper=warn".parse().unwrap())
            .add_directive("reqwest=warn".parse().unwrap())
            .add_directive("want=warn".parse().unwrap())
            .add_directive("h2=warn".parse().unwrap())
            .add_directive("tower=warn".parse().unwrap())
            .add_directive("axum=warn".parse().unwrap())
            .add_directive("ureq=warn".parse().unwrap())
            .add_directive("rustls=warn".parse().unwrap())
    } else {
        let filter = filter.add_directive("snarkos_node_bft::gateway=debug".parse().unwrap());

        // Disable logs from third-party crates by default.
        filter
            .add_directive("mio=off".parse().unwrap())
            .add_directive("tokio_util=off".parse().unwrap())
            .add_directive("hyper=off".parse().unwrap())
            .add_directive("reqwest=off".parse().unwrap())
            .add_directive("want=off".parse().unwrap())
            .add_directive("h2=off".parse().unwrap())
            .add_directive("tower=off".parse().unwrap())
            .add_directive("axum=off".parse().unwrap())
            .add_directive("ureq=off".parse().unwrap())
            .add_directive("rustls=off".parse().unwrap())
    };

    let filter = if verbosity >= 5 {
        filter.add_directive("snarkos_node_router=trace".parse().unwrap())
    } else {
        filter.add_directive("snarkos_node_router=debug".parse().unwrap())
    };

    let filter = if verbosity >= 6 {
        filter.add_directive("snarkos_node_tcp=trace".parse().unwrap())
    } else {
        filter.add_directive("snarkos_node_tcp=off".parse().unwrap())
    };

    Ok(filter)
}

fn parse_log_filter(filter_str: &str) -> Result<EnvFilter> {
    EnvFilter::from_str(filter_str).with_context(|| "Failed to set up log filter")
}

/// Holds any logging state that needs to be kept alive during the nodes execution.
/// Dropping this guard will flush the logs and close the flamegraph file (if any).
#[derive(Default)]
pub struct LogGuard {
    #[cfg(feature = "flamegraph")]
    flame_guard: Option<(String, tracing_flame::FlushGuard<std::io::BufWriter<std::fs::File>>)>,
}

impl Drop for LogGuard {
    fn drop(&mut self) {
        #[cfg(feature = "flamegraph")]
        if let Some((filename, flame_guard)) = &mut self.flame_guard {
            // Flush the flamegraph file and a message on what to do with it.
            match flame_guard.flush() {
                Ok(()) => println!(
                    "Written folded tracing data to disk. Run `inferno-flamegraph {filename} > flamegraph.svg` to generate a graph from it."
                ),
                Err(err) => eprintln!("Failed to flush flamegraph file at {filename}: {err}"),
            }
        }
    }
}

/// Sets the log filter based on the given verbosity level.
///
/// ```ignore
/// 0 => info
/// 1 => info, debug
/// 2 => info, debug, trace, snarkos_node_sync=trace
/// 3 => info, debug, trace, snarkos_node_bft=trace
/// 4 => info, debug, trace, snarkos_node_bft::gateway=trace,
///      [mio|tokio_util|hyper|reqwest|want|h2|tower|axum]=warn
/// 5 => info, debug, trace, snarkos_node_router=trace
/// 6 => info, debug, trace, snarkos_node_tcp=trace
/// ```
///
/// Do not drop the returned log guard until shutdown.
pub fn initialize_logger<P: AsRef<Path>>(
    verbosity: u8,
    log_filter: Option<String>,
    nodisplay: bool,
    logfile: P,
    enable_flamegraph: bool,
    shutdown: Arc<AtomicBool>,
) -> Result<(mpsc::Receiver<Vec<u8>>, LogGuard)> {
    let [stdout_filter, logfile_filter] = std::array::from_fn(|_| {
        if let Some(filter) = &log_filter { parse_log_filter(filter) } else { parse_log_verbosity(verbosity) }
    });

    // Create the directories tree for a logfile if it doesn't exist.
    let Some(logfile_dir) = logfile.as_ref().parent() else { bail!("Root directory passed as a logfile") };

    if !logfile_dir.exists() {
        if let Err(err) = std::fs::create_dir_all(logfile_dir) {
            bail!("Failed to create a directory: '{}' ({err})", logfile_dir.display());
        }
    }
    // Create a file to write logs to.
    let logfile = match File::options().append(true).create(true).open(logfile) {
        Ok(logfile) => logfile,
        Err(err) => bail!("Failed to open the file for writing logs: {err}"),
    };

    // Initialize the log channel.
    let (log_sender, log_receiver) = mpsc::channel(1024);

    // Initialize the log sender.
    // This is only needed when the terminal UI (display) wants to process log output.
    let log_sender = match nodisplay {
        true => None,
        false => Some(log_sender),
    };

    // At high verbosity or when there is a custom log filter we show the target
    // of the log event, i.e., the file/module where the log message was created.
    let show_target = verbosity > 2 || log_filter.is_some();

    // Initialize tracing.
    let registry = tracing_subscriber::registry()
        .with(
            // Add layer using LogWriter for stdout / terminal
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(log_sender.is_none() && io::stdout().is_tty())
                .with_writer(move || LogWriter::new(&log_sender))
                .with_target(show_target)
                .event_format(DynamicFormatter::new(shutdown))
                .with_filter(stdout_filter?),
        )
        .with(
            // Add layer redirecting logs to the file
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(false)
                .with_writer(logfile)
                .with_target(show_target)
                .with_filter(logfile_filter?),
        );

    if enable_flamegraph {
        cfg_if::cfg_if! {
            if #[cfg(feature = "flamegraph")] {
                // Add a timestamp to the trace file name, so we do not overwrite an existing one.
                let timestamp = UtcDateTime::now().format(format_description!("[year][month][day]-[hour][minute][second]")).with_context(|| "Failed to format timestamp")?;
                let trace_file_name = format!("snarkos-trace-{timestamp}.folded");
                let (flame_layer, log_guard) = FlameLayer::with_file(trace_file_name.clone()).with_context(|| format!("Failed to create flamegraph file at {trace_file_name}"))?;
                let _ = registry.with(flame_layer).try_init();

                Ok((log_receiver, LogGuard { flame_guard: Some((trace_file_name, log_guard)) }))
            } else {
                bail!("Cannot enable flamegraph. Disabled at compile time.");
            }
        }
    } else {
        let _ = registry.try_init();
        Ok((log_receiver, LogGuard::default()))
    }
}

/// Set up only terminal logging
pub fn initialize_terminal_logger(verbosity: u8) -> Result<()> {
    let stdout_filter = parse_log_verbosity(verbosity)?;

    // At high verbosity or when there is a custom log filter we show the target
    // of the log event, i.e., the file/module where the log message was created.
    let show_target = verbosity > 2;

    // Initialize tracing.
    let _ = tracing_subscriber::registry()
        .with(
            // Add layer using LogWriter for stdout / terminal
            tracing_subscriber::fmt::Layer::default()
                .with_ansi(io::stdout().is_tty())
                .with_target(show_target)
                .event_format(DynamicFormatter::new(Arc::new(AtomicBool::new(false))))
                .with_filter(stdout_filter),
        )
        .try_init();

    Ok(())
}

/// Returns the welcome message as a string.
pub fn welcome_message() -> String {
    use colored::Colorize;

    let mut output = String::new();
    output += &r#"

         ╦╬╬╬╬╬╦
        ╬╬╬╬╬╬╬╬╬                    ▄▄▄▄        ▄▄▄
       ╬╬╬╬╬╬╬╬╬╬╬                  ▐▓▓▓▓▌       ▓▓▓
      ╬╬╬╬╬╬╬╬╬╬╬╬╬                ▐▓▓▓▓▓▓▌      ▓▓▓     ▄▄▄▄▄▄       ▄▄▄▄▄▄
     ╬╬╬╬╬╬╬╬╬╬╬╬╬╬╬              ▐▓▓▓  ▓▓▓▌     ▓▓▓   ▄▓▓▀▀▀▀▓▓▄   ▐▓▓▓▓▓▓▓▓▌
    ╬╬╬╬╬╬╬╜ ╙╬╬╬╬╬╬╬            ▐▓▓▓▌  ▐▓▓▓▌    ▓▓▓  ▐▓▓▓▄▄▄▄▓▓▓▌ ▐▓▓▓    ▓▓▓▌
   ╬╬╬╬╬╬╣     ╠╬╬╬╬╬╬           ▓▓▓▓▓▓▓▓▓▓▓▓    ▓▓▓  ▐▓▓▀▀▀▀▀▀▀▀▘ ▐▓▓▓    ▓▓▓▌
  ╬╬╬╬╬╬╣       ╠╬╬╬╬╬╬         ▓▓▓▓▌    ▐▓▓▓▓   ▓▓▓   ▀▓▓▄▄▄▄▓▓▀   ▐▓▓▓▓▓▓▓▓▌
 ╬╬╬╬╬╬╣         ╠╬╬╬╬╬╬       ▝▀▀▀▀      ▀▀▀▀▘  ▀▀▀     ▀▀▀▀▀▀       ▀▀▀▀▀▀
╚╬╬╬╬╬╩           ╩╬╬╬╬╩


"#
    .white()
    .bold();
    output += &"👋 Welcome to Aleo! We thank you for running a node and supporting privacy.\n".bold();
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn log_filter() {
        let result = parse_log_filter("=");
        assert!(result.is_err(), "must disallow invalid log filter");

        let result = parse_log_filter("snarkos=trace");
        assert!(result.is_ok(), "must allow valid log filter");
    }
}
