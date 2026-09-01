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

use crate::locators::BlockLocators;
use snarkos_node_router::Router;
use snarkvm::prelude::Network;

#[cfg(feature = "locktick")]
use locktick::parking_lot::Mutex;
#[cfg(not(feature = "locktick"))]
use parking_lot::Mutex;
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::Arc,
    time::{Duration, Instant},
};
use tokio::{sync::Notify, time::timeout};

/// Internal state of the ping logic
///
/// Essentially, ping keeps a map `next_ping` of peer IPs to the time they should next be pinged.
/// When a new peer connects or a Pong message is received, an entry in next ping is created
/// for when a peer should next be pinged.
struct PingInner<N: Network> {
    /// The next time we should ping a peer.
    ///
    /// Keyed by peer, so a peer holds exactly one entry however many `Pong`s it sends - nothing
    /// checks that a `Pong` was solicited, and an entry per `Pong` would earn the sender a
    /// correspondingly large `Ping` per entry on the next block.
    next_ping: HashMap<SocketAddr, Instant>,
    /// The last time each peer was pinged.
    ///
    /// Entries older than `MAX_PING_INTERVAL` are pruned on every pass: past that point every
    /// peer satisfies `MIN_PING_INTERVAL` anyway, so a missing entry means "may be pinged now".
    last_ping: HashMap<SocketAddr, Instant>,
    /// The most recent block locators.
    /// (or None if this node does not offer block sync)
    block_locators: Option<BlockLocators<N>>,
}

/// Manages sending Ping messages to all connected peers.
pub struct Ping<N: Network> {
    router: Router<N>,
    inner: Arc<Mutex<PingInner<N>>>,
    notify: Arc<Notify>,
}

impl<N: Network> PingInner<N> {
    fn new(block_locators: Option<BlockLocators<N>>) -> Self {
        Self { block_locators, next_ping: Default::default(), last_ping: Default::default() }
    }
}

impl<N: Network> Ping<N> {
    /// The duration in seconds to wait between sending ping requests to a peer.
    const MAX_PING_INTERVAL: Duration = Duration::from_secs(20);
    /// The shortest interval at which a peer may be pinged.
    ///
    /// New blocks trigger a ping to every peer, and during catch-up they arrive in batches limited
    /// only by how fast responses can be processed, so without this floor the rate at which a peer
    /// is pinged is bounded by the round trip to it rather than by anything on this node.
    const MIN_PING_INTERVAL: Duration = Duration::from_secs(1);

    /// Create a new instance of the ping logic.
    /// There should only be one per node.
    ///
    /// # Usage
    /// Initialize this with the most up-to-date block locators and call
    /// update_block_locators, whenever a new block is received/created.
    pub fn new(router: Router<N>, block_locators: BlockLocators<N>) -> Self {
        let notify = Arc::new(Notify::default());
        let inner = Arc::new(Mutex::new(PingInner::new(Some(block_locators))));

        {
            let inner = inner.clone();
            let router_ = router.clone();
            let notify = notify.clone();

            router.spawn(async move {
                Self::ping_task(&inner, &router_, &notify).await;
            });
        }

        Self { inner, router, notify }
    }

    /// Same as [`Self::new`] but for nodes that peers cannot sync from
    /// such as provers.
    pub fn new_nosync(router: Router<N>) -> Self {
        let notify = Arc::new(Notify::default());
        let inner = Arc::new(Mutex::new(PingInner::new(None)));

        {
            let inner = inner.clone();
            let router_ = router.clone();
            let notify = notify.clone();

            router.spawn(async move {
                Self::ping_task(&inner, &router_, &notify).await;
            });
        }

        Self { inner, router, notify }
    }

    /// Notify the ping logic that we received a Pong response.
    pub fn on_pong_received(&self, peer_ip: SocketAddr) {
        let now = Instant::now();
        let mut inner = self.inner.lock();

        inner.next_ping.insert(peer_ip, now + Self::MAX_PING_INTERVAL);

        // self.notify.notify() is not needed as ping_task wakes up every MAX_PING_INTERVAL
    }

    /// Notify the ping logic that a new peer connected.
    pub fn on_peer_connected(&self, peer_ip: SocketAddr) {
        // Send the first ping.
        let now = Instant::now();
        let mut inner = self.inner.lock();
        if !Self::dispatch_ping(now, &mut inner, &self.router, peer_ip) {
            warn!("Peer {peer_ip} connected and immediately disconnected?");
        }
    }

