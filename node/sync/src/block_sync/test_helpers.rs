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

use super::*;

use snarkos_node_bft_ledger_service::MockLedgerService;
use snarkos_node_sync_communication_service::CommunicationService;
use snarkvm::{
    ledger::committee::Committee,
    prelude::{Field, TestRng},
};

use indexmap::IndexSet;
#[cfg(feature = "locktick")]
use locktick::parking_lot::RwLock;
#[cfg(not(feature = "locktick"))]
use parking_lot::RwLock;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::Notify;

type CurrentNetwork = snarkvm::prelude::MainnetV0;

pub use crate::locators::test_helpers::*;

/// Returns the peer IP for the sync pool.
pub fn sample_peer_ip(id: u16) -> SocketAddr {
    assert_ne!(id, 0, "The peer ID must not be 0 (reserved for local IP in testing)");
    SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), id)
}

/// Returns a sample committee.
pub fn sample_committee() -> Committee<CurrentNetwork> {
    let rng = &mut TestRng::default();
    snarkvm::ledger::committee::test_helpers::sample_committee(rng)
}

/// Returns the ledger service, initialized to the given height.
pub fn sample_ledger_service(height: u32) -> MockLedgerService<CurrentNetwork> {
    MockLedgerService::new_at_height(sample_committee(), height)
}

/// Returns the sync pool, with the ledger initialized to the given height.
pub fn sample_sync_at_height(height: u32) -> BlockSync<CurrentNetwork> {
    BlockSync::<CurrentNetwork>::new(Arc::new(sample_ledger_service(height)))
}

/// Returns a vector of randomly sampled block heights in [0, max_height].
///
/// The maximum value will always be included in the result.
pub fn generate_block_heights(max_height: u32, num_values: usize) -> Vec<u32> {
    assert!(num_values > 0, "Cannot generate an empty vector");
    assert!((max_height as usize) >= num_values);

    let mut rng = TestRng::default();

    let mut heights: Vec<u32> = (0..(max_height - 1)).choose_multiple(&mut rng, num_values);

    heights.push(max_height);

    heights
}

/// Returns a duplicate (deep copy) of the sync pool with a different ledger height.
pub fn duplicate_sync_at_new_height(sync: &BlockSync<CurrentNetwork>, height: u32) -> BlockSync<CurrentNetwork> {
    BlockSync::<CurrentNetwork> {
        notify: Notify::new(),
        metrics: Default::default(),
        block_heights: sync.block_heights.clone(),
        ledger: Arc::new(sample_ledger_service(height)),
        locators: RwLock::new(sync.locators.read().clone()),
        common_ancestors: RwLock::new(sync.common_ancestors.read().clone()),
        requests: RwLock::new(sync.requests.read().clone()),
        sync_state: RwLock::new(sync.sync_state.read().clone()),
        advance_with_sync_blocks_lock: Default::default(),
    }
}

/// Checks that the sync pool (starting at genesis) returns the correct requests.
pub async fn check_prepare_block_requests<C: CommunicationService>(
    communication: &C,
    sync: BlockSync<CurrentNetwork>,
    min_common_ancestor: u32,
    peers: IndexSet<SocketAddr>,
) {
    let rng = &mut TestRng::default();

    // Check test assumptions are met.
    assert_eq!(sync.ledger.latest_block_height(), 0, "This test assumes the sync pool is at genesis");
    /*
    // Determine the number of peers within range of this sync pool.
    // let num_peers_within_recent_range_of_ledger = {
        // If no peers are within range, then set to 0.
        if min_common_ancestor >= NUM_RECENT_BLOCKS as u32 {
            0
        }
        // Otherwise, manually check the number of peers within range.
        else {
            peers.iter().filter(|peer_ip| sync.get_peer_height(peer_ip).unwrap() < NUM_RECENT_BLOCKS as u32).count()
        }
    };*/

    // Prepare the block requests.
    let (requests, sync_peers) = sync.prepare_block_requests(communication).await;

    // If there are no peers, then there should be no requests.
    if peers.is_empty() {
        assert!(requests.is_empty());
        return;
    }

    // Otherwise, there should be requests.
    let expected_num_requests = core::cmp::min(min_common_ancestor as usize, MAX_BLOCK_REQUESTS);
    assert_eq!(requests.len(), expected_num_requests);

    for (idx, (height, (hash, previous_hash, num_sync_ips))) in requests.into_iter().enumerate() {
        // Construct the sync IPs.
        let sync_ips: IndexSet<_> = sync_peers.keys().choose_multiple(rng, num_sync_ips).into_iter().copied().collect();
        assert_eq!(height, 1 + idx as u32);
        assert_eq!(hash, Some((Field::<CurrentNetwork>::from_u32(height)).into()));
        assert_eq!(previous_hash, Some((Field::<CurrentNetwork>::from_u32(height - 1)).into()));

        /*
        if num_peers_within_recent_range_of_ledger >= REDUNDANCY_FACTOR {
            assert_eq!(sync_ips.len(), 1);
        } else {
            assert_eq!(sync_ips.len(), num_peers_within_recent_range_of_ledger);
            assert_eq!(sync_ips, peers);
        }*/

        //assert_eq!(sync_ips.len(), num_peers_within_recent_range_of_ledger);
        assert_eq!(sync_ips, peers);
    }
}
