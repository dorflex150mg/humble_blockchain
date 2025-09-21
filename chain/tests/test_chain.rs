#[cfg(test)]
pub mod tests {
    use chain::{chain::Chain, miner::miner::Miner};
    use wallet::block_chain::BlockChainBlock;
    use wallet::transaction::block_entry_common::BlockEntry;
    use wallet::transaction::record::Record;

    #[test]
    fn test_new() {
        let chain = Chain::new();
        assert_eq!(chain.get_blocks().len(), 1);
        assert_eq!(chain.get_len(), 1);
        assert_eq!(chain.difficulty, 1);
    }

    #[test]
    fn test_search() {
        let mut chain = Chain::new();
        let mut miner1 = Miner::new(1, String::from("Miner 1"), chain.clone());
        let last_block = chain.get_last_block();

        let mining_digest = miner1.mine(last_block).unwrap();
        let _ = chain.add_block(&mining_digest);
        let one_token = miner1.wallet.get_coins().pop().unwrap();
        let r1 = Record::new(
            miner1.wallet.get_pub_key(),
            "my_key",
            "a long and long and long and long long time."
                .as_bytes()
                .to_vec(),
            vec![one_token],
        );
        //let signed_t1 = miner1.wallet.sign(r1.clone());
        // Update miner1 with the latest chain metadata and mine a block with the transaction
        let last_block = chain.get_last_block();
        miner1.set_chain_meta(chain.clone());
        miner1.push_entry(r1.clone_box());
        let mining_digest = miner1.mine(last_block).unwrap();
        chain.add_block(&mining_digest).unwrap();
        let last_block = chain.get_last_block();
        println!("{}", last_block);
        let mut res = last_block.get_records();
        println!("{:?}", &res);
        assert!(res.pop().unwrap() == r1);
    }
}
