# snarkos-node-bft

[![Crates.io](https://img.shields.io/crates/v/snarkos-node-bft.svg?color=neon)](https://crates.io/crates/snarkos-node-bft)
[![Authors](https://img.shields.io/badge/authors-Aleo-orange.svg)](https://aleo.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE.md)

The `snarkos-node-bft` crate provides a node implementation for a BFT-based memory pool.

## Primary

The primary is the coordinator, responsible for advancing rounds and broadcasting the anchor.

#### Triggering Round Advancement

Each round runs until one of two conditions is met:
1. The coinbase target has been reached, or
2. The round has reached its timeout (currently set to 10 seconds)

#### Advancing Rounds

As described in the paper [Bullshark: The Partially Synchronous Version](https://arxiv.org/abs/2209.05633),
the BFT generally advances rounds when `n − f` vertices are delivered, however:
```
The problem in advancing rounds whenever n − f vertices are delivered is that parties
might not vote for the anchor even if the party that broadcast it is just slightly slower
than the fastest n − f parties. To deal with this, the BFT integrates timeouts into
the DAG construction. If the first n − f vertices a party p gets in an even-numbered round r 
do not include the anchor of round r, then p sets a timer and waits for the anchor
until the timer expires. Similarly, in an odd-numbered round, parties wait for either
f + 1 vertices that vote for the anchor, or 2f + 1 vertices that do not, or a timeout.
```
Note that in this quote `2f + 1` should really be `n - f`.

#### Batch Proposal

Batch proposals are driven by a dedicated **batch proposal task** that runs in a loop and is the only place that calls `propose_batch()`. This keeps proposal on a single execution path and avoids concurrent proposal attempts.

Each iteration of the inner loop waits for the first of these to fire before calling `propose_batch()`:

1. **Ready notification** (`is_ready_notify`) — When the primary advances to a new round (e.g. after a certificate is committed, or in the Narwhal case when storage increments the round), it signals readiness via `is_ready_notify`. The task wakes up and, if the node is synced, calls `propose_batch()`.
2. **Delay timeout** — If not sufficient time has elapsed, the task sets a timer for `MAX_BATCH_DELAY − elapsed`.
3. **Sync completion** If the node is currently syncing, it waits for the state to change to `Synced`. This lets the task wake up as soon as sync finishes without polling.

The primary tracks the latest proposed **(round, timestamp)** in `latest_proposed_batch`. This state is used to: avoid proposing the same round twice; rate-limit the primary's own proposals (via a dedicated check against the previous proposal timestamp); and decide whether to advance when a certificate is received. Peer proposal timestamps are validated separately so that the primary does not accept batches proposed too soon after a peer's previous proposal.

### Ledger Advancement

The BFT module also advances the ledger as new certificates are added to the DAG. There are two different ways the ledger can advance.

#### 1. Consensus Path (Normal Operation)

When the node is actively participating in consensus and is synced with the network:

1. **Certificate Collection**: The Primary receives batch certificates from validators and passes them to the BFT using `add_new_certificate()`, which then updates the DAG.
2. **Leader Election**: Leaders are elected in even rounds. When a certificate arrives for round `r`, the BFT checks if the leader certificate for round `r-1` can be committed.
3. **Availability Threshold**: The leader certificate is ready to commit when the availability threshold is reached—i.e., enough validators in round `r` have included the leader's certificate in their previous certificate IDs.
4. **Commit Chain**: `commit_leader_certificate()` is called, which:
   - Walks backwards through the DAG to find all uncommitted leader certificates that are linked to the current one
   - Builds a subDAG containing all certificates to be committed
   - Sends the subDAG to the Consensus module via `tx_consensus_subdag`
5. **Block Creation**: The Consensus module receives the subDAG and calls `try_advance_to_next_block()`, which:
   - Calls `ledger.begin_ledger_update()` to obtain a LedgerUpdate (blocking other writers until the handle is dropped)
   - Uses the handle to prepare a new block from the subDAG and its transmissions (`prepare_advance_to_next_quorum_block()`), validate it (`check_next_block()`), and advance the ledger (`advance_to_next_block()`)
   - Drops the handle so the ledger lock is released

#### 2. Sync Path (Catching Up)

When the node is behind and syncing blocks from peers, the `bft::Sync` module handles synchronization. The behavior differs based on how far behind the node is:

##### Within GC Range (Normal Sync)

When the node is within the garbage collection range of the network tip:

1. **Block Reception**: `bft::Sync` requests and receives blocks from peer nodes via `BlockSync`.
2. **Block Verification**: Blocks are verified using `check_block_subdag()` and added to a queue of `pending_blocks`.
3. **Certificate Insertion**: Each certificate from the block's subDAG is added to storage via `sync_certificate_with_block()` and sent to the BFT. This populates the DAG with the certificates needed for consensus.
4. **BFT-Driven Ledger Advancement**: The BFT module handles block creation through its normal consensus path -- when enough certificates are added to the DAG, the BFT commits leader certificates and creates blocks just as it does during normal operation.
5. **Pending Block Cleanup**: When the ledger advances because of leader commits (see 4), pending blocks are removed from the queue.

##### Outside GC Range (Fast Sync)
When the node is too far behind (outside the GC range):

1. **Block Reception**: `bft::Sync` requests and receives blocks from peer nodes via `BlockSync` (same as with normal sync).
2. **Block Verification**: Blocks are verified using `check_block_subdag()` and added to a queue of `pending_blocks` (same as with normal sync).
3. **No DAG Updates**: Certificates are **not** added to the BFT's DAG, since they are too old to be useful for consensus.
4. **Availability Threshold Check**: The Sync module checks whether each pending block's leader certificate has reached the availability threshold via `is_block_availability_threshold_reached()`. This uses certificates from subsequent pending blocks that reference the leader certificate.
5. **Ledger Advancement**: Once the availability threshold is confirmed, the Sync module acquires a LedgerUpdate via `ledger.begin_ledger_update()` and, for each confirmed block in sequence, calls `ledger_update.check_block_content(pending_block)` and `ledger_update.advance_to_next_block(&block)`. It also updates storage height and round. The single update handle ensures no concurrent advancement from the consensus path while sync is applying these blocks.
6. **Transition to Normal Sync**: Once the node catches up to within the GC range, `sync_storage_with_ledger_at_bootup()` is called to populate the BFT DAG with recent certificates, and the node switches back to normal BFT-driven sync.

### Startup Initialization

When a node starts, the sync module reconstructs the BFT DAG for the most recent rounds from the ledger's disk state. This is handled by `Sync::initialize()`, which calls `sync_storage_with_ledger_at_bootup()`:

1. **Determine the GC Height**: The sync module calculates the earliest block height that corresponds to rounds not yet garbage collected. Since at most one block is created every two rounds, this is computed as:
   ```
   gc_height = latest_block_height - (max_gc_rounds / 2)
   ```

2. **Load Blocks from Ledger**: All blocks from `gc_height` to the latest block are retrieved from the ledger (RocksDB).

3. **Sync Storage State**: The in-memory storage is synchronized with the latest block:
   - `sync_height_with_block()` updates the current height
   - `sync_round_with_block()` updates the current round
   - `garbage_collect_certificates()` removes any stale certificates

4. **Reconstruct Certificate Storage**: For each block in the range, if it has a quorum authority (subDAG):
   - The unconfirmed transactions are reconstructed from the block's transactions
   - Each certificate from the subDAG is inserted into storage via `sync_certificate_with_block()`
   - This populates the in-memory certificate maps and persists transmissions that are missing from disk

5. **Populate the BFT DAG**: All certificates from the loaded blocks are passed to the BFT module via `add_certificate_from_sync` and marked as committed using `commit_certificate_from_sync`, so the BFT won't try to re-commit them.

6. **Set Sync Height**: Finally, `BlockSync::set_sync_height()` is called to inform the block sync module of the current synchronized height.

After initialization completes:
- The BFT DAG contains all certificates from recent blocks (within GC range)
- The storage contains the corresponding transmissions
- The node is ready to participate in consensus or continue syncing from peers

## Workers

The workers are simple entry replicators that receive transactions from the network and append them to their memory pool.

In order to function properly, workers must be synced to the latest round, and capable of performing verification
on the entries they receive from other validators' workers.

## Test Cases

- Two validators, one with X workers, another with Y workers. Check that they are compatible.
- If a primary sees that f+1 other primaries have certified this round, it should skip to the next round if it has not been certified yet.
- Ensure taking a set number of transmissions from workers leaves the remaining transmissions in place for the next round.
- Send back a mismatching transmission for a transmission ID, ensure it catches it.
- Send back a mismatching certificate for a certificate ID, ensure it catches it.

## Open Questions

1. How does one guarantee the number of accepted transactions and solutions does not exceed the block limits?
   - We need to set limits on the number of transmissions for the workers, but also the primary.
