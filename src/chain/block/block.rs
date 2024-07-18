pub mod block {
    use std::time::{SystemTime, UNIX_EPOCH};
    use sha2::{Digest, Sha256};
    use std::fmt;

    pub struct Block {
        pub index: u64,
        pub previous_hash: String,
        pub hash: String,
        pub data: String,
        timestamp: u64,
        pub nonce: u64,
    }

    impl Block {
        pub fn new(index: u64, previous_hash: String, data: String) -> Block { 
            let timestamp = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs();
            Block {
                index,
                previous_hash,
                data,
                timestamp,
                hash: String::new(),
                nonce: 0,
            }
        }

        pub fn calculate_hash(&mut self) -> String {
            let str_block = format!("{}{}{}{}{}",
                             &self.hash,
                             &self.previous_hash,
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

