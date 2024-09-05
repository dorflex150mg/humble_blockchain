
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

use crate::miner::miner::miner::Miner as Miner;
use crate::chain::chain::chain::Chain as Chain;
use crate::wallet::wallet::wallet::Wallet as Wallet;
use crate::transaction::transaction::transaction::Transaction as Transaction;

use std::thread;
use std::sync::{Arc, Mutex};

fn main() {

    let mut my_chain = Chain::new(String::from("my_chain")); // a chain being created with a genesis block
    my_chain.print_last_block();
    let mut miner = Miner::new(1, String::from("some_miner"));
    println!("miner created -> {}", miner); 
    let wallet1 = Wallet::new(String::from("wallet1"));
    let last_block = my_chain.get_last_block();
    let hash = last_block.get_hash();
    miner.set_chain_meta(my_chain.get_len(), my_chain.difficulty, my_chain.get_blocks()); // mining a simple block 
    let (new_block, nonce) = match miner.mine(last_block, vec![]) {
        Ok((b, n)) => (b, n),
        Err(e) => panic!("Block mining failed: {}", e),
    };
    println!("Block mined by {}: {}", miner.name, new_block);
    match my_chain.add_block(new_block, nonce) {
        Ok(()) => (),
        Err(e) => println!("Failed add block with error: {}", e),
    }

    println!("Miner after block:\n{}", &miner);

    let one_token = miner.wallet.get_coins().pop().unwrap();
    let t1 = Transaction::new(miner.wallet.get_pub_key(), wallet1.get_pub_key(), vec![one_token]);
    let signed_t1 = miner.wallet.sign(t1);
    //miner.set_transactions(vec![signed_t1]); //this is ugly and for only for testing
    let last_block = my_chain.get_last_block();
    let hash = last_block.get_hash();
    miner.set_chain_meta(my_chain.get_len(), my_chain.difficulty, my_chain.get_blocks());
    let (newer_block, new_nonce) = match miner.mine(my_chain.get_last_block(), vec![signed_t1]) { // mining with transactions
        Ok((b, n)) => (b, n),
        Err(e) => panic!("Block mining failed: {}", e),
    };
    println!("Block mined by {}: {}", miner.name, &newer_block);

    match my_chain.add_block(newer_block, new_nonce) {
        Ok(()) => (),
        Err(e) => println!("Failed add block with error: {}", e),
    }

    let chain_arc = Arc::new(Mutex::new(my_chain));
    let other_chain_arc = chain_arc.clone();
                                                                   // TODO: chain should
                                                                   // be clonable
    let mut miner2 = Miner::new(2, String::from("another_miner")); // two miners 
    let chain2 = chain_arc.clone();
    let handle = thread::spawn(move || {
        for i in 0..100 {
            let last_block = chain2.lock().unwrap().get_last_block();
            let hash = last_block.get_hash();
            let chain_len = chain2.lock().unwrap().get_len();
            let difficulty = chain2.lock().unwrap().difficulty;
            miner2.set_chain_meta(chain_len, difficulty, chain.lock().unwrap().get_blocks());
            let (newer_block, new_nonce) = match miner2.mine(last_block, vec![]) {
                Ok((b, n)) => (b, n),
                Err(e) => panic!("Block mining failed: {}", e),
            };
            println!("Block mined by {}: {}", miner2.name, &newer_block);
            match chain2.lock().unwrap().add_block(newer_block, new_nonce) {
                Ok(()) => (),
                Err(e) => println!("Failed add block with error: {}", e),
            }
        }
    });

    for i in 0..100 {
        println!("miner 1 mining\n\n\n\n");
        let chain = other_chain_arc.clone();
        let last_block = chain.lock().unwrap().get_last_block();
        let hash = last_block.get_hash();
        let chain_len = chain.lock().unwrap().get_len();
        let difficulty = chain.lock().unwrap().difficulty;
        println!("miner 1 setting chain meta");
        miner.set_chain_meta(chain_len, difficulty, chain.lock().unwrap().get_blocks());
        let (newer_block, new_nonce) = match miner.mine(last_block, vec![]) {
            Ok((b, n)) => (b, n),
            Err(e) => panic!("Block mining failed: {}", e),
        };
        println!("Block mined by {}: {}", miner.name, &newer_block);
        match chain.lock().unwrap().add_block(newer_block, new_nonce) {
            Ok(()) => (),
            Err(e) => println!("Failed add block with error: {}", e),
        };
    }
    
}
