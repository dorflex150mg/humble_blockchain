mod chain {
    pub mod chain;
    pub mod block {
        pub mod block;
    }
}

mod miner {
    pub mod miner;
}

mod wallet {
    pub mod wallet;
}

mod transaction {
    pub mod transaction;
}

mod node {
    pub mod node;
    pub mod gossip;
    pub mod neighbour;
    pub mod protocol;
    pub mod receiver;
    pub mod reply;
    pub mod theme;
}

mod test {
    pub mod test_core;
    pub mod test_gossip;
}

use crate::miner::miner::miner::Miner as Miner;
use crate::chain::chain::chain::Chain as Chain;
use crate::wallet::wallet::wallet::Wallet as Wallet;
use crate::transaction::transaction::transaction::Transaction as Transaction;
use crate::test::test_core::test_core as test_core;
use crate::test::test_gossip::test_gossip as test_gossip;

use std::thread;
use std::sync::{Arc, Mutex};


#[tokio::main]
async fn main() {
    init_tracing();

    test_gossip::test_gossip().await;
    //test_core::test_core();
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
