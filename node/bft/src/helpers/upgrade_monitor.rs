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

use snarkos_node_bft_events::ConsensusSchedule;
use snarkvm::{
    console::network::Network,
    ledger::committee::Committee,
    prelude::{Address, Field, FromBytes, IoResult, Read, Result, ToBytes, Write, bail},
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
    time::{Duration, Instant},
};
use tokio::sync::watch;

/// How often the monitor asks for a scheduled upgrade to be logged.
const WARNING_INTERVAL: Duration = Duration::from_secs(600);

/// A consensus version that validators holding the availability threshold of stake activate at a
/// height where this build schedules an older one.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequiredUpgrade {
    pub height: u32,
    pub consensus_version: u16,
}

impl ToBytes for RequiredUpgrade {
    fn write_le<W: Write>(&self, mut writer: W) -> IoResult<()> {
        self.height.write_le(&mut writer)?;
        self.consensus_version.write_le(&mut writer)
    }
}

impl FromBytes for RequiredUpgrade {
    fn read_le<R: Read>(mut reader: R) -> IoResult<Self> {
        let height = u32::read_le(&mut reader)?;
        let consensus_version = u16::read_le(&mut reader)?;
        Ok(Self { height, consensus_version })
    }
}

/// A consensus schedule as a step function of the height.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Steps(Vec<(u32, u16)>);

impl Steps {
    /// Returns the steps of `schedule`, in ascending order of height and of version.
    ///
    /// Peers send their schedules unchecked, so an entry that does not raise the version is dropped,
    /// which makes [`Self::version_at`] the highest version activated at or below a height.
    fn new(schedule: &ConsensusSchedule) -> Self {
        let mut entries: Vec<(u32, u16)> = schedule
            .heights()
            .iter()
            .zip(1u16..)
            .filter(|(height, _)| **height != u32::MAX)
            .map(|(height, version)| (*height, version))
            .collect();
        entries.sort_unstable();

        let mut steps: Vec<(u32, u16)> = Vec::with_capacity(entries.len());
        for (height, version) in entries {
            match steps.last_mut() {
                Some((_, last_version)) if version <= *last_version => {}
                Some((last_height, last_version)) if height == *last_height => *last_version = version,
                _ => steps.push((height, version)),
            }
        }
        Self(steps)
    }

    /// Returns the consensus version at `height`, or `0` if none is active.
    fn version_at(&self, height: u32) -> u16 {
        match self.0.partition_point(|(step_height, _)| *step_height <= height) {
            0 => 0,
            index => self.0[index - 1].1,
        }
    }

    /// Returns the version that activates at exactly `height`, if any.
    fn activation_at(&self, height: u32) -> Option<u16> {
        self.0.binary_search_by_key(&height, |(step_height, _)| *step_height).ok().map(|index| self.0[index].1)
    }

    /// Returns the height at which `version`, or a later version, activates, if any.
    fn height_of(&self, version: u16) -> Option<u32> {
        self.0.iter().find(|(_, step_version)| *step_version >= version).map(|(height, _)| *height)
    }
}

/// Whether validators run a consensus version this build lacks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UpgradeStatus {
    /// No upgrade is required.
    Clear,
    /// An upgrade is required before the next block to build.
    Scheduled(RequiredUpgrade),
    /// An upgrade is required at or before the next block to build. Once set, it never changes.
    Required(RequiredUpgrade),
}

/// The evidence the monitor keeps.
struct State<N: Network> {
    /// The steps of the schedule each validator disclosed in its latest handshake.
    ///
    /// The steps of a validator that disconnects are kept: its schedule is fixed by its build, and
    /// dropping it could hide a required upgrade at the moment its height is reached.
    schedules: HashMap<Address<N>, Steps>,
    /// The required upgrade that a previous run recorded, until connected validators refute it.
    recorded: Option<RequiredUpgrade>,
    /// The required upgrade found in `schedules`, with the ID of the committee it was found for.
    cached: Option<(Field<N>, Option<RequiredUpgrade>)>,
    /// When the monitor last asked for the scheduled upgrade to be logged.
    warned_at: Option<Instant>,
}

