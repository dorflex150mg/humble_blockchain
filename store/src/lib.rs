//! Store
//!
//! Provides engine options for the storage of `Chain` data through the `[Store]` struct.
//! By default, the `[FileEngine]` is used, if no engine is specified when creating the `Store`
//! struct. The `FileEngine` is composed of a `BufReader` and a `BufWriter`.

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
mod engine;
mod file_engine;
/// Module that contains the `[Store]` trait.
pub mod store;
