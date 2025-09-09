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

#[warn(missing_docs)]
pub mod chain;
#[allow(clippy::module_inception)]
pub mod block {
    pub mod block;
    pub mod block_entry;
}
#[allow(clippy::module_inception)]
pub mod miner {
    pub mod miner;
}