/// Tracks the consensus schedules of validators, and decides whether this build may build a block.
///
/// A required upgrade is written to a file before the status becomes `Required`, so that the
/// requirement outlives the process.
pub struct UpgradeMonitor<N: Network> {
    /// The address of this validator.
    address: Address<N>,
    /// The steps of this build's schedule.
    own: Steps,
    /// The file that records a required upgrade.
    record_path: PathBuf,
    /// The evidence; status transitions happen while this lock is held.
    state: Mutex<State<N>>,
    /// The current status.
    status: watch::Sender<UpgradeStatus>,
}

impl<N: Network> UpgradeMonitor<N> {
    /// Initializes the monitor of the validator at `address` from the record at `record_path`.
    ///
    /// A record of an upgrade that this build schedules is removed. Any other record, including one
    /// that cannot be read, holds back the blocks at and above its height until connected validators
    /// refute it.
    pub fn new(address: Address<N>, record_path: PathBuf) -> Self {
        let own = Steps::new(&ConsensusSchedule::of::<N>());
        // A record that cannot be read holds back every block.
        let unreadable = RequiredUpgrade { height: 0, consensus_version: u16::MAX };
        let recorded = match fs::read(&record_path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => {
                warn!("Unable to read the required upgrade at {} - {error}", record_path.display());
                Some(unreadable)
            }
            Ok(bytes) => match RequiredUpgrade::from_bytes_le(&bytes) {
                Ok(recorded) if own.version_at(recorded.height) >= recorded.consensus_version => {
                    info!(
                        "This build schedules ConsensusVersion::V{} at height {}, as previously required",
                        recorded.consensus_version, recorded.height
                    );
                    if let Err(error) = fs::remove_file(&record_path) {
                        warn!("Unable to remove the required upgrade at {} - {error}", record_path.display());
                    }
                    None
                }
                Ok(recorded) => {
                    warn!(
                        "A previous run found that validators run ConsensusVersion::V{} at height {}, which this build \
                         does not schedule. No block at or above that height is built until connected validators \
                         confirm or refute it.",
                        recorded.consensus_version, recorded.height
                    );
                    Some(recorded)
                }
                Err(error) => {
                    warn!("Unable to parse the required upgrade at {} - {error}", record_path.display());
                    Some(unreadable)
                }
            },
        };
        let state = State { schedules: Default::default(), recorded, cached: None, warned_at: None };
        Self { address, own, record_path, state: Mutex::new(state), status: watch::Sender::new(UpgradeStatus::Clear) }
    }

    /// Records the schedule that the validator at `address` disclosed in a completed handshake, or
    /// forgets its previous one if it disclosed none.
    pub fn record_schedule(&self, address: Address<N>, schedule: Option<&ConsensusSchedule>) {
        let mut state = self.state.lock();
        match schedule {
            Some(schedule) => state.schedules.insert(address, Steps::new(schedule)),
            None => state.schedules.remove(&address),
        };
        state.cached = None;
    }

    /// Re-evaluates the status for the next block to build, at `next_height`.
    ///
    /// `connected` holds the validators that are connected now. A record is removed once this
    /// validator and the connected validators that are not ahead of it at the recorded height hold
    /// the quorum threshold of stake.
    pub fn update(&self, committee: &Committee<N>, connected: &HashSet<Address<N>>, next_height: u32) -> UpgradeStatus {
        let mut state = self.state.lock();
        if let UpgradeStatus::Required(required) = self.status() {
            return UpgradeStatus::Required(required);
        }

        if let Some(recorded) = state.recorded
            && self.is_refuted(&state, committee, connected, recorded.height)
        {
            info!(
                "Connected validators holding a quorum of stake do not run ConsensusVersion::V{} at height {}",
                recorded.consensus_version, recorded.height
            );
            if let Err(error) = fs::remove_file(&self.record_path) {
                warn!("Unable to remove the required upgrade at {} - {error}", self.record_path.display());
            }
            state.recorded = None;
        }

        let required = match state.cached {
            Some((committee_id, required)) if committee_id == committee.id() => required,
            _ => {
                let required = self.required_upgrade(&state, committee);
                state.cached = Some((committee.id(), required));
                required
            }
        };

        let status = match required {
            Some(required) if required.height <= next_height => {
                // The record is written before the status changes, as the status change stops the process.
                match required.to_bytes_le() {
                    Ok(bytes) => {
                        if let Err(error) = fs::write(&self.record_path, bytes) {
                            error!("Unable to record the required upgrade at {} - {error}", self.record_path.display());
                        }
                    }
                    Err(error) => error!("Unable to serialize the required upgrade - {error}"),
                }
                UpgradeStatus::Required(required)
            }
            Some(required) => UpgradeStatus::Scheduled(required),
            None => UpgradeStatus::Clear,
        };
        self.status.send_if_modified(|current| {
            let modified = *current != status;
            *current = status;
            modified
        });
        status
    }

