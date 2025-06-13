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

use crate::test_helpers::*;

use snarkos_node_sync_communication_service::test_helpers::DummyCommunicationService;

use snarkvm::prelude::*;

use indexmap::{IndexSet, indexset};
use rand::seq::IteratorRandom;

type CurrentNetwork = snarkvm::prelude::MainnetV0;

/// Tests that height and hash values are set correctly using many different maximum block heights.
#[test]
fn test_latest_block_height() {
    for height in generate_block_heights(100_001, 5000) {
        let sync = sample_sync_at_height(height);
        // Check that the latest block height is the maximum height.
        assert_eq!(sync.ledger.latest_block_height(), height);
    }
}

#[test]
fn test_get_block_height() {
    for height in generate_block_heights(100_001, 5000) {
        let sync = sample_sync_at_height(height);
        assert_eq!(sync.ledger.get_block_height(&(Field::<CurrentNetwork>::from_u32(0)).into()).unwrap(), 0);
        assert_eq!(sync.ledger.get_block_height(&(Field::<CurrentNetwork>::from_u32(height)).into()).unwrap(), height);
    }
}

#[test]
fn test_get_block_hash() {
    for height in generate_block_heights(100_001, 5000) {
        let sync = sample_sync_at_height(height);
        assert_eq!(sync.ledger.get_block_hash(0).unwrap(), (Field::<CurrentNetwork>::from_u32(0)).into());
        assert_eq!(sync.ledger.get_block_hash(height).unwrap(), (Field::<CurrentNetwork>::from_u32(height)).into());
    }
}

#[tokio::test]
async fn test_requests_insert_remove_insert() {
    let rng = &mut TestRng::default();
    let sync = sample_sync_at_height(0);

    // Add a peer.
    let peer_ip = sample_peer_ip(1);
    sync.update_peer_block_locators(peer_ip, sample_block_locators(0, 10)).await.unwrap();

    // Prepare the block requests.
    let comm = DummyCommunicationService;
    let (requests, sync_peers) = sync.prepare_block_requests(&comm).await;
    assert_eq!(requests.len(), 10);

    for (height, (hash, previous_hash, num_sync_ips)) in requests.clone() {
        // Construct the sync IPs.
        let sync_ips: IndexSet<_> = sync_peers.keys().choose_multiple(rng, num_sync_ips).into_iter().copied().collect();
        // Insert the block request.
        sync.insert_block_request(height, (hash, previous_hash, sync_ips.clone())).unwrap();
        // Check that the block requests were inserted.
        assert_eq!(sync.get_block_request(height), Some((hash, previous_hash, sync_ips)));
        assert!(sync.get_block_request_timestamp(height).is_some());
    }

    // Remove the peer.
    sync.remove_peer(&peer_ip);

    for (height, _) in requests {
        // Check that the block requests were removed.
        assert_eq!(sync.get_block_request(height), None);
        assert!(sync.get_block_request_timestamp(height).is_none());
    }

    // As there is no peer, it should not be possible to prepare block requests.
    let (requests, _) = sync.prepare_block_requests(&comm).await;
    assert_eq!(requests.len(), 0);

    // Add the peer again.
    sync.update_peer_block_locators(peer_ip, sample_block_locators(0, 10)).await.unwrap();

    // Prepare the block requests.
    let (requests, _) = sync.prepare_block_requests(&comm).await;
    assert_eq!(requests.len(), 10);

    for (height, (hash, previous_hash, num_sync_ips)) in requests {
        // Construct the sync IPs.
        let sync_ips: IndexSet<_> = sync_peers.keys().choose_multiple(rng, num_sync_ips).into_iter().copied().collect();
        // Insert the block request.
        sync.insert_block_request(height, (hash, previous_hash, sync_ips.clone())).unwrap();
        // Check that the block requests were inserted.
        assert_eq!(sync.get_block_request(height), Some((hash, previous_hash, sync_ips)));
        assert!(sync.get_block_request_timestamp(height).is_some());
    }
}
#[tokio::test]
async fn test_insert_block_requests() {
    let rng = &mut TestRng::default();
    let sync = sample_sync_at_height(0);

    // Add a peer.
    sync.update_peer_block_locators(sample_peer_ip(1), sample_block_locators(0, 10)).await.unwrap();

    // Prepare the block requests.
    let comm = DummyCommunicationService;
    let (requests, sync_peers) = sync.prepare_block_requests(&comm).await;
    assert_eq!(requests.len(), 10);

    for (height, (hash, previous_hash, num_sync_ips)) in requests.clone() {
        // Construct the sync IPs.
        let sync_ips: IndexSet<_> = sync_peers.keys().choose_multiple(rng, num_sync_ips).into_iter().copied().collect();
        // Insert the block request.
        sync.insert_block_request(height, (hash, previous_hash, sync_ips.clone())).unwrap();
        // Check that the block requests were inserted.
        assert_eq!(sync.get_block_request(height), Some((hash, previous_hash, sync_ips)));
        assert!(sync.get_block_request_timestamp(height).is_some());
    }

    for (height, (hash, previous_hash, num_sync_ips)) in requests.clone() {
        // Construct the sync IPs.
        let sync_ips: IndexSet<_> = sync_peers.keys().choose_multiple(rng, num_sync_ips).into_iter().copied().collect();
        // Check that the block requests are still inserted.
        assert_eq!(sync.get_block_request(height), Some((hash, previous_hash, sync_ips)));
        assert!(sync.get_block_request_timestamp(height).is_some());
    }

    for (height, (hash, previous_hash, num_sync_ips)) in requests {
        // Construct the sync IPs.
        let sync_ips: IndexSet<_> = sync_peers.keys().choose_multiple(rng, num_sync_ips).into_iter().copied().collect();
        // Ensure that the block requests cannot be inserted twice.
        sync.insert_block_request(height, (hash, previous_hash, sync_ips.clone())).unwrap_err();
        // Check that the block requests are still inserted.
        assert_eq!(sync.get_block_request(height), Some((hash, previous_hash, sync_ips)));
        assert!(sync.get_block_request_timestamp(height).is_some());
    }
}

