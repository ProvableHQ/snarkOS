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

use snarkvm::{
    ledger::{
        committee::Committee,
        narwhal::{BatchCertificate, BatchHeader, Subdag},
    },
    prelude::{Address, Field, Network, cfg_chunks, cfg_iter},
};

use indexmap::{IndexMap, IndexSet};
#[cfg(feature = "locktick")]
use locktick::parking_lot::RwLock;
#[cfg(not(feature = "locktick"))]
use parking_lot::RwLock;
#[cfg(not(feature = "serial"))]
use rayon::prelude::*;
use std::{collections::BTreeMap, sync::Arc};

// TODO: Consider other metrics to track:
//  - Response time
//  - Sync rate
//  - Latest height of each validator
//  - Percentage of proposals that are converted into certificates
//  - Fullness of proposals
//  - Connectivity (how many other validators are they connected to)
//  - Various stake weight considerations
//  - The latest seen IP address of each validator (useful for debugging purposes)

/// The participation scores for each validator.
///     Certificate Score: The % of rounds the validator has a valid certificate
///     Signature Score: The % of certificates the validator has a valid signature for
///     Combined Score: The weighted score using the certificate and signature scores
type ParticipationScores = (f64, f64, f64);

/// Tracker for the participation metrics of validators.
#[derive(Clone, Debug)]
pub struct Telemetry<N: Network> {
    /// The certificates seen for each round
    /// A mapping of `round` to set of certificate IDs.
    /// Note that this map is sorted to allow grouped iteration over rounds.
    tracked_certificates: Arc<RwLock<BTreeMap<u64, IndexSet<Field<N>>>>>,

    /// The total number of signatures seen for a validator, including for their own certificates.
    /// A mapping of `address` to a mapping of `round` to `count`.
    validator_signatures: Arc<RwLock<IndexMap<Address<N>, IndexMap<u64, u32>>>>,

    /// The total number of certificates seen for a validator.
    /// A mapping of `address` to a list of rounds.
    validator_certificates: Arc<RwLock<IndexMap<Address<N>, IndexSet<u64>>>>,

    /// The certificate, signature, and participation scores for each validator.
    participation_scores: Arc<RwLock<IndexMap<Address<N>, ParticipationScores>>>,
}

impl<N: Network> Default for Telemetry<N> {
    /// Initializes a new instance of telemetry.
    fn default() -> Self {
        Self::new()
    }
}

impl<N: Network> Telemetry<N> {
    /// Initializes a new instance of telemetry.
    pub fn new() -> Self {
        Self {
            tracked_certificates: Default::default(),
            validator_signatures: Default::default(),
            validator_certificates: Default::default(),
            participation_scores: Default::default(),
        }
    }

    // TODO (raychu86): Consider using committee lookback here.
    /// Fetch the participation scores for each validator in the committee set.
    pub fn get_participation_scores(&self, committee: &Committee<N>) -> IndexMap<Address<N>, f64> {
        // Fetch the participation scores.
        let participation_scores = self.participation_scores.read();
        // Calculate the weighted score for each validator.
        committee
            .members()
            .iter()
            .map(|(address, _)| {
                let score =
                    participation_scores.get(address).map(|(_, _, combined_score)| *combined_score).unwrap_or(0.0);
                (*address, score)
            })
            .collect()
    }

    /// Insert a subdag to the telemetry tracker.
    /// Note: This currently assumes the subdag is fully formed and included in the block.
    pub fn insert_subdag(&self, subdag: &Subdag<N>) {
        // Garbage collect the old certificates.
        let next_gc_round = subdag.anchor_round().saturating_sub(BatchHeader::<N>::MAX_GC_ROUNDS as u64);
        self.garbage_collect_certificates(next_gc_round);

        // Insert the subdag certificates.
        cfg_iter!(subdag).for_each(|(_round, certificates)| {
            cfg_iter!(certificates).for_each(|certificate| {
                // TODO (raychu86): Can be greatly optimized by doing a one-shot update instead of individual certificates.
                self.insert_certificate(certificate);
            })
        });

        // Calculate the participation scores.
        self.update_participation_scores();
    }

