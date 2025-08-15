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

use crate::miner::miner::Miner as Miner;
use crate::chain::chain::Chain as Chain;
use crate::wallet::wallet::Wallet as Wallet;
use crate::transaction::transaction::Transaction as Transaction;


#[tokio::main]
async fn main() {
    init_tracing();
}

pub fn init_tracing() {
    use tracing::level_filters::LevelFilter;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::EnvFilter;

    let env = EnvFilter::builder()
        .with_default_directive(LevelFilter::DEBUG.into())
        .with_env_var("RUST_LOG")
        .from_env_lossy();

    let fmt_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_target(false);
    tracing_subscriber::registry()
        .with(fmt_layer)
        .with(env)
        .init();
}
