
//mod block;
mod chain {
    pub mod chain;
    pub mod block {
        pub mod block;
    }
}

mod miner {
    pub mod miner;
}

use crate::miner::miner::miner::Miner as Miner;

use crate::chain::chain::chain::Chain as Chain;

fn main() {
    let my_chain = Chain::new(String::from("my_chain"));

    println!("Hello, world!");
}
