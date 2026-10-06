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

use snarkos_node_bft_events::UpgradeSignal;
use snarkvm::{console::network::Network, ledger::committee::Committee, prelude::Address};

#[cfg(feature = "locktick")]
use locktick::parking_lot::Mutex;
#[cfg(not(feature = "locktick"))]
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use tokio::sync::watch;

/// How many blocks past its latest block a validator looks when announcing its consensus version.
pub const UPGRADE_SIGNAL_LOOKAHEAD: u32 = 100;

/// Returns the upgrade signal for a validator whose latest block is at `latest_height`.
pub fn local_upgrade_signal<N: Network>(latest_height: u32) -> anyhow::Result<UpgradeSignal> {
    let height = latest_height.saturating_add(UPGRADE_SIGNAL_LOOKAHEAD);
    let consensus_version = N::CONSENSUS_VERSION(height)? as u16;
    Ok(UpgradeSignal { height, consensus_version })
}

/// Returns `true` if `signal` announces a newer consensus version than this build schedules at its height.
fn is_ahead<N: Network>(signal: &UpgradeSignal) -> bool {
    N::CONSENSUS_VERSION(signal.height).is_ok_and(|version| signal.consensus_version > version as u16)
}

/// Returns the highest consensus version that validators holding at least the availability
/// threshold of stake announce for a height where this build schedules an older version.
///
/// The availability threshold `(f + 1)` guarantees that at least one honest validator runs a build
/// scheduling that version, so up to `f` faulty validators cannot produce a result on their own.
pub fn required_upgrade<N: Network>(
    committee: &Committee<N>,
    signals: impl IntoIterator<Item = (Address<N>, UpgradeSignal)>,
) -> Option<u16> {
    let mut ahead: Vec<_> = signals
        .into_iter()
        .filter(|(_, signal)| is_ahead::<N>(signal))
        .map(|(address, signal)| (signal.consensus_version, address))
        .collect();
    ahead.sort_unstable_by_key(|(version, _)| std::cmp::Reverse(*version));

    // Validators announcing a higher version also vouch for every lower version that is ahead of this build.
    let mut supporters = HashSet::with_capacity(ahead.len());
    for (version, address) in ahead {
        supporters.insert(address);
        if committee.is_availability_threshold_reached(&supporters) {
            return Some(version);
        }
    }
    None
}

/// Tracks the upgrade signals of connected validators, and latches the consensus version returned by
/// [`required_upgrade`] the first time there is one.
pub struct UpgradeMonitor<N: Network> {
    /// The latest upgrade signal of each validator, if it is ahead of this build.
    signals: Mutex<HashMap<Address<N>, UpgradeSignal>>,
    /// The latched consensus version; once set, it never changes.
    required_version: watch::Sender<Option<u16>>,
}

impl<N: Network> Default for UpgradeMonitor<N> {
    fn default() -> Self {
        Self { signals: Default::default(), required_version: watch::Sender::new(None) }
    }
}

impl<N: Network> UpgradeMonitor<N> {
    /// Records the upgrade signal of the validator at `address`, and returns the consensus version if
    /// this call latched it.
    ///
    /// `lookup` returns the committee and the connected validators. It is only called for a signal
    /// that is ahead of this build, which keeps the common case cheap. Signals of validators that are
    /// no longer connected are then discarded.
    pub fn record(
        &self,
        address: Address<N>,
        signal: UpgradeSignal,
        lookup: impl FnOnce() -> Option<(Committee<N>, HashSet<Address<N>>)>,
    ) -> Option<u16> {
        if self.required_version().is_some() {
            return None;
        }
        if !is_ahead::<N>(&signal) {
            self.signals.lock().remove(&address);
            return None;
        }

        let (committee, connected) = lookup()?;
        let required = {
            let mut signals = self.signals.lock();
            signals.insert(address, signal);
            signals.retain(|address, _| connected.contains(address));
            required_upgrade(&committee, signals.iter().map(|(address, signal)| (*address, *signal)))?
        };

        self.required_version
            .send_if_modified(|latched| match latched {
                Some(_) => false,
                None => {
                    *latched = Some(required);
                    true
                }
            })
            .then_some(required)
    }

    /// Returns the latched consensus version, if any.
    pub fn required_version(&self) -> Option<u16> {
        *self.required_version.borrow()
    }