/* TODO fix these
#[tokio::test]
async fn test_obsolete_block_requests() {
    let rng = &mut TestRng::default();
    let sync = sample_sync_at_height(0);

    let locator_height = rng.gen_range(0..50);

    // Add a peer.
    let locators = sample_block_locators(0, locator_height);
    sync.update_peer_block_locators(sample_peer_ip(1), locators.clone()).await.unwrap();

    // Construct block requests
    let comm = DummyCommunicationService::default();
    let (requests, sync_peers) = sync.prepare_block_requests(&comm).await;
    assert_eq!(requests.len(), locator_height as usize);

    // Add the block requests to the sync module.
    for (height, (hash, previous_hash, num_sync_ips)) in requests.clone() {
        // Construct the sync IPs.
        let sync_ips: IndexSet<_> = sync_peers.keys().choose_multiple(rng, num_sync_ips).into_iter().copied().collect();
        // Insert the block request.
        sync.insert_block_request(height, (hash, previous_hash, sync_ips.clone())).unwrap();
        // Check that the block requests were inserted.
        assert_eq!(sync.get_block_request(height), Some((hash, previous_hash, sync_ips)));
        assert!(sync.get_block_request_timestamp(height).is_some());
    }

    // Duplicate a new sync module with a different height to simulate block advancement.
    // This range needs to be inclusive, so that the range is never empty,
    // even with a locator height of 0.
    let ledger_height = rng.gen_range(0..=locator_height);
    let new_sync = duplicate_sync_at_new_height(&sync, ledger_height);

    // Check that the number of requests is the same.
    assert_eq!(new_sync.requests.read().len(), requests.len());

    // Remove timed out block requests.
    let c = DummyCommunicationService::default();
    new_sync.handle_block_request_timeouts(&c);

    // Check that the number of requests is reduced based on the ledger height.
    assert_eq!(new_sync.requests.read().len(), (locator_height - ledger_height) as usize);
}

#[tokio::test]
async fn test_timed_out_block_request() {
    let sync = sample_sync_at_height(0);
    let peer_ip = sample_peer_ip(1);
    let locators = sample_block_locators(0, 10);
    let block_hash = locators.get_hash(1);

    sync.update_peer_block_locators(peer_ip, locators.clone()).await.unwrap();

    let timestamp = Instant::now() - BLOCK_REQUEST_TIMEOUT - Duration::from_secs(1);

    // Add a timed-out request
    sync.requests.write().insert(1, OutstandingRequest {
        request: (block_hash, None, [peer_ip].into()),
        timestamp,
        response: None,
    });

    assert_eq!(sync.requests.read().len(), 1);
    assert_eq!(sync.locators.read().len(), 1);

    // Remove timed out block requests.
    let c = DummyCommunicationService::default();
    sync.handle_block_request_timeouts(&c);

    let ban_list = c.peers_to_ban.lock();
    assert_eq!(ban_list.len(), 1);
    assert_eq!(ban_list.iter().next(), Some(&peer_ip));

    assert!(sync.requests.read().is_empty());
    assert!(sync.locators.read().is_empty());
}

#[tokio::test]
async fn test_reissue_timed_out_block_request() {
    let sync = sample_sync_at_height(0);
    let peer_ip1 = sample_peer_ip(1);
    let peer_ip2 = sample_peer_ip(2);
    let peer_ip3 = sample_peer_ip(3);

    let locators = sample_block_locators(0, 10);
    let block_hash1 = locators.get_hash(1);
    let block_hash2 = locators.get_hash(2);

    sync.update_peer_block_locators(peer_ip1, locators.clone()).await.unwrap();
    sync.update_peer_block_locators(peer_ip2, locators.clone()).await.unwrap();
    sync.update_peer_block_locators(peer_ip3, locators.clone()).await.unwrap();

    assert_eq!(sync.locators.read().len(), 3);

    let timestamp = Instant::now() - BLOCK_REQUEST_TIMEOUT - Duration::from_secs(1);

    // Add a timed-out request
    sync.requests.write().insert(1, OutstandingRequest {
        request: (block_hash1, None, [peer_ip1].into()),
        timestamp,
        response: None,
    });

    // Add a timed-out request
    sync.requests.write().insert(2, OutstandingRequest {
        request: (block_hash2, None, [peer_ip2].into()),
        timestamp: Instant::now(),
        response: None,
    });

    assert_eq!(sync.requests.read().len(), 2);

    // Remove timed out block requests.
    let c = DummyCommunicationService::default();
    let re_requests = sync.handle_block_request_timeouts(&c);

    let ban_list = c.peers_to_ban.lock();
    assert_eq!(ban_list.len(), 1);
    assert_eq!(ban_list.iter().next(), Some(&peer_ip1));

    assert_eq!(sync.requests.read().len(), 1);
    assert_eq!(sync.locators.read().len(), 2);

    let (new_requests, new_sync_ips) = re_requests.unwrap();
    assert_eq!(new_requests.len(), 1);

    let (height, (hash, _, _)) = new_requests.first().unwrap();
    assert_eq!(*height, 1);
    assert_eq!(*hash, block_hash1);
    assert_eq!(new_sync_ips.len(), 2);

    // Make sure the removed peer is not in the sync_peer set.
    let mut iter = new_sync_ips.iter();
    assert_ne!(iter.next().unwrap().0, &peer_ip1);
    assert_ne!(iter.next().unwrap().0, &peer_ip1);
}*/

