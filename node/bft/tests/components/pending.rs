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

use crate::common::{CurrentNetwork, primary::new_test_committee, utils::sample_ledger};
use snarkos_node_bft::helpers::max_redundant_requests;
use snarkvm::prelude::{Network, TestRng};

#[test]
fn test_max_redundant_requests() {
    let num_nodes: u16 = CurrentNetwork::MAX_CERTIFICATES.first().unwrap().1;

    // Initialize the RNG.
    let mut rng = TestRng::default();
    // Initialize the accounts and the committee.
    let (accounts, committee) = new_test_committee(num_nodes, &mut rng);
    // Sample a ledger.
    let ledger = sample_ledger(&accounts, &committee, &mut rng);
    // Ensure the maximum number of redundant requests is correct and consistent across iterations.
    assert_eq!(max_redundant_requests(ledger, 0).unwrap(), 6, "Update me if the formula changes");
}
