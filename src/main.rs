#[allow(clippy::module_inception)]
mod chain {
    pub mod chain;
    pub mod block {
        pub mod block;
    }
}

#[allow(clippy::module_inception)]
mod miner {
    pub mod miner;
}

#[allow(clippy::module_inception)]
mod wallet {
    pub mod wallet;
}

#[allow(clippy::module_inception)]
mod transaction {
    pub mod transaction;
}

pub mod dht;
pub mod node;
pub mod object;

pub mod tests;

use crate::miner::miner::Miner as Miner;
use crate::chain::chain::Chain as Chain;
use crate::wallet::wallet::Wallet as Wallet;
use crate::transaction::transaction::Transaction as Transaction;

#[tokio::main]
async fn main() {
    init_tracing();
}

use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt, Registry};
use tracing_subscriber::fmt::writer::TestWriter;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_tracing() {
    INIT.call_once(|| {
        let env_filter = EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("trace")); // default if RUST_LOG not set

        let fmt_layer = fmt::layer()
            .with_writer(TestWriter::default()) // ✅ output visible in cargo test
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

//pub fn init_tracing() {
//    use tracing::level_filters::LevelFilter;
//    use tracing_subscriber::prelude::*;
//    use tracing_subscriber::EnvFilter;
//
//    let env = EnvFilter::builder()
//        .with_default_directive(LevelFilter::DEBUG.into())
//        .with_env_var("RUST_LOG")
//        .from_env_lossy();
//
//    let fmt_layer = tracing_subscriber::fmt::layer()
//        .compact()
//        .with_file(true)
//        .with_line_number(true)
//        .with_thread_ids(false)
//        .with_target(false);
//    tracing_subscriber::registry()
//        .with(fmt_layer)
//        .with(env)
//        .init();
//}