#[tokio::test]
async fn test_insert_block_requests_fails() {
    let sync = sample_sync_at_height(9);

    // Add a peer.
    sync.update_peer_block_locators(sample_peer_ip(1), sample_block_locators(0, 10)).await.unwrap();

    // Inserting a block height that is already in the ledger should fail.
    sync.insert_block_request(9, (None, None, indexset![sample_peer_ip(1)])).unwrap_err();
    // Inserting a block height that is not in the ledger should succeed.
    sync.insert_block_request(10, (None, None, indexset![sample_peer_ip(1)])).unwrap();
}

#[tokio::test]
async fn test_prepare_block_requests() {
    for num_peers in 0..111 {
        println!("Testing with {num_peers} peers");

        let sync = sample_sync_at_height(0);

        let mut peers = indexset![];

        for peer_id in 1..=num_peers {
            // Add a peer.
            sync.update_peer_block_locators(sample_peer_ip(peer_id), sample_block_locators(0, 10)).await.unwrap();
            // Add the peer to the set of peers.
            peers.insert(sample_peer_ip(peer_id));
        }

        // If all peers are ahead, then requests should be prepared.
        let comm = DummyCommunicationService;
        check_prepare_block_requests(&comm, sync, 10, peers).await;
    }
}

#[tokio::test]
async fn test_prepare_block_requests_with_leading_fork_at_11() {
    let sync = sample_sync_at_height(0);

    // Intuitively, peer 1's fork is above peer 2 and peer 3's height.
    // So from peer 2 and peer 3's perspective, they don't even realize that peer 1 is on a fork.
    // Thus, you can sync up to block 10 from any of the 3 peers.

    // When there are NUM_REDUNDANCY peers ahead, and 1 peer is on a leading fork at 11,
    // then the sync pool should request blocks 1..=10 from the NUM_REDUNDANCY peers.
    // This is safe because the leading fork is at 11, and the sync pool is at 0,
    // so all candidate peers are at least 10 blocks ahead of the sync pool.

    // Add a peer (fork).
    let peer_1 = sample_peer_ip(1);
    sync.update_peer_block_locators(peer_1, sample_block_locators_with_fork(0, 20, 11)).await.unwrap();

    // Add a peer.
    let peer_2 = sample_peer_ip(2);
    sync.update_peer_block_locators(peer_2, sample_block_locators(0, 10)).await.unwrap();

    // Add a peer.
    let peer_3 = sample_peer_ip(3);
    sync.update_peer_block_locators(peer_3, sample_block_locators(0, 10)).await.unwrap();

    // Prepare the block requests.
    let comm = DummyCommunicationService;
    let (requests, _) = sync.prepare_block_requests(&comm).await;
    assert_eq!(requests.len(), 10);

    // Check the requests.
    for (idx, (height, (hash, previous_hash, num_sync_ips))) in requests.into_iter().enumerate() {
        assert_eq!(height, 1 + idx as u32);
        assert_eq!(hash, Some((Field::<CurrentNetwork>::from_u32(height)).into()));
        assert_eq!(previous_hash, Some((Field::<CurrentNetwork>::from_u32(height - 1)).into()));
        assert_eq!(num_sync_ips, 1); // Only 1 needed since we have redundancy factor on this (recent locator) hash.
    }
}

