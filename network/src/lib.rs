//! Network
//!
//! The Network crate contains the modules responsible for running the gossip protocol.
//! A `[Node]` represents a node on the gossip protocol. `Node`s are responsible for sending copies
//! of their version of a `[Chain]` copies to each other. They can assume the `[Role::Tracker]` role, which serves as a gateway
//! to new participants. They can also assume the `[Role::Miner]` role, where they aggregate
//! transactions and try to mine a `[Block]`.

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

//pub mod dht;
/// Module containg the `[Node]`, the `[gossip]` module and their helper modules.
pub mod node;
//pub mod object;
