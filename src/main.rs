
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

mod wallet {
    pub mod wallet;
}

mod transaction {
    pub mod transaction;
}

use std::sync::{Arc, Mutex};

use crate::miner::miner::miner::Miner as Miner;
use crate::chain::chain::chain::Chain as Chain;
use crate::wallet::wallet::wallet::Wallet as Wallet;

fn main() {
    let my_chain = Chain::new(String::from("my_chain"));
    my_chain.print_last_block();
    let mut miner = Miner::new(1, String::from("some_miner"));
    println!("miner created -> {}", miner); 
    let wallet1 = Wallet::new(String::from("wallet1"));
    let wallet2 = Wallet::new(String::from("wallet2"));
    //let winner = None; 
    //let winner_arc = Arc::new(Mutex::new(winner));
    let last_block = my_chain.get_last_block();
    let hash = last_block.get_hash();
    miner.set_chain_meta(my_chain.get_len(), hash);
    let block = miner.mine(last_block);
    println!("Block mined: {}", block.unwrap());
    println!("Miner after block:\n{}", miner);
}
