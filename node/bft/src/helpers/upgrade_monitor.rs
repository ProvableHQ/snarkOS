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
use snarkvm::{
    console::network::Network,
    ledger::committee::Committee,
    prelude::{Address, FromBytes, Result, ToBytes, bail},
};

#[cfg(feature = "locktick")]
use locktick::parking_lot::Mutex;
#[cfg(not(feature = "locktick"))]
use parking_lot::Mutex;
use std::{
    collections::{HashMap, HashSet},
    fs,
    io,
    path::PathBuf,
};
use tokio::sync::watch;

/// How many blocks past its latest block a validator looks when announcing its consensus version.
pub const UPGRADE_SIGNAL_LOOKAHEAD: u32 = 100;

/// Returns the upgrade signal for a validator whose latest block is at `latest_height`.
///
/// Once a version activates, the signal keeps announcing it, so that a validator still running a
/// build without it detects the upgrade no matter how late it starts.
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
/// threshold of stake announce for a height where this build schedules an older version, with the
/// lowest height among their announcements.
///
/// The availability threshold `(f + 1)` guarantees that at least one honest validator runs a build
/// scheduling that version, so up to `f` faulty validators cannot produce a result on their own.
pub fn required_upgrade<N: Network>(
    committee: &Committee<N>,
    signals: impl IntoIterator<Item = (Address<N>, UpgradeSignal)>,
) -> Option<UpgradeSignal> {
    let mut ahead: Vec<_> = signals.into_iter().filter(|(_, signal)| is_ahead::<N>(signal)).collect();
    ahead.sort_unstable_by_key(|(_, signal)| std::cmp::Reverse(signal.consensus_version));

    // Validators announcing a higher version also vouch for every lower version that is ahead of this build.
    let mut supporters = HashSet::with_capacity(ahead.len());
    let mut height = u32::MAX;
    for (address, signal) in ahead {
        supporters.insert(address);
        height = height.min(signal.height);
        if committee.is_availability_threshold_reached(&supporters) {
            return Some(UpgradeSignal { height, consensus_version: signal.consensus_version });
        }
    }
    None
}

/// Returns `true` if `address` and the validators whose signals are not ahead of this build hold
/// the quorum threshold of stake, which leaves the remaining validators short of the availability threshold.
fn is_upgrade_ruled_out<N: Network>(
    committee: &Committee<N>,
    address: Address<N>,
    signals: impl IntoIterator<Item = (Address<N>, UpgradeSignal)>,
) -> bool {
    let current: HashSet<_> = signals
        .into_iter()
        .filter(|(_, signal)| !is_ahead::<N>(signal))
        .map(|(address, _)| address)
        .chain([address])
        .collect();
    committee.is_quorum_threshold_reached(&current)
}

/// Whether this build may advance the ledger with blocks it builds itself.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpgradeStatus {
    /// No required upgrade is known.
    Clear,
    /// A previous run recorded a required upgrade that this build does not schedule. It stays
    /// pending until connected validators either require the upgrade again or rule it out.
    Pending,
    /// Validators holding the availability threshold of stake run a consensus version this build lacks.
    Required(UpgradeSignal),
}

/// Tracks the upgrade signals of connected validators, and records the signal returned by
/// [`required_upgrade`] in a file, so that the requirement outlives the process.
pub struct UpgradeMonitor<N: Network> {
    /// The address of this validator.
    address: Address<N>,
    /// The file that records a required upgrade.
    record_path: PathBuf,
    /// The latest upgrade signal of each validator.
    ///
    /// Status transitions happen while this lock is held.
    signals: Mutex<HashMap<Address<N>, UpgradeSignal>>,
    /// The current status; once it is `Required`, it never changes.
    status: watch::Sender<UpgradeStatus>,
}

impl<N: Network> UpgradeMonitor<N> {
    /// Initializes the monitor of the validator at `address` from the record at `record_path`.
    ///
    /// The status starts `Pending` if the record holds an upgrade this build does not schedule, or
    /// cannot be read. A record of an upgrade this build schedules is removed.
    pub fn new(address: Address<N>, record_path: PathBuf) -> Self {
        let status = match fs::read(&record_path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => UpgradeStatus::Clear,
            Err(error) => {
                warn!("Unable to read the required upgrade at {} - {error}", record_path.display());
                UpgradeStatus::Pending
            }
            Ok(bytes) => match UpgradeSignal::from_bytes_le(&bytes) {
                Ok(recorded) if Self::schedules(&recorded) => {
                    info!(
                        "This build schedules ConsensusVersion::V{} at height {}, as previously required",
                        recorded.consensus_version, recorded.height
                    );
                    if let Err(error) = fs::remove_file(&record_path) {
                        warn!("Unable to remove the required upgrade at {} - {error}", record_path.display());
                    }
                    UpgradeStatus::Clear
                }
                Ok(recorded) => {
                    warn!(
                        "A previous run found that validators run ConsensusVersion::V{} at height {}, which this build \
                         does not schedule. Blocks are not built until connected validators confirm or refute it.",
                        recorded.consensus_version, recorded.height
                    );
                    UpgradeStatus::Pending
                }
                Err(error) => {
                    warn!("Unable to parse the required upgrade at {} - {error}", record_path.display());
                    UpgradeStatus::Pending
                }
            },
        };
        Self { address, record_path, signals: Default::default(), status: watch::Sender::new(status) }
    }

