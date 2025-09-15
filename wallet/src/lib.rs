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

pub mod block_chain;
pub mod wallet;
pub mod transaction {
    pub mod block_entry_common;
    pub mod record;
    #[allow(clippy::module_inception)]
    pub mod transaction;
}
pub mod token;