    /// Insert a certificate to the telemetry tracker.
    pub fn insert_certificate(&self, certificate: &BatchCertificate<N>) {
        // Acquire the lock for `tracked_certificates`.
        let mut tracked_certificates = self.tracked_certificates.write();

        // Retrieve the certificate round, author, and ID.
        let certificate_round = certificate.round();
        let certificate_author = certificate.author();
        let certificate_id = certificate.id();

        // If the certificate already exists in the tracker, then return early.
        if tracked_certificates.get(&certificate_round).is_some_and(|certs| certs.contains(&certificate_id)) {
            return;
        }

        // Insert the certificate ID.
        tracked_certificates.entry(certificate_round).or_default().insert(certificate_id);

        // Acquire the lock for `validator_signatures`
        let mut validator_signatures = self.validator_signatures.write();

        // Insert the certificate author and signers.
        for address in
            [certificate_author].into_iter().chain(certificate.signatures().map(|signature| signature.to_address()))
        {
            validator_signatures
                .entry(address)
                .or_default()
                .entry(certificate_round)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }

        // Acquire the lock for `validator_certificates`.
        let mut validator_certificates = self.validator_certificates.write();

        // Insert the certificate
        validator_certificates.entry(certificate_author).or_default().insert(certificate_round);
    }

    /// Calculate and update the participation scores for each validator.
    pub fn update_participation_scores(&self) {
        // Calculate the combined score with custom weights:
        // - 90% certificate participation score
        // - 10% signature participation score
        fn weighted_score(certificate_score: f64, signature_score: f64) -> f64 {
            let score = (0.9 * certificate_score) + (0.1 * signature_score);

            // Truncate to the last 2 decimal places.
            (score * 100.0).round() / 100.0
        }

        // Fetch the certificates and signatures.
        let tracked_certificates = self.tracked_certificates.read();
        let validator_signatures = self.validator_signatures.read();
        let validator_certificates = self.validator_certificates.read();

        // Fetch the total number of certificates.
        let total_certificates = validator_certificates.values().map(|rounds| rounds.len()).sum::<usize>();

        // Calculate the signature participation scores for each validator.
        let signature_participation_scores: IndexMap<_, _> = cfg_iter!(&*validator_signatures)
            .map(|(address, signatures)| {
                let total_signatures = signatures.values().sum::<u32>() as f64;
                let score = total_signatures / total_certificates as f64 * 100.0;
                (*address, score as u16)
            })
            .collect();

        // Calculate the certificate participation scores for each validator.
        // This score is based on how many certificates the validator has included in every two rounds.
        let tracked_rounds: Vec<_> = tracked_certificates.keys().skip_while(|r| *r % 2 == 0).copied().collect();
        let certificate_participation_scores: IndexMap<_, _> = cfg_iter!(&*validator_certificates)
            .map(|(address, certificate_rounds)| {
                // Count the number of round pairs that are included in the certificate rounds.
                let num_included_round_pairs = cfg_chunks!(&tracked_rounds, 2)
                    .filter(|chunk| chunk.iter().any(|r| certificate_rounds.contains(r)))
                    .count();
                // Calculate the number of round pairs.
                let num_round_pairs = (tracked_rounds.len().saturating_add(1)).saturating_div(2);
                // Calculate the score based on the number of certificate rounds the validator is a part of.
                let score = num_included_round_pairs as f64 / num_round_pairs.max(1) as f64 * 100.0;
                (*address, score as u16)
            })
            .collect();

        // Calculate the final participation scores for each validator.
        let validator_addresses: IndexSet<_> =
            signature_participation_scores.keys().chain(certificate_participation_scores.keys()).copied().collect();
        let mut new_participation_scores = IndexMap::new();
        for address in validator_addresses {
            let signature_score = *signature_participation_scores.get(&address).unwrap_or(&0) as f64;
            let certificate_score = *certificate_participation_scores.get(&address).unwrap_or(&0) as f64;
            let combined_score = weighted_score(certificate_score, signature_score);
            new_participation_scores.insert(address, (certificate_score, signature_score, combined_score));
        }

        // Update the participation scores.
        *self.participation_scores.write() = new_participation_scores;
    }

