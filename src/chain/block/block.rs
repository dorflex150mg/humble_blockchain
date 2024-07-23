pub mod block {
    use std::time::{SystemTime, UNIX_EPOCH};
    use sha2::{Digest, Sha256};
    use std::fmt;


    pub const MAX_TRANSACTIONS: usize = 8;

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

