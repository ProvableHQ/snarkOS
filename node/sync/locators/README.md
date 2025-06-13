# snarkos-node-sync-locators

[![Crates.io](https://img.shields.io/crates/v/snarkos-node-sync-locators.svg?color=neon)](https://crates.io/crates/snarkos-node-sync-locators)
[![Authors](https://img.shields.io/badge/authors-Aleo-orange.svg)](https://aleo.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE.md)

The `snarkos-node-sync-locators` crate provides _block locators_,
which are data structures used by nodes to advertise to other nodes the blocks in their possession.
Block locators are then provided to other nodes to help the latter sync their blockchain with the rest of the network.

In the absence of Byzantine failures, e.g., malicious nodes, a block's height, i.e. its position in the chain, uniquely identifies it, because it is impossible for there to be forks or conflicting blocks.
However, such assumptions cannot be made in production and a attacker may attempt to propagate an invalid block.
Block locators protect against this by including the hash of a block in addition to its height.
If two nodes advertise the same block has for the same height, we know with very high probability that they advertise the same block.
As a result, each block locator contains not only a block's height, but also a block's hash.

The `BlockLocators` struct in this crate represents a set of block locators.
More concretely `BlockLocators` contains a continuous sequence of block locators starting at some block height. 

Besides the `BlockLocators` struct, this crate provides operations
to construct block locators, to check them for well-formedness and consistency,
and to serialize and deserialize them to and from bytes.

## Usage During Sync

These locators are generated on demand when a peer requests locators for a specific range.
Peers will request specific block locator ranges based on their current sync height and the maximum locator height a node advertises in its Ping message.

## Previous BlockLocator Design
In the past, BlockLocators were considerably more complex.
They were organized as two maps from block heights to block hashes: a `checkpoints` map and a `recents` map, which the following figure illustrates.

![Block Locators](block-locators.png)
The rectangular bar represents the whole blockchain; each circle represents a block locator.

This enabled checking consistency between block locators to detect malicious peers.
snarkOS has since moved to verifying every block on sync, so such checks are no longer needed.