/*#[tokio::test]
async fn test_prepare_block_requests_with_leading_fork_at_10() {
    let rng = &mut TestRng::default();
    let sync = sample_sync_at_height(0);

    // Intuitively, peer 1's fork is at peer 2 and peer 3's height.
    // So from peer 2 and peer 3's perspective, they recognize that peer 1 has forked.
    // Thus, you don't have NUM_REDUNDANCY peers to sync to block 10.
    //
    // Now, while you could in theory sync up to block 9 from any of the 3 peers,
    // we choose not to do this as either side is likely to disconnect from us,
    // and we would rather wait for enough redundant peers before syncing.

    // When there are NUM_REDUNDANCY peers ahead, and 1 peer is on a leading fork at 10,
    // then the sync pool should not request blocks as 1 peer conflicts with the other NUM_REDUNDANCY-1 peers.
    // We choose to sync with a cohort of peers that are *consistent* with each other,
    // and prioritize from descending heights (so the highest peer gets priority).

    // Add a peer (fork).
    let peer_1 = sample_peer_ip(1);
    sync.update_peer_block_locators(peer_1, sample_block_locators_with_fork(0, 20, 10)).await.unwrap();

    // Add a peer.
    let peer_2 = sample_peer_ip(2);
    sync.update_peer_block_locators(peer_2, sample_block_locators(0, 10)).await.unwrap();

    // Add a peer.
    let peer_3 = sample_peer_ip(3);
    sync.update_peer_block_locators(peer_3, sample_block_locators(0, 10)).await.unwrap();

    // Prepare the block requests.
    let comm = DummyCommunicationService::default();
    let (requests, _) = sync.prepare_block_requests(&comm).await;
    assert_eq!(requests.len(), 0);

    // When there are NUM_REDUNDANCY+1 peers ahead, and 1 is on a fork, then there should be block requests.

    // Add a peer.
    let peer_4 = sample_peer_ip(4);
    sync.update_peer_block_locators(peer_4, sample_block_locators(0, 10)).await.unwrap();

    // Prepare the block requests.
    let (requests, sync_peers) = sync.prepare_block_requests(&comm).await;
    assert_eq!(requests.len(), 10);

    // Check the requests.
    for (idx, (height, (hash, previous_hash, num_sync_ips))) in requests.into_iter().enumerate() {
        // Construct the sync IPs.
        let sync_ips: IndexSet<_> = sync_peers.keys().choose_multiple(rng, num_sync_ips).into_iter().copied().collect();
        assert_eq!(height, 1 + idx as u32);
        assert_eq!(hash, Some((Field::<CurrentNetwork>::from_u32(height)).into()));
        assert_eq!(previous_hash, Some((Field::<CurrentNetwork>::from_u32(height - 1)).into()));
        assert_eq!(sync_ips.len(), 1); // Only 1 needed since we have redundancy factor on this (recent locator) hash.
        assert_ne!(sync_ips[0], peer_1); // It should never be the forked peer.
    }
}

#[tokio::test]
async fn test_prepare_block_requests_with_trailing_fork_at_9() {
    let rng = &mut TestRng::default();
    let sync = sample_sync_at_height(0);

    // Peer 1 and 2 diverge from peer 3 at block 10. We only sync when there are NUM_REDUNDANCY peers
    // who are *consistent* with each other. So if you add a 4th peer that is consistent with peer 1 and 2,
    // then you should be able to sync up to block 10, thereby biasing away from peer 3.

    // Add a peer (fork).
    let peer_1 = sample_peer_ip(1);
    sync.update_peer_block_locators(peer_1, sample_block_locators(0, 10)).await.unwrap();

    // Add a peer.
    let peer_2 = sample_peer_ip(2);
    sync.update_peer_block_locators(peer_2, sample_block_locators(0, 10)).await.unwrap();

    // Add a peer.
    let peer_3 = sample_peer_ip(3);
    sync.update_peer_block_locators(peer_3, sample_block_locators_with_fork(0, 20, 10)).await.unwrap();

    // Prepare the block requests.
    let comm = DummyCommunicationService::default();
    let (requests, _) = sync.prepare_block_requests(&comm).await;
    assert_eq!(requests.len(), 0);

    // When there are NUM_REDUNDANCY+1 peers ahead, and peer 3 is on a fork, then there should be block requests.

    // Add a peer.
    let peer_4 = sample_peer_ip(4);
    sync.update_peer_block_locators(peer_4, sample_block_locators(0, 10)).await.unwrap();

    // Prepare the block requests.
    let (requests, sync_peers) = sync.prepare_block_requests(&comm).await;
    assert_eq!(requests.len(), 10);

    // Check the requests.
    for (idx, (height, (hash, previous_hash, num_sync_ips))) in requests.into_iter().enumerate() {
        // Construct the sync IPs.
        let sync_ips: IndexSet<_> = sync_peers.keys().choose_multiple(rng, num_sync_ips).into_iter().copied().collect();
        assert_eq!(height, 1 + idx as u32);
        assert_eq!(hash, Some((Field::<CurrentNetwork>::from_u32(height)).into()));
        assert_eq!(previous_hash, Some((Field::<CurrentNetwork>::from_u32(height - 1)).into()));
        assert_eq!(sync_ips.len(), 1); // Only 1 needed since we have redundancy factor on this (recent locator) hash.
        assert_ne!(sync_ips[0], peer_3); // It should never be the forked peer.
    }
}*/

