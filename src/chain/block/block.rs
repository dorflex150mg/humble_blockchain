pub mod block {
    use crate::Transaction;

    use std::time::{SystemTime, UNIX_EPOCH};
    use std::fmt;

    use sha2::{Digest, Sha256};
    use serde::{Deserialize, Serialize};
    use thiserror::Error;


    pub const MAX_TRANSACTIONS: usize = 8;
    pub const TRANSACTION_OFFSET: usize = 250;
    pub const N_TRANSACTION_PARAMS: usize = 6;

    pub const FIELD_END: char = ';';

    #[derive(Default, Debug, Clone, Serialize, Deserialize)]
    pub struct Block {
        pub index: usize,
        pub previous_hash: String,
        pub hash: String,
        pub data: String,
        pub timestamp: u64,
        pub nonce: u64,
    }

    #[derive(Error, Debug)]    
    pub enum InvalidTransactionErr {
        IncompleteChain,
        UnknownCoin,
    }
    
    impl fmt::Display for InvalidTransactionErr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                InvalidTransactionErr::IncompleteChain => write!(f, "The last owner of this coin is not this transaction's spender."),
                InvalidTransactionErr::UnknownCoin => write!(f, "The coin spent in this transaction is not valid."),
            }
        }
    }

    pub fn check_transaction(transaction: Transaction, blocks: &Vec<Block>) -> 
            Result<Transaction, InvalidTransactionErr> {
        let coins = &transaction.coins;
        for coin in coins { //verify each coin is valid:
            let mut coin_found = false;
            for block in blocks.iter().rev().collect::<Vec<&Block>>() { //check each block
                for t in block.get_transactions() { //check each transaction in the block
                    println!("coin in transaction: {}", t.coins[0]);
                    if t.coins[0] == *coin { 
                        coin_found = true; //if the coin gets found, check if the spender is
                                           //the last owner of the coin
                        if t.receiver != transaction.sender { // fail if sender doesnt own the
                                                              // coin
                            return Err(InvalidTransactionErr::IncompleteChain); 
                        }
                        break;
                    }
                }            
            }
            if !coin_found { // if the coin is not in any blocks, fail
                return Err(InvalidTransactionErr::UnknownCoin); 
            }
        }
        Ok(transaction)
    }

    impl Block {
        pub fn new(index: usize, previous_hash: String, data: String, hash: Option<String>) -> Block { 
            let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
            let private_hash = match hash {
                Some(h) => h,
                None => String::new(),
            };
            Block {
                index,
                previous_hash,
                data,
                timestamp,
                hash: private_hash, 
                nonce: 0,
            }
        }

        pub fn get_transactions(&self) -> Vec<Transaction> { 
            let mut transactions = vec![];
            let mut separator_counter = 1;
            let mut last_tx = 0;
            for i in 0..self.data.len() {
                if self.data[i..].chars().next().unwrap() == FIELD_END { //this is the 'byte' way of indexing
                    separator_counter += 1;
                }
                if separator_counter % N_TRANSACTION_PARAMS == 0 {
                    let str_transaction = String::from(&self.data[last_tx..i + 1]);
                    transactions.push(Transaction::try_from(str_transaction).unwrap());
                    last_tx = i + 1;
                }
            }
            transactions
        }

        pub fn get_hash(&self) -> String {
            self.hash.clone()
        }

        pub fn calculate_hash(&mut self) -> String {
            let str_block = format!("{}{}{}{}{}{}",
                             self.hash,
                             self.previous_hash,
                             self.data,
                             self.timestamp,
                             self.index,
                             self.nonce,
            );
            let mut hasher = Sha256::new();
            hasher.update(str_block);
            let digest = hasher.finalize();
            format!("{:x}", digest)
        }
    }

    impl fmt::Display for Block {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "index: {}, previous hash: {}, hash: {}, timestamp: {}", self.index, self.previous_hash, self.hash, self.timestamp)
        }
    }
}

