# snarkos-node-sync

[![Crates.io](https://img.shields.io/crates/v/snarkos-node-sync.svg?color=neon)](https://crates.io/crates/snarkos-node-sync)
[![Authors](https://img.shields.io/badge/authors-Aleo-orange.svg)](https://aleo.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE.md)

The `snarkos-node-sync` crate provides a synchronization module for nodes.

## Background
snarkos nodes have two ways to synchronize with and follow the blockchain: Validators that are up-to-date generate new blocks from the DAG of certificates, while clients and validators that are "behind", directly sync blocks from other nodes.

The idea behind block sync is that it can be more efficient because, instead of sending many certificate, whe send one block contianing certificates. It also means that validating nodes do not need to track independent certificates for historical blocks, reducing storage overhead.

This crates provides the logic to fetch blocks from other peers, which is used by `snarkos-node-client` and `snarkos-node-bft`.
Block synchronization operates on ranges of blocks, to reduce the number of individual network messages.

Block synchronization in AleoBFT is tricky, however, because the committe doesn't sign or vote on blocks.
This means, faulty nodes can send invalid blocks where the DAG is missing entries, the block's hash does not match its certificates, or the corresponding anchor certificate was never accepted by the majority network.
So, in addition to fetching blocks from peers, this crate also handles conflicting blocks (i.e., forks).

## Implementation Overview
The crate provide `BlockSync` struct that implements the bulk of its logic.

To manage confliciting blocks, `BlockSync` can track conflicting blocks until one is confirmated by the network.
A confirmation in AleoBFT is achieved when the round after a potential's block's leader certificate, certificates representing a supermajority of the current committe point to it.

A block's approval is usually contained within the subsequent block, but it is possible that an approval does happen in some future round, when there are network errors.
More concretely, when a block is approved, all of its predecessors are approved as well. This means, that an unapproved block can
Thus, this `BlockSync` operated on pending *chains* of blocks.

Blocks are processed in three steps.

### 1. Block Request Generation
Nodes call `BlockSync::prepare_block_requests`, which then invokes `BlockSync::construct_request`.
Each block request operates over a range of blocks and have a designated peer to sync from.
If the peer is unresponsive, `BlockSync` deletes the request, disconnects from the peer, and generates a new request from another peer.

The request is converted into a `Message` (for clients) or an `Event` (for validators) and send to another peer.
In the absence of failures, the peer replies with a *block response* containing all requested blocks.

Importantly, at this stage, `BlockSync` cannot determine yet which blocks are correct and which are not.
It may fetch conflicting blocks concurrently from different peers, and only applies them to the ledger once it has detected that a (chain of) block(s) was confirmed by the network (see step 3).

### 2. Processing of a Block Response



### 3. Block Confirmation or Rejection

## Example



## Past Implementation

TODO