#[tokio::test]
async fn test_update_peer_locators() {
    let sync = sample_sync_at_height(0);

    // Test 2 peers.
    let peer1_ip = sample_peer_ip(1);
    for peer1_height in 0..500u32 {
        sync.update_peer_block_locators(
            peer1_ip,
            sample_block_locators(peer1_height.saturating_sub(100), peer1_height),
        )
        .await
        .unwrap();
        assert_eq!(sync.get_peer_height(&peer1_ip), Some(peer1_height));

        let peer2_ip = sample_peer_ip(2);
        for peer2_height in 0..500u32 {
            println!("Testing peer 1 height at {peer1_height} and peer 2 height at {peer2_height}");

            sync.update_peer_block_locators(
                peer2_ip,
                sample_block_locators(peer2_height.saturating_sub(0), peer2_height),
            )
            .await
            .unwrap();
            assert_eq!(sync.get_peer_height(&peer2_ip), Some(peer2_height));
        }
    }
}

#[tokio::test]
async fn test_remove_peer() {
    let sync = sample_sync_at_height(0);

    let peer_ip = sample_peer_ip(1);
    sync.update_peer_block_locators(peer_ip, sample_block_locators(0, 100)).await.unwrap();
    assert_eq!(sync.get_peer_height(&peer_ip), Some(100));

    sync.remove_peer(&peer_ip);
    assert_eq!(sync.get_peer_height(&peer_ip), None);

    sync.update_peer_block_locators(peer_ip, sample_block_locators(0, 200)).await.unwrap();
    assert_eq!(sync.get_peer_height(&peer_ip), Some(200));

    sync.remove_peer(&peer_ip);
    assert_eq!(sync.get_peer_height(&peer_ip), None);
}

#[tokio::test]
async fn test_locators_insert_remove_insert() {
    let sync = sample_sync_at_height(0);

    let peer_ip = sample_peer_ip(1);
    sync.update_peer_block_locators(peer_ip, sample_block_locators(0, 100)).await.unwrap();
    assert_eq!(sync.get_peer_height(&peer_ip), Some(100));

    sync.remove_peer(&peer_ip);
    assert_eq!(sync.get_peer_height(&peer_ip), None);

    sync.update_peer_block_locators(peer_ip, sample_block_locators(0, 200)).await.unwrap();
    assert_eq!(sync.get_peer_height(&peer_ip), Some(200));
}