    /// Returns an error unless this build may build the block at `height`.
    ///
    /// Beyond a required upgrade, a block is held back until this validator and the connected
    /// validators that are not ahead of it at that height hold the quorum threshold of stake, if
    /// either:
    /// - a committee member's schedule activates a newer version than this build's at exactly that
    ///   height, or
    /// - a record claims a newer version at or below that height.
    ///
    /// Connected validators that disclosed no schedule count as not ahead. A member's schedule
    /// counts only at its activation heights; a schedule that activated a version below every
    /// height would otherwise let one faulty member hold back every block.
    pub fn ensure_block_may_be_built(
        &self,
        committee: &Committee<N>,
        connected: &HashSet<Address<N>>,
        height: u32,
    ) -> Result<()> {
        if let UpgradeStatus::Required(required) = self.update(committee, connected, height) {
            bail!(
                "Validators run ConsensusVersion::V{} from height {}, which this build lacks",
                required.consensus_version,
                required.height
            );
        }

        let state = self.state.lock();
        let own_version = self.own.version_at(height);
        let is_claimed = state.recorded.is_some_and(|recorded| recorded.height <= height)
            || state.schedules.iter().any(|(address, steps)| {
                committee.get_stake(*address) > 0
                    && steps.activation_at(height).is_some_and(|version| version > own_version)
            });
        if is_claimed && !self.is_refuted(&state, committee, connected, height) {
            bail!(
                "Validators may run a newer consensus version than this build at height {height}, and connected \
                 validators holding a quorum of stake have not refuted it"
            );
        }
        Ok(())
    }

    /// Returns `true` if this validator and the connected validators that are not ahead of this
    /// build at `height` hold the quorum threshold of stake, which leaves the remaining validators
    /// short of the availability threshold.
    fn is_refuted(
        &self,
        state: &State<N>,
        committee: &Committee<N>,
        connected: &HashSet<Address<N>>,
        height: u32,
    ) -> bool {
        let own_version = self.own.version_at(height);
        let current: HashSet<_> = connected
            .iter()
            .filter(|address| state.schedules.get(*address).is_none_or(|steps| steps.version_at(height) <= own_version))
            .copied()
            .chain([self.address])
            .collect();
        committee.is_quorum_threshold_reached(&current)
    }

    /// Returns the lowest height at which committee members holding the availability threshold of
    /// stake are ahead of this build, with the highest version they share there.
    ///
    /// The threshold `(f + 1)` guarantees that at least one honest validator runs a build that
    /// schedules that version at that height, so up to `f` faulty validators can neither produce a
    /// result on their own nor move it to an earlier height.
    fn required_upgrade(&self, state: &State<N>, committee: &Committee<N>) -> Option<RequiredUpgrade> {
        let members: Vec<_> =
            state.schedules.iter().filter(|(address, _)| committee.get_stake(**address) > 0).collect();

        // The set of validators ahead of this build only grows at the height of one of their steps.
        let mut heights: Vec<u32> =
            members.iter().flat_map(|(_, steps)| steps.0.iter().map(|(height, _)| *height)).collect();
        heights.sort_unstable();
        heights.dedup();

        for height in heights {
            let own_version = self.own.version_at(height);
            let mut ahead: Vec<_> = members
                .iter()
                .map(|(address, steps)| (steps.version_at(height), **address))
                .filter(|(version, _)| *version > own_version)
                .collect();
            ahead.sort_unstable_by_key(|(version, _)| std::cmp::Reverse(*version));

            // A validator ahead with a higher version also vouches for every lower one that is ahead.
            let mut supporters = HashSet::with_capacity(ahead.len());
            for (version, address) in ahead {
                supporters.insert(address);
                if committee.is_availability_threshold_reached(&supporters) {
                    return Some(RequiredUpgrade { height, consensus_version: version });
                }
            }
        }
        None
    }