    /// Remove the certificates from the telemetry tracker that are no longer relevant based on gc.
    pub fn garbage_collect_certificates(&self, gc_round: u64) {
        // Acquire the locks.
        let mut tracked_certificates = self.tracked_certificates.write();
        let mut validator_signatures = self.validator_signatures.write();
        let mut validator_certificates = self.validator_certificates.write();

        // Remove certificates that are not longer relevant
        tracked_certificates.retain(|&round, _| round > gc_round);

        // Remove signatures that are no longer relevant.
        validator_signatures.retain(|_, rounds| {
            rounds.retain(|&round, _| round > gc_round);
            // Remove the address if there are no more tracked signatures.
            !rounds.is_empty()
        });

        // Remove certificates that are no longer relevant.
        validator_certificates.retain(|_, rounds| {
            rounds.retain(|&round| round > gc_round);
            // Remove the address if there are no more tracked certificates.
            !rounds.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use snarkvm::{
        ledger::{
            committee::test_helpers::sample_committee_for_round_and_members,
            narwhal::batch_certificate::test_helpers::sample_batch_certificate_for_round,
        },
        prelude::MainnetV0,
        utilities::TestRng,
    };

    use rand::Rng;

    type CurrentNetwork = MainnetV0;

    #[test]
    fn test_insert_certificates() {
        let rng = &mut TestRng::default();

        // Initialize the telemetry tracker.
        let telemetry = Telemetry::<CurrentNetwork>::new();

        // Set the current round.
        let current_round = 2;

        // Sample the certificates.
        let mut certificates = IndexSet::new();
        for _ in 0..10 {
            certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        }

        // Insert the certificates.
        assert!(telemetry.tracked_certificates.read().is_empty());
        for certificate in &certificates {
            telemetry.insert_certificate(certificate);
        }
        assert_eq!(telemetry.tracked_certificates.read().get(&current_round).unwrap().len(), certificates.len());
    }

    #[test]
    fn test_participation_scores() {
        let rng = &mut TestRng::default();

        // Initialize the telemetry tracker.
        let telemetry = Telemetry::<CurrentNetwork>::new();

        // Set the current round.
        let current_round = 2;

        // Sample the certificates.
        let mut certificates = IndexSet::new();
        certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        certificates.insert(sample_batch_certificate_for_round(current_round, rng));

        // Initialize the committee.
        let committee = sample_committee_for_round_and_members(
            current_round,
            vec![
                certificates[0].author(),
                certificates[1].author(),
                certificates[2].author(),
                certificates[3].author(),
            ],
            rng,
        );

        // Insert the certificates.
        assert!(telemetry.tracked_certificates.read().is_empty());
        for certificate in &certificates {
            telemetry.insert_certificate(certificate);
        }

        // Fetch the participation scores.
        let participation_scores = telemetry.get_participation_scores(&committee);
        assert_eq!(participation_scores.len(), committee.members().len());
        for (address, _) in committee.members() {
            assert_eq!(*participation_scores.get(address).unwrap(), 0.0);
        }

        // Calculate the participation scores.
        telemetry.update_participation_scores();

        // Ensure that the participation scores are updated.
        let participation_scores = telemetry.get_participation_scores(&committee);
        for (address, _) in committee.members() {
            assert!(*participation_scores.get(address).unwrap() > 0.0);
        }

        println!("{participation_scores:?}");
    }

    #[test]
    fn test_garbage_collection() {
        let rng = &mut TestRng::default();

        // Initialize the telemetry tracker.
        let telemetry = Telemetry::<CurrentNetwork>::new();

        // Set the current round.
        let current_round = 2;
        let next_round = current_round + 1;

        // Sample the certificates for round `current_round`
        let mut certificates = IndexSet::new();
        let num_initial_certificates = rng.gen_range(1..10);
        for _ in 0..num_initial_certificates {
            certificates.insert(sample_batch_certificate_for_round(current_round, rng));
        }

        // Sample the certificates for round `next_round`
        let num_new_certificates = rng.gen_range(1..10);
        for _ in 0..num_new_certificates {
            certificates.insert(sample_batch_certificate_for_round(next_round, rng));
        }

        // Insert the certificates.
        for certificate in &certificates {
            telemetry.insert_certificate(certificate);
        }
        assert_eq!(telemetry.tracked_certificates.read().get(&current_round).unwrap().len(), num_initial_certificates);
        assert_eq!(telemetry.tracked_certificates.read().get(&next_round).unwrap().len(), num_new_certificates);

        // Garbage collect the certificates
        telemetry.garbage_collect_certificates(current_round);
        assert!(telemetry.tracked_certificates.read().get(&current_round).is_none());
        assert_eq!(telemetry.tracked_certificates.read().get(&next_round).unwrap().len(), num_new_certificates);
    }
}
