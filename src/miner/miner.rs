pub mod miner {

    use std::sync::{Arc, Mutex};
    use rand::{self, Rng};
    use std::fmt;

    use crate::chain::block::block::block::{Block};
    use crate::chain::block::block::block;

    use crate::transaction::transaction::transaction::Transaction;
    use crate::Chain;
    use crate::Wallet;

    #[derive(Clone)]
    pub struct ChainMeta {
        pub len: usize,
        pub last_hash: String
    }


    pub struct Miner {
        pub id: u64,
        pub name: String,
        pub wallet: Wallet,
        pub transactions: Vec<Transaction>,
        pub chain_meta: Option<ChainMeta>
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

        pub fn mine(&mut self, mut block: Block) -> Option<Block> {
            let mut count = 0;
            loop {
                count += 1;
                let mut rng = rand::thread_rng();
                block.nonce  = rng.gen_range(0..=u64::MAX);
                let str_digest = block.calculate_hash();
                if str_digest.chars().next().unwrap() == '0' {
                    println!("found digest: {} in attept: {}", str_digest.clone(), count);
                    return Some(self.create_new_block(str_digest));
                } else {
                    continue;
                }
            }
        }

        pub fn set_chain_meta(&mut self, len: usize, last_hash: String) {
            self.chain_meta = Some(ChainMeta {
                                     len,
                                     last_hash,
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

        pub fn create_new_block(&mut self, hash: String) -> Block { // will receive
                                                                                   // transactions
                                                                                   // as argument
            let index = self.chain_meta.clone().unwrap().len + 1; 
            let previous_hash = self.chain_meta.clone().unwrap().last_hash;
            let mut encoded_transactions: Vec<String> = self.transactions.iter().map(|transaction| {
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
