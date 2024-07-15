pub mod chain {

    use crate::chain::block::block::block::Block;
    use sha2::{Digest, Sha256};
    
    pub struct Chain {
        name: String,
        blocks: Vec<Block>,
        len: usize,
    }

    impl Chain {
        pub fn new(name: String) -> Self {
            let genesis_block = Block::new(0, "0".repeat(64), "0".repeat(64));
            let mut chain = Chain {
                name,
                blocks: vec![],
                len: 0,
            };
            chain.add_block(genesis_block).unwrap();
            chain
        }

        fn check_block_data(&self, data: String, previous_hash: String) -> Result<(), String> {
            let mut hasher = Sha256::new();
            hasher.update(data);
            let digest = hasher.finalize();  
            let digest_str = format!("{:x}", digest);
            if digest_str.get(..1).unwrap() != "0" || previous_hash != self.blocks.last().unwrap().hash {
                Err(String::from("Invalid Hash digest"))
            } else { 
                Ok(())
            }
        }

        pub fn add_block(&mut self, block: Block) -> Result<(), String> {
            let data = block.data.clone();
            let previous_hash = block.previous_hash.clone();
            if block.index != 0 {
                self.check_block_data(data, previous_hash)?;
            }
            let hash = block.hash.clone();
            self.blocks.push(block);
            //todo: create display for block
            println!("added block {}", hash);
            Ok(())
        }

        pub fn get_len(self) -> usize {
            self.len
        }
    }
}