    /// Returns `true` if this build schedules `signal.consensus_version`, or a later one, at `signal.height`.
    fn schedules(signal: &UpgradeSignal) -> bool {
        N::CONSENSUS_VERSION(signal.height).is_ok_and(|version| version as u16 >= signal.consensus_version)
    }

    /// Records the upgrade signal of the validator at `address`, and returns the required upgrade if
    /// this call found it.
    ///
    /// `lookup` returns the committee and the connected validators. While the status is `Clear`, it
    /// is only called for a signal that is ahead of this build, which keeps the common case cheap.
    /// Signals of validators that are no longer connected are then discarded.
    pub fn record(
        &self,
        address: Address<N>,
        signal: UpgradeSignal,
        lookup: impl FnOnce() -> Option<(Committee<N>, HashSet<Address<N>>)>,
    ) -> Option<UpgradeSignal> {
        let status = self.status();
        if matches!(status, UpgradeStatus::Required(_)) {
            return None;
        }
        if status == UpgradeStatus::Clear && !is_ahead::<N>(&signal) {
            self.signals.lock().remove(&address);
            return None;
        }

        let (committee, connected) = lookup()?;
        let mut signals = self.signals.lock();
        signals.insert(address, signal);
        signals.retain(|address, _| connected.contains(address));
        let current = || signals.iter().map(|(address, signal)| (*address, *signal));

        if let Some(required) = required_upgrade(&committee, current()) {
            return self.require(required);
        }
        if self.status() == UpgradeStatus::Pending && is_upgrade_ruled_out(&committee, self.address, current()) {
            self.clear();
        }
        None
    }

    /// Records `required` and sets the status to `Required`, unless it already is.
    ///
    /// The caller must hold the `signals` lock.
    fn require(&self, required: UpgradeSignal) -> Option<UpgradeSignal> {
        if matches!(self.status(), UpgradeStatus::Required(_)) {
            return None;
        }
        // The record is written before the status changes, as the status change stops the process.
        match required.to_bytes_le() {
            Ok(bytes) => {
                if let Err(error) = fs::write(&self.record_path, bytes) {
                    error!("Unable to record the required upgrade at {} - {error}", self.record_path.display());
                }
            }
            Err(error) => error!("Unable to serialize the required upgrade - {error}"),
        }
        self.status.send_replace(UpgradeStatus::Required(required));
        Some(required)
    }

    /// Removes the record of a required upgrade, and sets the status to `Clear`.
    ///
    /// The caller must hold the `signals` lock.
    fn clear(&self) {
        info!("Connected validators holding a quorum of stake run the consensus versions this build schedules");
        if let Err(error) = fs::remove_file(&self.record_path) {
            warn!("Unable to remove the required upgrade at {} - {error}", self.record_path.display());
        }
        self.status.send_replace(UpgradeStatus::Clear);
    }

    /// Returns the current status.
    pub fn status(&self) -> UpgradeStatus {
        *self.status.borrow()
    }

    /// Returns a receiver that observes the status.
    pub fn subscribe(&self) -> watch::Receiver<UpgradeStatus> {
        self.status.subscribe()
    }

