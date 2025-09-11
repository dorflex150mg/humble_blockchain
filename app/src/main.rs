//! # `HumbleBlockchain`
//!                                                                                    //
//! Entry point for the crypto project. Initializes tracing and starts the application.//
//! Uses the following crates:                                                        
//! - `chain` for blockchain logic                                                     
//! - `miner` for mining operations                                                    
//! - `wallet` for wallet management                                                   
//! - `transaction` for transactions
//! - `network` for DHT, node, and object management                                  

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

use std::sync::Once;
use tracing_subscriber::fmt::writer::TestWriter;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

static INIT: Once = Once::new();

/// Initializes tracing for the application.
/// Ensures tracing is only initialized once, even in tests.
fn init_tracing() {
    INIT.call_once(|| {
        let env_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("trace")); // default if RUST_LOG not set

        let fmt_layer = fmt::layer()
            .with_writer(TestWriter::default()) // output visible in tests
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(false)
            .with_target(false);

        Registry::default().with(env_filter).with(fmt_layer).init();
    });
}

#[tokio::main]
async fn main() {
    init_tracing();

    tracing::info!("Application started successfully!");
}
