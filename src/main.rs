
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
    my_chain.print_last_block();
    let miner = Miner::new(1, String::from("some_miner"));
    println!("miner created -> {}", miner); 


    println!("Hello, world!");
}
