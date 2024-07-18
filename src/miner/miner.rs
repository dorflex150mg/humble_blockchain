pub mod miner {

    use std::sync::{Arc, Mutex};
    use crate::chain::block::block::block::Block;
    use crate::Wallet;
    use rand::{self, Rng};
    use std::fmt;


    pub struct Miner {
        pub id: u64,
        pub name: String,
        pub wallet: Wallet,
    }
    
    impl Miner {
        pub fn new(id: u64, name: String) -> Self {
            let w_name = name.clone();
            Miner {
                id,
                name,
                wallet: Wallet::new(w_name),
            }
        }

        pub fn mine(&self, mut block: Block, winner: Arc<Mutex<Option<u64>>>) -> Option<String> {
            loop {
                let mut rng = rand::thread_rng();
                block.nonce  = rng.gen_range(0..=u64::MAX);
                let str_digest = block.calculate_hash();
                let mut won = winner.lock().unwrap(); 
                match *won {
                    Some(_) => return None,
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

    impl fmt::Display for Miner {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "id: {}, name: {}", self.id, self.name)
        }
    }
}