    /// Notify the ping logic that new blocks were created or synced.
    pub fn update_block_locators(&self, locators: BlockLocators<N>) {
        self.inner.lock().block_locators = Some(locators);

        // wake up the ping task
        self.notify.notify_one();
    }

    /// Background task that periodically sends out new ping messages.
    async fn ping_task(inner: &Mutex<PingInner<N>>, router: &Router<N>, notify: &Notify) {
        let mut new_block = false;

        loop {
            if router.ledger().is_stopped() {
                break;
            }

            // Do not hold the lock while waiting.
            let sleep_time = {
                let mut inner = inner.lock();
                let now = Instant::now();

                // Drop ping times too old to constrain anything, bounding the map by peer count.
                inner.last_ping.retain(|_, time| now.saturating_duration_since(*time) < Self::MAX_PING_INTERVAL);

                // Ping peers.
                if new_block {
                    Self::ping_all_peers(now, &mut inner, router);
                    new_block = false;
                } else {
                    Self::ping_expired_peers(now, &mut inner, router);
                }

                // Figure out how long to sleep.
                match inner.next_ping.values().min() {
                    Some(time) => time.saturating_duration_since(now),
                    None => Self::MAX_PING_INTERVAL,
                }
            };

            // wait to be woke up, either by timer or notify
            if timeout(sleep_time, notify.notified()).await.is_ok() {
                // If the timer is not expired, it means we got woken up by a new block.
                new_block = true;
            }
        }
    }

    /// Ping all peers that have an expired timer.
    fn ping_expired_peers(now: Instant, inner: &mut PingInner<N>, router: &Router<N>) {
        let due =
            inner.next_ping.iter().filter(|(_, time)| **time <= now).map(|(peer_ip, _)| *peer_ip).collect::<Vec<_>>();

        Self::ping_peers(now, inner, router, due);
    }

    /// Ping all known peers.
    fn ping_all_peers(now: Instant, inner: &mut PingInner<N>, router: &Router<N>) {
        let peers = inner.next_ping.keys().copied().collect::<Vec<_>>();

        Self::ping_peers(now, inner, router, peers);
    }

    /// Pings each of `peers` that `MIN_PING_INTERVAL` permits, and reschedules the rest for the
    /// moment it does permit them.
    ///
    /// Rescheduling rather than leaving the existing timer is what keeps the floor cheap: the
    /// announcement a suppressed ping would have carried is what a peer most wants when a burst of
    /// new blocks ends, so it is delayed by the floor rather than by the peer's full
    /// `MAX_PING_INTERVAL`.
    fn ping_peers(now: Instant, inner: &mut PingInner<N>, router: &Router<N>, peers: Vec<SocketAddr>) {
        for peer_ip in peers {
            if let Some(earliest) = Self::next_permitted_ping(now, inner, &peer_ip) {
                inner.next_ping.insert(peer_ip, earliest);
                continue;
            }

            inner.next_ping.remove(&peer_ip);
            Self::dispatch_ping(now, inner, router, peer_ip);
        }
    }

    /// Returns the earliest time `peer_ip` may be pinged, or `None` if it may be pinged now.
    ///
    /// A peer with no recorded ping - including one whose entry was pruned for being older than
    /// `MAX_PING_INTERVAL` - may always be pinged.
    fn next_permitted_ping(now: Instant, inner: &PingInner<N>, peer_ip: &SocketAddr) -> Option<Instant> {
        let earliest = *inner.last_ping.get(peer_ip)? + Self::MIN_PING_INTERVAL;
        (earliest > now).then_some(earliest)
    }

    /// Sends a ping to the peer, recording the time so that `MIN_PING_INTERVAL` can be enforced.
    /// Returns `false` if the peer is no longer connected.
    fn dispatch_ping(now: Instant, inner: &mut PingInner<N>, router: &Router<N>, peer_ip: SocketAddr) -> bool {
        let locators = inner.block_locators.clone();

        if router.send_ping(peer_ip, locators) {
            inner.last_ping.insert(peer_ip, now);
            true
        } else {
            // Leave `last_ping` alone: the periodic prune reclaims it, and dropping it here would
            // let a peer that failed once be re-sent to without regard for the floor.
            trace!("Failed to send ping to peer {peer_ip}. Disconnected.");
            false
        }
    }
}
