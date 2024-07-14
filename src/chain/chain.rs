pub mod chain {

    use crate::chain::block::block::block::Block;
    use sha2::{Digest, Sha256};
    
    pub struct Chain {
        name: String,
        blocks: Vec<Block>,
        len: usize,
    }

    impl Chain {
        fn new(name: String) -> Self {
            Chain {
                name,
                blocks: vec![],
                len: 0,
            }
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
            self.check_block_data(data, previous_hash)?;
            self.blocks.push(block);
            Ok(())
        }

        pub fn get_len(self) -> usize {
            self.len
        }
    }
}
