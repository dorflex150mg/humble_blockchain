pub mod miner {

    use rand::{self, Rng};
    use std::fmt;
    use std::cmp;

    use crate::chain::block::block::block::{Block};
    use crate::chain::block::block::block;

    use crate::transaction::transaction::transaction::Transaction;
    use crate::Wallet;

    #[derive(Clone)]
    pub struct ChainMeta {
        pub len: usize,
        pub difficulty: usize,
    }

    pub struct UninitializedChainMetaErr;

    impl fmt::Display for UninitializedChainMetaErr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "This miner's ChainMeta has not been initialized. Consider set_chain_meta(len, last_hash, difficulty)")
        }
    }

    pub struct Miner {
        pub id: u64,
        pub name: String,
        pub wallet: Wallet,
        pub transactions: Vec<Transaction>,
        pub chain_meta: Option<ChainMeta>,
    }
    
    impl Miner {
        pub fn new(id: u64, name: String) -> Self {
            let w_name = name.clone();
            Miner {
                id,
                name,
                wallet: Wallet::new(w_name),
                transactions: vec![],
                chain_meta: None,
            }
        }

        pub fn mine(&mut self, mut block: Block, mut transactions: Vec<Transaction>) -> Result<(Block, u64), UninitializedChainMetaErr> {
            self.transactions = transactions;
            let chain_meta = self.chain_meta.as_ref().ok_or(UninitializedChainMetaErr)?;
            let mut count = 0;
            loop {
                count += 1;
                let mut rng = rand::thread_rng();
                block.nonce  = rng.gen_range(0..=u64::MAX);
                let str_digest = block.calculate_hash();
                if str_digest.starts_with(&"0".repeat(chain_meta.difficulty)) {
                    println!("found digest: {} in attept: {}", str_digest.clone(), count);
                    return Ok((self.create_new_block(str_digest, block.hash.clone()), block.nonce));
                } else {
                    continue;
                }
            }
        }

        pub fn set_chain_meta(&mut self, len: usize, difficulty: usize) {
            self.chain_meta = Some(ChainMeta {
                                     len,
                                     difficulty,
                                })
        }
            

        pub fn set_transactions(&mut self, new_transactions: Vec<Transaction>) {
            self.transactions = new_transactions;
        }

        //pub fn get_transactions(&mut self) -> Vec<Transaction> { // in the future, miner will not own
        //                                                     // transactions, hence this method
        //    let mut transactions: Vec<Transaction> = vec![];
        //    for i in 0..block::MAX_TRANSACTIONS {
        //        match self.transactions.iter().next().clone() {
        //            Some(t) => transactions.push(*t),
        //            None => break,
        //        }
        //    }
        //    transactions
        //}

        pub fn create_new_block(&mut self, hash: String, previous_hash: String) -> Block { // will receive
                                                                                   // transactions
                                                                                   // as argument
            let index = self.chain_meta.clone().unwrap().len + 1; 
            let cap = cmp::min(self.transactions.len(), block::MAX_TRANSACTIONS);
            let mut capped_transactions: Vec<Transaction> = self.transactions.drain(0..cap).collect();
            let mut encoded_transactions: Vec<String> = capped_transactions.iter().map(|transaction| {
                                                               transaction.to_base64()
                                                            }).collect();
            encoded_transactions.push(hash.clone());
            let data = encoded_transactions.join("");
            self.wallet.add_coin(hash.clone());
            Block::new(index, previous_hash, data, Some(hash)) 
        }
    }

    impl fmt::Display for Miner {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            //println!("{}:\n{}", "miner wallet: ", self.wallet);
            let joint_coins = self.wallet.coins.join(",");
            write!(f, "id: {}, name: {}, wallet: {}", self.id, self.name, joint_coins)
        }
    }
}