    /// Returns an error unless the status is `Clear`.
    pub fn ensure_blocks_may_be_built(&self) -> Result<()> {
        match self.status() {
            UpgradeStatus::Clear => Ok(()),
            UpgradeStatus::Pending => {
                bail!("Waiting for connected validators to confirm or refute a previously required upgrade")
            }
            UpgradeStatus::Required(required) => {
                bail!("Validators run ConsensusVersion::V{}, which this build lacks", required.consensus_version)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::{
        ledger::committee::test_helpers::sample_committee_for_round_and_members,
        prelude::{ConsensusVersion, MainnetV0, TestRng},
    };

    use rand::RngExt;
    use std::{
        path::Path,
        sync::atomic::{AtomicUsize, Ordering},
    };

    type CurrentNetwork = MainnetV0;

    const HEIGHT: u32 = 1_000_000;

    /// Returns a committee of four validators with equal stake, so that one validator is below the
    /// availability threshold, two reach it, and three reach the quorum threshold.
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

    /// A directory for the record of a required upgrade, removed on drop.
    struct RecordDir(PathBuf);

    impl RecordDir {
        fn new() -> Self {
            static COUNT: AtomicUsize = AtomicUsize::new(0);
            let count = COUNT.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("snarkos-upgrade-monitor-{}-{count}", std::process::id()));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }

        fn record_path(&self) -> PathBuf {
            self.0.join("required-consensus-upgrade")
        }
    }

    impl Drop for RecordDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// Returns a monitor for the last of `addresses`, recording in `dir`.
    fn monitor(addresses: &[Address<CurrentNetwork>], dir: &RecordDir) -> UpgradeMonitor<CurrentNetwork> {
        UpgradeMonitor::new(*addresses.last().unwrap(), dir.record_path())
    }

    fn read_record(path: &Path) -> UpgradeSignal {
        UpgradeSignal::from_bytes_le(&fs::read(path).unwrap()).unwrap()
    }

    fn scheduled_version() -> u16 {
        CurrentNetwork::CONSENSUS_VERSION(HEIGHT).unwrap() as u16
    }

    fn signal(consensus_version: u16) -> UpgradeSignal {
        UpgradeSignal { height: HEIGHT, consensus_version }
    }

    fn version_of(required: Option<UpgradeSignal>) -> Option<u16> {
        required.map(|required| required.consensus_version)
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
        assert_eq!(required_upgrade(&committee, signals), Some(signal(next)));
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
        // The case only exists while this build defines a version it does not schedule at any reachable height.
        let latest = ConsensusVersion::latest();
        if CurrentNetwork::CONSENSUS_VERSION(u32::MAX - 1).unwrap() >= latest {
            return;
        }
        let (committee, addresses) = sample_committee();
        let upgrade = UpgradeSignal { height: u32::MAX - 1, consensus_version: latest as u16 };
        let signals = [(addresses[0], upgrade), (addresses[1], upgrade)];
        assert_eq!(required_upgrade(&committee, signals), Some(upgrade));
    }

    #[test]
    fn the_required_upgrade_has_the_backing_of_the_availability_threshold() {
        let (committee, addresses) = sample_committee();
        let next = scheduled_version() + 1;
        let signals = [(addresses[0], signal(u16::MAX)), (addresses[1], signal(next))];
        assert_eq!(version_of(required_upgrade(&committee, signals)), Some(next));
    }

    #[test]
    fn the_required_upgrade_has_the_lowest_announced_height() {
        let (committee, addresses) = sample_committee();
        let next = scheduled_version() + 1;
        let later = UpgradeSignal { height: HEIGHT + 10, consensus_version: next };
        let signals = [(addresses[0], later), (addresses[1], signal(next))];
        assert_eq!(required_upgrade(&committee, signals), Some(signal(next)));
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
    fn the_local_signal_announces_the_version_scheduled_past_the_tip() {
        // Past the last activation, the signal announces the current version.
        let signal = local_upgrade_signal::<CurrentNetwork>(HEIGHT).unwrap();
        assert_eq!(signal, UpgradeSignal {
            height: HEIGHT + UPGRADE_SIGNAL_LOOKAHEAD,
            consensus_version: scheduled_version()
        });
    }

    #[test]
    fn the_monitor_counts_each_address_once() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        let next = scheduled_version() + 1;
        assert_eq!(monitor.record(addresses[0], signal(next), lookup(&committee, &addresses)), None);
        assert_eq!(monitor.record(addresses[0], signal(next), lookup(&committee, &addresses)), None);
        assert_eq!(monitor.status(), UpgradeStatus::Clear);
    }

    #[test]
    fn the_monitor_ignores_disconnected_validators() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        let next = scheduled_version() + 1;
        monitor.record(addresses[0], signal(next), lookup(&committee, &addresses));
        assert_eq!(monitor.record(addresses[1], signal(next), lookup(&committee, &addresses[1..])), None);
        assert_eq!(monitor.status(), UpgradeStatus::Clear);
    }

    #[test]
    fn the_monitor_forgets_a_validator_that_is_no_longer_ahead() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        let next = scheduled_version() + 1;
        monitor.record(addresses[0], signal(next), lookup(&committee, &addresses));
        monitor.record(addresses[0], signal(scheduled_version()), || unreachable!("the signal is not ahead"));
        assert_eq!(monitor.record(addresses[1], signal(next), lookup(&committee, &addresses)), None);
        assert_eq!(monitor.status(), UpgradeStatus::Clear);
    }

