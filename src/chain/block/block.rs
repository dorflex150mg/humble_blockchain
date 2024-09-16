pub mod block {
    use std::time::{SystemTime, UNIX_EPOCH};
    use sha2::{Digest, Sha256};
    use std::fmt;

    use crate::Transaction;


    pub const MAX_TRANSACTIONS: usize = 8;
    pub const TRANSACTION_OFFSET: usize = 250;
    pub const N_TRANSACTION_PARAMS: usize = 6;

    pub const FIELD_END: char = ';';


    #[derive(Default, Debug, Clone)]
    pub struct Block {
        pub index: usize,
        pub previous_hash: String,
        pub hash: String,
        pub data: String,
        pub timestamp: u64,
        pub nonce: u64,
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

        pub fn get_transactions(&self) -> Vec<Transaction> { //TODO: There needs to be a separator
                                                             //between transactions in data
            let mut transactions = vec![];
            let mut cursor = self.data.chars();
            let mut cur_char = cursor.next();
            let mut field = 1;
            let mut transaction_params = vec![];
            while cur_char != None {
                let mut cur_field = String::from("");
                while cur_char != Some(FIELD_END) {
                    cur_field.push(cur_char.unwrap());
                    cur_char = cursor.next();
                }
                println!("Finished a field - {}", field);
                field += 1;
                transaction_params.push(cur_field);
                cur_char = cursor.next();
                if field == N_TRANSACTION_PARAMS {
                    let transaction = Transaction::from_base64(transaction_params).unwrap();
                    transactions.push(transaction);
                    field = 0;
                    transaction_params = vec![];
                }
            }
            println!("get transactions - n transactions: {}", transactions.len());
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