    /// Returns the height at which this build schedules `consensus_version`, if it does.
    pub fn own_height_of(&self, consensus_version: u16) -> Option<u32> {
        self.own.height_of(consensus_version)
    }

    /// Returns `true`, at most once per [`WARNING_INTERVAL`], if a scheduled upgrade should be logged.
    pub fn should_warn(&self) -> bool {
        if !matches!(self.status(), UpgradeStatus::Scheduled(_)) {
            return false;
        }
        let mut state = self.state.lock();
        let should_warn = state.warned_at.is_none_or(|warned_at| warned_at.elapsed() >= WARNING_INTERVAL);
        if should_warn {
            state.warned_at = Some(Instant::now());
        }
        should_warn
    }

    /// Returns the current status.
    pub fn status(&self) -> UpgradeStatus {
        *self.status.borrow()
    }

    /// Returns a receiver that observes the status.
    pub fn subscribe(&self) -> watch::Receiver<UpgradeStatus> {
        self.status.subscribe()
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
    use std::sync::atomic::{AtomicUsize, Ordering};

    type CurrentNetwork = MainnetV0;

    /// A height past every activation this build schedules.
    const H: u32 = u32::MAX - 1_000;

    /// Returns a committee of four validators with equal stake, so that one validator is below the
    /// availability threshold, two reach it, and three reach the quorum threshold. The monitor runs
    /// as the last one.
    fn sample_committee() -> (Committee<CurrentNetwork>, Vec<Address<CurrentNetwork>>) {
        let rng = &mut TestRng::default();
        let addresses: Vec<_> = (0..4).map(|_| Address::new(rng.random())).collect();
        (sample_committee_for_round_and_members(1, addresses.clone(), rng), addresses)
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

        fn write(&self, bytes: &[u8]) {
            fs::write(self.record_path(), bytes).unwrap();
        }

        fn read(&self) -> Option<RequiredUpgrade> {
            fs::read(self.record_path()).ok().map(|bytes| RequiredUpgrade::from_bytes_le(&bytes).unwrap())
        }
    }

    impl Drop for RecordDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn monitor(addresses: &[Address<CurrentNetwork>], dir: &RecordDir) -> UpgradeMonitor<CurrentNetwork> {
        UpgradeMonitor::new(*addresses.last().unwrap(), dir.record_path())
    }

    fn own_schedule() -> ConsensusSchedule {
        ConsensusSchedule::of::<CurrentNetwork>()
    }

    /// The version that a peer schedules beyond every version of this build.
    fn next_version() -> u16 {
        own_schedule().heights().len() as u16 + 1
    }

    /// Returns this build's schedule, plus a version beyond it at `height`.
    fn ahead_at(height: u32) -> ConsensusSchedule {
        let mut heights = own_schedule().heights().to_vec();
        heights.push(height);
        ConsensusSchedule::new(heights).unwrap()
    }

    fn connected(addresses: &[Address<CurrentNetwork>]) -> HashSet<Address<CurrentNetwork>> {
        addresses.iter().copied().collect()
    }

    fn required(height: u32) -> RequiredUpgrade {
        RequiredUpgrade { height, consensus_version: next_version() }
    }

    #[test]
    fn steps_are_independent_of_the_order_of_the_table() {
        let steps = Steps::new(&ConsensusSchedule::new(vec![0, 30, 20, u32::MAX, 10, 30]).unwrap());
        // Versions 2 and 3 activate after version 5, so they never raise the version.
        assert_eq!(steps.0, vec![(0, 1), (10, 5), (30, 6)]);
        assert_eq!(steps.version_at(0), 1);
        assert_eq!(steps.version_at(9), 1);
        assert_eq!(steps.version_at(10), 5);
        assert_eq!(steps.version_at(29), 5);
        assert_eq!(steps.version_at(u32::MAX), 6);
        assert_eq!(steps.height_of(2), Some(10));
        assert_eq!(steps.height_of(7), None);
    }

    #[test]
    fn an_empty_or_unscheduled_table_has_no_version() {
        assert_eq!(Steps::new(&ConsensusSchedule::new(vec![]).unwrap()).version_at(u32::MAX), 0);
        assert_eq!(Steps::new(&ConsensusSchedule::new(vec![u32::MAX; 3]).unwrap()).version_at(u32::MAX), 0);
    }

    #[test]
    fn a_faulty_minority_cannot_require_an_upgrade() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&ahead_at(0)));
        assert_eq!(monitor.update(&committee, &connected(&addresses), H), UpgradeStatus::Clear);
    }

    #[test]
    fn the_availability_threshold_schedules_an_upgrade() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&ahead_at(H)));
        monitor.record_schedule(addresses[1], Some(&ahead_at(H)));
        assert_eq!(monitor.update(&committee, &connected(&addresses), H - 1), UpgradeStatus::Scheduled(required(H)));
        assert_eq!(dir.read(), None);
    }

    #[test]
    fn faulty_validators_cannot_move_the_upgrade_earlier() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&ahead_at(H - 500)));
        monitor.record_schedule(addresses[1], Some(&ahead_at(H)));
        assert_eq!(monitor.update(&committee, &connected(&addresses), 0), UpgradeStatus::Scheduled(required(H)));
    }

    #[test]
    fn a_postponed_version_schedules_an_upgrade() {
        // The last version this build schedules, moved earlier.
        let own = own_schedule();
        let (index, own_height) = own.heights().iter().enumerate().rfind(|(_, height)| **height != u32::MAX).unwrap();
        assert!(*own_height >= 5);
        let mut heights = own.heights().to_vec();
        heights[index] = own_height - 5;
        let earlier = ConsensusSchedule::new(heights).unwrap();

        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&earlier));
        monitor.record_schedule(addresses[1], Some(&earlier));
        let expected = RequiredUpgrade { height: own_height - 5, consensus_version: index as u16 + 1 };
        assert_eq!(monitor.update(&committee, &connected(&addresses), 0), UpgradeStatus::Scheduled(expected));
        assert_eq!(monitor.own_height_of(expected.consensus_version), Some(*own_height));
    }

    #[test]
    fn non_members_carry_no_stake() {
        let (committee, addresses) = sample_committee();
        let rng = &mut TestRng::default();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        for _ in 0..4 {
            monitor.record_schedule(Address::new(rng.random()), Some(&ahead_at(H)));
        }
        monitor.record_schedule(addresses[0], Some(&ahead_at(H)));
        assert_eq!(monitor.update(&committee, &connected(&addresses), H), UpgradeStatus::Clear);
    }

    #[test]
    fn a_handshake_without_a_schedule_replaces_the_previous_one() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&ahead_at(H)));
        monitor.record_schedule(addresses[1], Some(&ahead_at(H)));
        monitor.record_schedule(addresses[1], None);
        assert_eq!(monitor.update(&committee, &connected(&addresses), H), UpgradeStatus::Clear);
    }

    #[test]
    fn the_upgrade_is_required_at_its_height_and_recorded() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        let receiver = monitor.subscribe();
        monitor.record_schedule(addresses[0], Some(&ahead_at(H)));
        monitor.record_schedule(addresses[1], Some(&ahead_at(H)));

        assert!(monitor.ensure_block_may_be_built(&committee, &connected(&addresses), H - 1).is_ok());
        assert!(monitor.ensure_block_may_be_built(&committee, &connected(&addresses), H).is_err());
        assert_eq!(*receiver.borrow(), UpgradeStatus::Required(required(H)));
        assert_eq!(dir.read(), Some(required(H)));

        // Once required, the upgrade stays required.
        monitor.record_schedule(addresses[0], None);
        monitor.record_schedule(addresses[1], None);
        assert_eq!(monitor.update(&committee, &connected(&addresses), 0), UpgradeStatus::Required(required(H)));
    }

    #[test]
    fn schedules_of_disconnected_validators_still_count() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&ahead_at(H)));
        monitor.record_schedule(addresses[1], Some(&ahead_at(H)));
        assert_eq!(monitor.update(&committee, &HashSet::new(), H), UpgradeStatus::Required(required(H)));
    }

    #[test]
    fn a_block_is_held_back_without_a_quorum_where_a_member_is_ahead() {
        // Validator 0 is ahead and connected, validator 1 discloses no schedule, and validator 2 is
        // ahead but never connected, so too little stake is known to be ahead to require an upgrade.
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&ahead_at(H)));
        let connected = connected(&addresses[..2]);

        assert!(monitor.ensure_block_may_be_built(&committee, &connected, H - 1).is_ok());
        assert!(monitor.ensure_block_may_be_built(&committee, &connected, H).is_err());
        assert_eq!(monitor.status(), UpgradeStatus::Clear);
    }

    #[test]
    fn a_member_ahead_holds_back_only_the_block_at_its_activation() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&ahead_at(H)));
        let connected = connected(&addresses[..2]);

        assert!(monitor.ensure_block_may_be_built(&committee, &connected, H).is_err());
        assert!(monitor.ensure_block_may_be_built(&committee, &connected, H + 1).is_ok());
    }

    #[test]
    fn a_member_that_activates_a_version_at_genesis_holds_back_no_block() {
        // Without enough connected stake to refute it, this would hold back every block.
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&ahead_at(0)));
        let connected = connected(&addresses[..2]);

        for height in [1, 1_000, H] {
            assert!(monitor.ensure_block_may_be_built(&committee, &connected, height).is_ok());
        }
        assert_eq!(monitor.status(), UpgradeStatus::Clear);
    }

    #[test]
    fn a_quorum_that_is_not_ahead_lets_a_block_be_built() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&ahead_at(H)));
        monitor.record_schedule(addresses[1], Some(&own_schedule()));
        assert!(monitor.ensure_block_may_be_built(&committee, &connected(&addresses), H).is_ok());
    }

    #[test]
    fn a_block_is_built_when_no_validator_is_known_to_be_ahead() {
        // A known limit: a validator that never hears from one that is ahead builds the block.
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        assert!(monitor.ensure_block_may_be_built(&committee, &connected(&addresses[..1]), H).is_ok());
    }

    #[test]
    fn a_build_that_schedules_the_recorded_upgrade_removes_the_record() {
        let (_, addresses) = sample_committee();
        let dir = RecordDir::new();
        let height = own_schedule().heights()[0];
        dir.write(&RequiredUpgrade { height, consensus_version: 1 }.to_bytes_le().unwrap());
        let _monitor = monitor(&addresses, &dir);
        assert_eq!(dir.read(), None);
    }

    #[test]
    fn a_record_holds_back_blocks_from_its_height_until_refuted() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        dir.write(&required(H).to_bytes_le().unwrap());
        let monitor = monitor(&addresses, &dir);

        // Below the recorded height, blocks are built.
        assert!(monitor.ensure_block_may_be_built(&committee, &HashSet::new(), H - 1).is_ok());
        // At the recorded height, this validator alone holds less than a quorum.
        assert!(monitor.ensure_block_may_be_built(&committee, &HashSet::new(), H).is_err());
        // Two connected validators that are not ahead make a quorum with this one.
        monitor.record_schedule(addresses[0], Some(&own_schedule()));
        assert!(monitor.ensure_block_may_be_built(&committee, &connected(&addresses[..2]), H).is_ok());
        assert_eq!(dir.read(), None);
    }

    #[test]
    fn a_record_is_confirmed_by_the_availability_threshold() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        dir.write(&required(H).to_bytes_le().unwrap());
        let monitor = monitor(&addresses, &dir);
        monitor.record_schedule(addresses[0], Some(&ahead_at(H)));
        monitor.record_schedule(addresses[1], Some(&ahead_at(H)));
        assert_eq!(monitor.update(&committee, &connected(&addresses), H), UpgradeStatus::Required(required(H)));
        assert_eq!(dir.read(), Some(required(H)));
    }

    #[test]
    fn an_unreadable_record_holds_back_every_block_until_refuted() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        dir.write(&[1, 2, 3]);
        let monitor = monitor(&addresses, &dir);
        assert!(monitor.ensure_block_may_be_built(&committee, &HashSet::new(), 1).is_err());
        assert!(monitor.ensure_block_may_be_built(&committee, &connected(&addresses[..2]), 1).is_ok());
        assert_eq!(dir.read(), None);
    }

    #[test]
    fn a_scheduled_upgrade_is_logged_at_most_once_per_interval() {
        let (committee, addresses) = sample_committee();
        let dir = RecordDir::new();
        let monitor = monitor(&addresses, &dir);
        assert!(!monitor.should_warn());
        monitor.record_schedule(addresses[0], Some(&ahead_at(H)));
        monitor.record_schedule(addresses[1], Some(&ahead_at(H)));
        monitor.update(&committee, &connected(&addresses), 0);
        assert!(monitor.should_warn());
        assert!(!monitor.should_warn());
    }
}
