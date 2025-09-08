#![allow(unused_imports)]
use chain::chain::Chain;
#[cfg(test)]
use chain::miner::miner::Miner;
use wallet::transaction::transaction::Transaction;
use wallet::wallet::Wallet;

use std::sync::{Arc, Mutex};
use std::thread;
use tracing::info;

/// Tests the core functionality of the blockchain, mining, and transactions.
///
/// This function creates a blockchain with a genesis block, adds blocks with and without
/// transactions, and demonstrates two miners working in parallel on the same chain.
#[test]
pub fn test_core() {
    // Create a new blockchain and print the genesis block
    let mut my_chain = Chain::new();
    my_chain.print_last_block();

    // Create the first miner
    let mut miner1 = Miner::new(1, String::from("Miner 1"));
    info!("Miner created -> {}", miner1);

    // Create a wallet for future transactions
    let wallet1 = Wallet::new();

    // Setup mining metadata for miner1 and mine the first block
    let last_block = my_chain.get_last_block();
    miner1.set_chain_meta(
        my_chain.get_len(),
        my_chain.difficulty,
        my_chain.get_blocks(),
    );

    let res_mining_digest = miner1.mine(last_block);
    assert!(res_mining_digest.is_ok());
    let mining_digest = res_mining_digest.unwrap();

    // Log details about the mined block
    // Add the new block to the chain
    let res_my_chain = my_chain.add_block(mining_digest.clone());
    assert!(res_my_chain.is_ok());

    // Create a transaction from miner1 to wallet1 using one token
    let res_one_token = miner1.wallet.get_coins().pop();
    assert!(res_one_token.is_some());
    let one_token = res_one_token.unwrap();
    let t1 = Transaction::new(
        miner1.wallet.get_pub_key(),
        wallet1.get_pub_key(),
        vec![one_token],
    );
    let signed_t1 = miner1.wallet.sign(t1);

    // Update miner1 with the latest chain metadata and mine a block with the transaction
    miner1.set_chain_meta(
        my_chain.get_len(),
        my_chain.difficulty,
        my_chain.get_blocks(),
    );

    miner1.push_transaction(signed_t1);

    let res_new_mining_digest = miner1.mine(my_chain.get_last_block());
    assert!(res_new_mining_digest.is_ok());
    let new_mining_digest = res_new_mining_digest.unwrap();
    assert_ne!(new_mining_digest, mining_digest);

    // Log the newly mined block with the transaction
    info!(
        "Block mined by {}: {}",
        miner1.get_name(),
        new_mining_digest.get_block()
    );

    // Add the new block with transactions to the chain
    assert!(my_chain.add_block(new_mining_digest).is_ok());

    // Shared blockchain instance for multiple miners
    let chain_arc = Arc::new(Mutex::new(my_chain));
    let other_chain_arc = Arc::clone(&chain_arc);

    // Create the second miner
    let mut miner2 = Miner::new(2, String::from("Miner 2"));

    // Spawn a new thread for miner2 to mine blocks concurrently
    let iterations = 2;
    let chain_clone = Arc::clone(&chain_arc);
    thread::spawn(move || {
        for i in 0..iterations {
            info!("Miner 2 iteration: {}", i);
            let mut chain = chain_clone.lock().unwrap();
            let last_block = chain.get_last_block();
            let chain_len = chain.get_len();
            let difficulty = chain.difficulty;

            // Update miner2 with the latest chain metadata and mine a block
            miner2.set_chain_meta(chain_len, difficulty, chain.get_blocks());

            let res_mining_digest = miner2.mine(last_block);
            assert!(res_mining_digest.is_ok());

            let mining_digest = res_mining_digest.unwrap();
            // Log and add the mined block to the chain
            info!(
                "Block mined by {}: {}",
                miner2.get_name(),
                mining_digest.get_block()
            );
            assert!(chain.add_block(mining_digest).is_ok());
        }
    });

    // Miner1 continues mining in the main thread
    for i in 0..iterations {
        info!("Miner 1 iteration: {}", i);
        let chain = Arc::clone(&other_chain_arc);
        let last_block = chain.lock().unwrap().get_last_block();
        let chain_len = chain.lock().unwrap().get_len();
        let difficulty = chain.lock().unwrap().difficulty;

        // Update miner1 with the latest chain metadata and mine a block
        miner1.set_chain_meta(chain_len, difficulty, chain.lock().unwrap().get_blocks());

        let res_mining_digest = miner1.mine(last_block);
        assert!(res_mining_digest.is_ok());
        let mining_digest = res_mining_digest.unwrap();

        // Log and add the mined block to the chain

        if let Err(e) = chain.lock().unwrap().add_block(mining_digest) {
            info!("Failed to add block: {}", e);
        };
    }
}
