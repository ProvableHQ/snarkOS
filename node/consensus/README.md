# snarkos-node-consensus

[![Crates.io](https://img.shields.io/crates/v/snarkos-node-consensus.svg?color=neon)](https://crates.io/crates/snarkos-node-consensus)
[![Authors](https://img.shields.io/badge/authors-Aleo-orange.svg)](https://aleo.org)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](./LICENSE.md)

The crate builds on top of the `snarkos-node-bft`, which implements AleoBFT.
It manages a ratelimiter/mempool for incoming transmissions, and manages construction of blocks from batches that have been confirmed by the BFT layer.
