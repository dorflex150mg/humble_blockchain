pub mod miner {

    use std::sync::{Arc, Mutex};
    use crate::chain::block::block::block::Block;
    use rand::{self, Rng};


    pub struct Miner {
        pub id: u64,
        pub name: String,
    }
    
    impl Miner {
    
        pub fn new(self, id: u64, name: String) -> Self {
            Miner {
                id,
                name,
            }
        }

        pub fn mine(&self, mut block: Block, winner: Arc<Mutex<Option<u64>>>) -> Option<String> {
            loop {
                let mut rng = rand::thread_rng();
                block.nonce  = rng.gen_range(0..=u64::MAX);
                let str_digest = block.calculate_hash();
                let mut won = winner.lock().unwrap(); 
                match *won {
                    Some(id) => return None,
                    None => {
                        if str_digest.chars().next().unwrap() == '0' {
                            *won = Some(self.id); 
                            return Some(str_digest);
                        } else {
                            continue;
                        }
                    },
                }
            }
        }
    }
}