    /// Returns a receiver that observes the latched consensus version.
    pub fn subscribe(&self) -> watch::Receiver<Option<u16>> {
        self.required_version.subscribe()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::{
        ledger::committee::test_helpers::sample_committee_for_round_and_members,
        prelude::{MainnetV0, TestRng},
    };

    use rand::RngExt;

    type CurrentNetwork = MainnetV0;

    const HEIGHT: u32 = 1_000_000;

    /// Returns a committee of four validators with equal stake, so that one validator is below the
    /// availability threshold and two reach it.
    fn sample_committee() -> (Committee<CurrentNetwork>, Vec<Address<CurrentNetwork>>) {
        let rng = &mut TestRng::default();
        let addresses: Vec<_> = (0..4).map(|_| Address::new(rng.random())).collect();
        (sample_committee_for_round_and_members(1, addresses.clone(), rng), addresses)
    }

    /// Returns a lookup that reports `committee` and every address in `connected`.
    fn lookup<'a>(
        committee: &'a Committee<CurrentNetwork>,
        connected: &'a [Address<CurrentNetwork>],
    ) -> impl FnOnce() -> Option<(Committee<CurrentNetwork>, HashSet<Address<CurrentNetwork>>)> + 'a {
        move || Some((committee.clone(), connected.iter().copied().collect()))
    }

    fn scheduled_version() -> u16 {
        CurrentNetwork::CONSENSUS_VERSION(HEIGHT).unwrap() as u16
    }

    fn signal(consensus_version: u16) -> UpgradeSignal {
        UpgradeSignal { height: HEIGHT, consensus_version }
    }

    #[test]
    fn a_faulty_minority_cannot_require_an_upgrade() {
        let (committee, addresses) = sample_committee();
        assert_eq!(required_upgrade(&committee, [(addresses[0], signal(u16::MAX))]), None);
    }

    #[test]
    fn the_availability_threshold_requires_an_upgrade() {
        let (committee, addresses) = sample_committee();
        let next = scheduled_version() + 1;
        let signals = [(addresses[0], signal(next)), (addresses[1], signal(next))];
        assert_eq!(required_upgrade(&committee, signals), Some(next));
    }

    #[test]
    fn signals_matching_this_build_do_not_require_an_upgrade() {
        let (committee, addresses) = sample_committee();
        let signals = addresses.iter().map(|address| (*address, signal(scheduled_version())));
        assert_eq!(required_upgrade(&committee, signals), None);
        let signals = addresses.iter().map(|address| (*address, signal(scheduled_version() - 1)));
        assert_eq!(required_upgrade(&committee, signals), None);
    }

    #[test]
    fn an_unscheduled_version_defined_by_this_build_requires_an_upgrade() {
        let (committee, addresses) = sample_committee();
        // This build defines the latest version, but does not schedule it at any reachable height.
        let latest = snarkvm::prelude::ConsensusVersion::latest();
        let height = u32::MAX - 1;
        assert!((CurrentNetwork::CONSENSUS_VERSION(height).unwrap() as u16) < latest as u16);
        let upgrade = UpgradeSignal { height, consensus_version: latest as u16 };
        let signals = [(addresses[0], upgrade), (addresses[1], upgrade)];
        assert_eq!(required_upgrade(&committee, signals), Some(latest as u16));
    }

    #[test]
    fn the_reported_version_has_the_backing_of_the_availability_threshold() {
        let (committee, addresses) = sample_committee();
        let next = scheduled_version() + 1;
        let signals = [(addresses[0], signal(u16::MAX)), (addresses[1], signal(next))];
        assert_eq!(required_upgrade(&committee, signals), Some(next));
    }

    #[test]
    fn non_members_carry_no_stake() {
        let (committee, addresses) = sample_committee();
        let rng = &mut TestRng::default();
        let outsiders: Vec<Address<CurrentNetwork>> = (0..4).map(|_| Address::new(rng.random())).collect();
        let next = scheduled_version() + 1;
        let signals = outsiders.iter().chain(&addresses[..1]).map(|address| (*address, signal(next)));
        assert_eq!(required_upgrade(&committee, signals), None);
    }

    #[test]
    fn the_monitor_counts_each_address_once() {
        let (committee, addresses) = sample_committee();
        let monitor = UpgradeMonitor::default();
        let next = scheduled_version() + 1;
        assert_eq!(monitor.record(addresses[0], signal(next), lookup(&committee, &addresses)), None);
        assert_eq!(monitor.record(addresses[0], signal(next), lookup(&committee, &addresses)), None);
        assert_eq!(monitor.required_version(), None);
    }

    #[test]
    fn the_monitor_ignores_disconnected_validators() {
        let (committee, addresses) = sample_committee();
        let monitor = UpgradeMonitor::default();
        let next = scheduled_version() + 1;
        monitor.record(addresses[0], signal(next), lookup(&committee, &addresses));
        assert_eq!(monitor.record(addresses[1], signal(next), lookup(&committee, &addresses[1..])), None);
        assert_eq!(monitor.required_version(), None);
    }

    #[test]
    fn the_monitor_forgets_a_validator_that_is_no_longer_ahead() {
        let (committee, addresses) = sample_committee();
        let monitor = UpgradeMonitor::default();
        let next = scheduled_version() + 1;
        monitor.record(addresses[0], signal(next), lookup(&committee, &addresses));
        monitor.record(addresses[0], signal(scheduled_version()), || unreachable!("the signal is not ahead"));
        assert_eq!(monitor.record(addresses[1], signal(next), lookup(&committee, &addresses)), None);
        assert_eq!(monitor.required_version(), None);
    }

    #[test]
    fn the_monitor_latches() {
        let (committee, addresses) = sample_committee();
        let monitor = UpgradeMonitor::default();
        let receiver = monitor.subscribe();
        let next = scheduled_version() + 1;
        monitor.record(addresses[0], signal(next), lookup(&committee, &addresses));
        assert_eq!(monitor.record(addresses[1], signal(next), lookup(&committee, &addresses)), Some(next));
        assert_eq!(*receiver.borrow(), Some(next));

        // Later signals neither clear nor change the latched version.
        for address in &addresses {
            assert_eq!(monitor.record(*address, signal(scheduled_version()), lookup(&committee, &addresses)), None);
        }
        assert_eq!(monitor.required_version(), Some(next));
    }
}
