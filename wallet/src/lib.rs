//! Wallet
//!
//! the `[Wallet]` crate contains the `[Wallet]` Struct and the types that
//! get directly modified by a `Wallet`, namely the `[BlockEntry]` types:
//! * `[Transaction]`: Represents a transaction between `Wallet`s.
//! * `[Record]`: Some data submitted by some `Wallet` to be inserte in `[BlockChainBlock]`.

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

/// Module containing dependency inversion types for `Block` and `Chain`.
pub mod block_chain;
/// Module containing the `[Wallet]` struct.
pub mod wallet;
/// Modules containing `[BlockEntry]` types.
pub mod transaction {
    /// Module containing the `[BlockEntry]` trait.
    pub mod block_entry_common;
    /// Module containing the `[Record]` struct.
    pub mod record;
    #[allow(clippy::module_inception)]
    /// Module containing the `[Transaction]` struct.
    pub mod transaction;
}
/// Module containing the `[Token]` struct.
pub mod token;
