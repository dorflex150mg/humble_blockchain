pub mod test_core {
    
    use crate::{
        miner::miner::miner::Miner as Miner,
        chain::chain::chain::Chain as Chain,
        wallet::wallet::wallet::Wallet as Wallet,
        transaction::transaction::transaction::Transaction as Transaction,
    };

    use std::thread;
    use std::sync::{Arc, Mutex};


    pub fn test_core() {
        let mut my_chain = Chain::new(); // a chain being created with a genesis block
        my_chain.print_last_block();
        let mut miner = Miner::new(1, String::from("miner 1"));
        println!("miner created -> {}", miner); 
        let wallet1 = Wallet::new();
        let last_block = my_chain.get_last_block();
        miner.set_chain_meta(my_chain.get_len(), my_chain.difficulty, my_chain.get_blocks()); // mining a simple block 
        let (new_block, nonce) = match miner.mine(last_block, vec![]) {
            Ok((b, n)) => (b, n),
            Err(e) => panic!("Block mining failed: {}", e),
        };
        println!("Block mined by {}: {}", miner.name, new_block);
        println!("new block data: {:?}", &new_block.data);
        match my_chain.add_block(new_block, nonce) {
            Ok(()) => (),
            Err(e) => println!("Failed add block with error: {}", e),
        }
    
        println!("Miner after block:\n{}", &miner);
    
        let one_token = miner.wallet.get_coins().pop().unwrap();
        let t1 = Transaction::new(miner.wallet.get_pub_key(), wallet1.get_pub_key(), vec![one_token]);
        let signed_t1 = miner.wallet.sign(t1);
        miner.set_chain_meta(my_chain.get_len(), my_chain.difficulty, my_chain.get_blocks());
        println!("mining with signed_t1");
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

        let mut miner2 = Miner::new(2, String::from("miner 2")); // two miners 
        let chain2 = chain_arc.clone();
        let _ = thread::spawn(move || {
            for _ in 0..100 {
                println!("miner 2 mining\n\n\n\n");
                let last_block = chain2.lock().unwrap().get_last_block();
                let chain_len = chain2.lock().unwrap().get_len();
                let difficulty = chain2.lock().unwrap().difficulty;
                miner2.set_chain_meta(chain_len, difficulty, chain2.lock().unwrap().get_blocks());
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

        for _ in 0..100 {
            println!("miner 1 mining\n\n\n\n");
            let chain = other_chain_arc.clone();
            let last_block = chain.lock().unwrap().get_last_block();
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
}
