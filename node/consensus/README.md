# snarkos-node-consensus

[![Crates.io](https://img.shields.io/crates/v/snarkos-node-consensus.svg?color=neon)](https://crates.io/crates/snarkos-node-consensus)
[![Authors](https://img.shields.io/badge/authors-Aleo-orange.svg)](https://aleo.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE.md)

This crate builds on top of `snarkos-node-bft`, which implements AleoBFT.
It manages a rate-limiting mempool for incoming transmissions and constructs blocks from batches that have been confirmed by the BFT layer.

## Mempool Architecture

The mempool uses a two-tier architecture to prevent overloading the BFT layer's workers:

### Tier 1: Consensus Inbound Queues

When transmissions (solutions or transactions) are received, they are first added to the **Consensus inbound queues**:

- **Solutions Queue**: An LRU cache with capacity for up to 1024 solutions.
- **Transactions Queue**: Separate queues for deployments (1024 capacity) and executions (1024 capacity), each with priority sub-queues that order transactions by their priority fee.

At this stage, transmissions are only checked for duplicates (against recently-seen caches and the ledger) but are not fully verified.

### Tier 2: Worker Ready Queues

The BFT layer maintains multiple **workers**, each with its own **ready queue**. Transmissions in the ready queue have been verified and are ready for inclusion in a batch proposal.

Consensus periodically (every `MAX_BATCH_DELAY_IN_MS`) attempts to move transmissions from the inbound queues to the workers:

1. **Capacity Check**: Transmissions are only forwarded if the workers have capacity (less than `MAX_TRANSMISSIONS_TOLERANCE` unconfirmed transmissions).
2. **Worker Assignment**: Each transmission is assigned to a specific worker based on a hash of its ID.
3. **Verification**: The worker verifies the transmission before adding it to the ready queue:
   - **Solutions**: Checked via `check_solution_basic()` to ensure they are well-formed and unique.
   - **Transactions**: Deserialized (if in buffer form) and validated via `check_transaction_basic()`.

### Transmission Flow

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CONSENSUS LAYER                                │
│  ┌─────────────────────┐         ┌──────────────────────────────────────┐   │
│  │   Solutions Queue   │         │         Transactions Queue           │   │
│  │    (LRU, 1024)      │         │  ┌────────────┐  ┌────────────────┐  │   │
│  └──────────┬──────────┘         │  │Deployments │  │   Executions   │  │   │
│             │                    │  │(LRU, 1024) │  │  (LRU, 1024)   │  │   │
│             │                    │  │ + priority │  │  + priority    │  │   │
│             │                    │  └─────┬──────┘  └───────┬────────┘  │   │
│             │                    └────────┼─────────────────┼───────────┘   │
│             │                             │                 │               │
│             └──────────────┬──────────────┴─────────────────┘               │
│                            │                                                │
│                   process_unconfirmed_*()                                   │
│                  (periodic, capacity-based)                                 │
│                            │                                                │
└────────────────────────────┼────────────────────────────────────────────────┘
                             │
                             ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                                BFT LAYER                                    │
│                                                                             │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐   │
│  │  Worker 0   │    │  Worker 1   │    │  Worker 2   │    │  Worker 3   │   │
│  │ Ready Queue │    │ Ready Queue │    │ Ready Queue │    │ Ready Queue │   │
│  └──────┬──────┘    └──────┬──────┘    └──────┬──────┘    └──────┬──────┘   │
│         │                  │                  │                  │          │
│         └──────────────────┴────────┬─────────┴──────────────────┘          │
│                                     │                                       │
│                           propose_batch()                                   │
│                                     │                                       │
│                                     ▼                                       │
│                          ┌───────────────────┐                              │
│                          │  Batch Proposal   │                              │
│                          └─────────┬─────────┘                              │
│                                    │                                        │
│                           (certification)                                   │
│                                    │                                        │
│                                    ▼                                        │
│                          ┌───────────────────┐                              │
│                          │ Batch Certificate │                              │
│                          └─────────┬─────────┘                              │
│                                    │                                        │
│                         insert_certificate()                                │
│                                    │                                        │
│                                    ▼                                        │
│                          ┌───────────────────┐                              │
│                          │ Persistent Storage│ ◄── Transmissions stored     │
│                          │    (RocksDB)      │     to disk here             │
│                          └───────────────────┘                              │
└─────────────────────────────────────────────────────────────────────────────┘
```

### When Are Transmissions Stored on Disk?

Transmissions are **not** persisted to disk when first received. They are stored on disk only when a **batch certificate** is inserted into storage:

1. A primary proposes a batch containing transmissions from worker ready queues.
2. The batch receives sufficient signatures from validators, forming a **batch certificate**.
3. When `Storage::insert_certificate()` is called, the certificate's transmissions are persisted via `StorageService::insert_transmissions()`.
4. The persistent storage (RocksDB) maintains a map from transmission ID to `(transmission, certificate_ids)`.

This design ensures that only transmissions included in certified batches consume disk space, while unconfirmed transmissions remain in memory.

### Aborted Transmissions

When a transmission (solution or transaction) is included in a certificate but later marked as **aborted** in a block, it is handled differently:

- **Non-aborted transmissions**: The full transmission data is stored in the `transmissions` map as `TransmissionID -> (Transmission, CertificateIDs)`.
- **Aborted transmissions**: Only the transmission **ID** is stored in a separate `aborted_transmission_ids` map as `TransmissionID -> CertificateIDs`. The actual transmission payload is **not** persisted.

This distinction saves disk space by avoiding storage of transaction data that was ultimately aborted, while still tracking which certificates referenced them.

### When Are Certificates Stored on Disk?

Certificates are stored on disk, but **not directly by themselves**. Instead, they are included as part of the block data structure that is persisted once a block is finalized and written to disk. This means:

- **When a certificate is created and inserted via `insert_certificate()`,** the certificate object (and its consensus state) remain **in memory**, and only only the **transmissions** associated with the certificate are directly persisted to disk. 
- **When a block is finalized and persisted to disk (e.g., via `insert_block()` or similar),** the certificate(s) included in that block are also persisted to disk as part of the block data. Thus, certificates are indirectly stored on disk by being embedded within blocks.

This design ensures efficient operation and allows nodes to reconstruct certificates from blocks even after restarts, while avoiding redundant or unnecessary certificate storage outside of finalized blocks.


### Gossip and Peer Synchronization

Workers also receive transmissions from peers via:

- **Worker Pings**: Periodic broadcasts of transmission IDs to other validators.
- **Transmission Requests**: When a worker sees a transmission ID it doesn't have, it requests the full transmission from the peer.

Transmissions received from peers via `process_transmission_from_peer()` are added directly to the worker's ready queue after basic validation checks.