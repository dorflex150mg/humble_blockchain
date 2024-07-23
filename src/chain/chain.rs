pub mod chain {

    use crate::chain::block::block::block::Block;
    use std::fmt;
    use sha2::{Digest, Sha256};

    const interval: u64 = 60; //difficulty increases if mining a block takes more than 1 minute
    
    pub struct Chain {
        name: String,
        blocks: Vec<Block>,
        len: usize,
        pub difficulty: usize,
    }
    
    #[derive(Debug)]
    pub enum BlockCheckError {
        InvalidPrefix(usize),
        NotInChain {expected: String, got: String},
        WrongHash {expected: String, got: String},
    }


    impl fmt::Display for BlockCheckError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                BlockCheckError::InvalidPrefix(difficulty) => write!(f, "Invalid prefix - Not enough \"0\"'s at the beginning. Current difficulty: {}", difficulty),
                BlockCheckError::NotInChain {expected, got} => write!(f, "Previous hash not in chain. Expected: {}, but got: {}", expected, got),
                BlockCheckError::WrongHash {expected, got} => write!(f, "Wrong hash. Expected: {}, but got: {}", expected, got),
            }
        }
    }

    impl Chain {
        pub fn new(name: String) -> Self {
            let mut genesis_block = Block::new(0, "0".repeat(64), "0".repeat(64), Some("0".repeat(64)));
            let mut chain = Chain {
                name,
                blocks: vec![],
                len: 0,
                difficulty: 1,
            };
            chain.add_block(genesis_block, 0).unwrap();
            chain
        }

        fn check_block_data(&self, data: String, previous_hash: String, block_hash: String) -> Result<(), BlockCheckError> {
            let mut hasher = Sha256::new();
            hasher.update(data);
            let digest = hasher.finalize();  
            let digest_str = format!("{:x}", digest);
            if !digest_str.starts_with(&"0".repeat(self.difficulty)) {
                return Err(BlockCheckError::InvalidPrefix(self.difficulty));
            }
            let last_chain_hash = self.blocks.last().unwrap().hash.clone(); 
            if previous_hash != last_chain_hash {
                return Err(BlockCheckError::NotInChain {expected: previous_hash, got: last_chain_hash}); 
            }
            if digest_str != block_hash {
                Err(BlockCheckError::WrongHash {expected: digest_str, got: block_hash})
            } else { 
                println!("It's alive!!");
                Ok(())
            }
        }

        fn check_difficulty(&mut self, block_timestamp: u64) {
            if block_timestamp < self.blocks.iter().last().unwrap().timestamp + interval { 
                println!("difficulty just went up");
                self.difficulty += 1;
            }
        }

        pub fn get_last_block(&self) -> Block {
            self.blocks.iter().last().unwrap().clone() //impossible to have a chain with 0 blocks
        }

        pub fn add_block(&mut self, block: Block, nonce: u64) -> Result<(), BlockCheckError> {
            if block.index != 0 {
                //todo: check last block instead of the new block
                let last_block = self.blocks.iter().last().clone().unwrap();
                let str_block = format!("{}{}{}{}{}{}",
                                 last_block.hash,
                                 last_block.previous_hash,
                                 last_block.data,
                                 last_block.timestamp,
                                 last_block.index,
                                 nonce, //add mined nonce
                );
                let data = str_block.clone();
                let previous_hash = last_block.previous_hash.clone();
                let block_hash = block.previous_hash.clone();
                self.check_block_data(data, previous_hash, block_hash)?;
                self.check_difficulty(block.timestamp);
            }
            self.blocks.push(block);
            Ok(())
        }

        pub fn get_len(&self) -> usize {
            self.len
        }

        pub fn print_last_block(&self) {
            println!("{}", self.blocks.last().unwrap()); 
        }
    }
}