    #[test]
    fn the_monitor_records_the_required_upgrade() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        let receiver = monitor.subscribe();
        let next = scheduled_version() + 1;
        monitor.record(addresses[0], signal(next), lookup(&committee, &addresses));
        assert!(!dir.record_path().exists());
        assert_eq!(monitor.record(addresses[1], signal(next), lookup(&committee, &addresses)), Some(signal(next)));
        assert_eq!(*receiver.borrow(), UpgradeStatus::Required(signal(next)));
        assert_eq!(read_record(&dir.record_path()), signal(next));
        assert!(monitor.ensure_blocks_may_be_built().is_err());

        // Later signals neither clear nor change the required upgrade.
        for address in &addresses {
            assert_eq!(monitor.record(*address, signal(scheduled_version()), lookup(&committee, &addresses)), None);
        }
        assert_eq!(monitor.status(), UpgradeStatus::Required(signal(next)));
        assert_eq!(read_record(&dir.record_path()), signal(next));
    }

    #[test]
    fn a_recorded_upgrade_holds_back_blocks_after_a_restart() {
        let dir = RecordDir::new();
        let (_, addresses) = sample_committee();
        fs::write(dir.record_path(), signal(scheduled_version() + 1).to_bytes_le().unwrap()).unwrap();
        let monitor = monitor(&addresses, &dir);
        assert_eq!(monitor.status(), UpgradeStatus::Pending);
        assert!(monitor.ensure_blocks_may_be_built().is_err());
    }

    #[test]
    fn an_unreadable_record_holds_back_blocks() {
        let dir = RecordDir::new();
        let (_, addresses) = sample_committee();
        fs::write(dir.record_path(), [1u8, 2, 3]).unwrap();
        assert_eq!(monitor(&addresses, &dir).status(), UpgradeStatus::Pending);
    }

    #[test]
    fn a_build_that_schedules_the_recorded_upgrade_removes_the_record() {
        let dir = RecordDir::new();
        let (_, addresses) = sample_committee();
        fs::write(dir.record_path(), signal(scheduled_version()).to_bytes_le().unwrap()).unwrap();
        let monitor = monitor(&addresses, &dir);
        assert_eq!(monitor.status(), UpgradeStatus::Clear);
        assert!(monitor.ensure_blocks_may_be_built().is_ok());
        assert!(!dir.record_path().exists());
    }

    #[test]
    fn a_restarted_monitor_requires_the_upgrade_again() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let next = scheduled_version() + 1;
        fs::write(dir.record_path(), signal(next).to_bytes_le().unwrap()).unwrap();
        let monitor = monitor(&addresses, &dir);
        monitor.record(addresses[0], signal(next), lookup(&committee, &addresses));
        assert_eq!(monitor.status(), UpgradeStatus::Pending);
        assert_eq!(monitor.record(addresses[1], signal(next), lookup(&committee, &addresses)), Some(signal(next)));
        assert_eq!(monitor.status(), UpgradeStatus::Required(signal(next)));
        assert_eq!(read_record(&dir.record_path()), signal(next));
    }

    #[test]
    fn a_quorum_of_current_validators_clears_a_recorded_upgrade() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        fs::write(dir.record_path(), signal(scheduled_version() + 1).to_bytes_le().unwrap()).unwrap();
        let monitor = monitor(&addresses, &dir);

        // This validator and one more hold less than a quorum.
        monitor.record(addresses[0], signal(scheduled_version()), lookup(&committee, &addresses));
        assert_eq!(monitor.status(), UpgradeStatus::Pending);

        // This validator and two more hold a quorum.
        monitor.record(addresses[1], signal(scheduled_version()), lookup(&committee, &addresses));
        assert_eq!(monitor.status(), UpgradeStatus::Clear);
        assert!(monitor.ensure_blocks_may_be_built().is_ok());
        assert!(!dir.record_path().exists());
    }

    #[test]
    fn disconnected_validators_do_not_clear_a_recorded_upgrade() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        fs::write(dir.record_path(), signal(scheduled_version() + 1).to_bytes_le().unwrap()).unwrap();
        let monitor = monitor(&addresses, &dir);
        monitor.record(addresses[0], signal(scheduled_version()), lookup(&committee, &addresses));
        monitor.record(addresses[1], signal(scheduled_version()), lookup(&committee, &addresses[1..]));
        assert_eq!(monitor.status(), UpgradeStatus::Pending);
    }
}
