//! Chain
//!
//! This crate the core block chain functionality, including the `[Chain]` and `[Block]`
//! modules, that constitute the Block Chain basic datastructures. It also contains the
//! `[Miner]`struct, responsible for mining new blocks and aggregating block entries such as
//! `[Transaction]`s and `[Record]`s into the newly added blocks.
#![warn(missing_docs)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![deny(clippy::panic)]
#![deny(clippy::unimplemented)]
#![deny(clippy::dbg_macro)]
#![deny(clippy::wildcard_imports)]
#![warn(clippy::cast_possible_truncation)]
#![warn(clippy::cast_precision_loss)]
#![warn(clippy::too_many_arguments)]
#![warn(clippy::large_enum_variant)]
#![warn(clippy::pedantic)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]

#[warn(missing_docs)]
/// Contains the `[Chain]` struct.
pub mod chain;

#[allow(clippy::module_inception)]
/// Contains the `[Block]` struct and `[BlockEntry]` trait.
pub mod block {

    /// Contains the `[Block]` struct.
    pub mod block;
    /// Contains the `[BlockEntry]` trait.
    pub mod block_entry;
}

/// Contains the `[Miner]` struct.
#[allow(clippy::module_inception)]
pub mod miner {
    /// Contains the `[Miner]` struct.
    pub mod miner;
}
