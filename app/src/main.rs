//! # HumbleBlockchain 
//!                                                                                    //
//! Entry point for the crypto project. Initializes tracing and starts the application.//
//! Uses the following crates:                                                         // Turn on stricter lint groups
//! - `chain` for blockchain logic                                                     #![warn(clippy::pedantic)]   // enable pedantic checks
//! - `miner` for mining operations                                                    #![warn(clippy::nursery)]    // enable experimental, but useful checks
//! - `wallet` for wallet management                                                   #![warn(missing_docs)]
//! - `transaction` for transactions
//! - `network` for DHT, node, and object management                                   // Explicitly forbid dangerous practices
#![deny(clippy::unwrap_used)]  // ban .unwrap()
#![deny(clippy::expect_used)]  // ban .expect()
#![deny(clippy::panic)]        // ban explicit panic! calls
#![deny(clippy::todo)]         // ban todo! placeholders

// Silence some of the overly nitpicky pedantic lints
#![allow(clippy::module_name_repetitions)] // don’t complain if module names repeat
#![allow(clippy::missing_panics_doc)]      // don’t force panic! documentation
#![allow(clippy::missing_errors_doc)]      // don’t force error-return documentation

use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt, Registry};
use tracing_subscriber::fmt::writer::TestWriter;
use std::sync::Once;

use chain::chain::Chain;
use chain::miner::miner::Miner;
use wallet::wallet::Wallet;
use transaction::transaction::Transaction;

static INIT: Once = Once::new();

/// Initializes tracing for the application.
/// Ensures tracing is only initialized once, even in tests.
fn init_tracing() {
    INIT.call_once(|| {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("trace")); // default if RUST_LOG not set

        let fmt_layer = fmt::layer()
            .with_writer(TestWriter::default()) // output visible in tests
            .compact()
            .with_file(true)
            .with_line_number(true)
            .with_thread_ids(false)
            .with_target(false);

        Registry::default()
            .with(env_filter)
            .with(fmt_layer)
            .init();
    });
}

#[tokio::main]
async fn main() {
    init_tracing();

    tracing::info!("Application started successfully!");
}
