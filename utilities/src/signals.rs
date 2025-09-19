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

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::Notify;

use tracing::{debug, error};

/// Generic trait that can be queried for whether current process should be stopped.
/// This is implemented by `SignalHandler` and `SimpleStoppable`.
pub trait Stoppable: Send + Sync {
    fn stop(&self);
    fn is_stopped(&self) -> bool;
}

/// Wrapper around `AtomicBool` that implements the `Stoppable` trait.
///
/// This is useful when no signal or complex shutdown handling is necessary.
pub struct SimpleStoppable {
    state: AtomicBool,
}

impl SimpleStoppable {
    pub fn new() -> Arc<Self> {
        Arc::new(Self { state: AtomicBool::new(false) })
    }
}

impl Stoppable for SimpleStoppable {
    fn stop(&self) {
        self.state.store(true, Ordering::SeqCst);
    }

    fn is_stopped(&self) -> bool {
        self.state.load(Ordering::SeqCst)
    }
}

/// Helper for signal handling
pub struct SignalHandler {
    stopped: AtomicBool,
    notify: Notify,
}

impl SignalHandler {
    pub fn new() -> Arc<Self> {
        let obj = Arc::new(Self { stopped: AtomicBool::new(false), notify: Default::default() });

        {
            let obj = obj.clone();
            tokio::spawn(async move {
                obj.handle_signals().await;
            });
        }

        obj
    }

    /// Background task that wait for signal.
    async fn handle_signals(&self) {
        #[cfg(target_family = "unix")]
        let signal_listener = async move {
            use tokio::signal::unix::{SignalKind, signal};

            // Handle SIGINT, SIGTERM, SIGQUIT, and SIGHUP.
            let mut s_int = signal(SignalKind::interrupt())?;
            let mut s_term = signal(SignalKind::terminate())?;
            let mut s_quit = signal(SignalKind::quit())?;
            let mut s_hup = signal(SignalKind::hangup())?;

            tokio::select!(
                _ = s_int.recv() => debug!("Received SIGINT"),
                _ = s_term.recv() => debug!("Received SIGTERM"),
                _ = s_quit.recv() => debug!("Received SIGQUIT"),
                _ = s_hup.recv() => debug!("Received SIGHUP"),
            );

            std::io::Result::<()>::Ok(())
        };

        #[cfg(not(target_family = "unix"))]
        let signal_listener = async move {
            tokio::signal::ctrl_c().await?;
            debug!("Received signal");

            std::io::Result::<()>::Ok(())
        };

        // Block until the signal.
        match signal_listener.await {
            Ok(()) => {}
            Err(error) => {
                error!("tokio::signal encountered an error: {error}");
            }
        }

        self.stop();
    }

    /// Blocks until the signal handler was invoked.
    /// Note: This can only be called once, and must not be called concurrently.
    pub async fn wait_for_signals(&self) {
        while !self.is_stopped() {
            self.notify.notified().await
        }
    }
}

impl Stoppable for SignalHandler {
    fn stop(&self) {
        self.stopped.store(true, Ordering::SeqCst);
        self.notify.notify_one();
    }

    fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::SeqCst)
    }
}
