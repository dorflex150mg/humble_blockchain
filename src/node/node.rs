pub mod node {

    use crate::Wallet;
    use crate::Chain;
    use crate::Transaction;
    use crate::node::gossip::gossip;
    use crate::node::neighbour::neighbour::{
        Neighbour,
        Role,
    };
    use crate::node::protocol::protocol;

    use std::fmt;

    use uuid::{
        self,
        Uuid, 
    };
    use thiserror::Error;
    use rand::prelude::*;

    #[derive(Error, Debug)] 
    pub enum EnterAttemptError {
        NoListeners
    }

    impl fmt::Display for EnterAttemptError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to enter network - No trackers listening")
        }
    }

    #[derive(Error, Debug)] 
    pub enum UpdateChainError {
        NoListeners
    }

    impl fmt::Display for UpdateChainError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to update chain - No neighbours listening")
        }
    }
    
    pub struct Node {
        id: Uuid,
        role: Role,
        wallet: Wallet,
        chain: Chain,
        neighbours: Vec<Neighbour>,
        new_neighbours: Vec<Neighbour>,
    }

    impl Node {

        fn enterNetwork(&mut self, id: String, trackers: Vec<String>) -> Result<(), EnterAttemptError> {
            let mut cleared = false;
            for tracker in trackers {
                match gossip::greet(tracker) {
                    Ok(neighbour) => {
                        self.neighbours.push(neighbour);
                        cleared = true;
                    },
                    Err(e) => continue,
                }
            }
            if !cleared {
                return Err(EnterAttemptError::NoListeners)
            }
            Ok(())
        }

        fn leaveNetwork(&self) {
            for neighbour in &self.neighbours {
                gossip::farewell(neighbour.address.clone());
            }
        }         


        fn submitTransaction(&self, transaction: Transaction) {
            self.neighbours
                .iter()
                .filter(|neighbour| { neighbour.role == Role::Miner })
                .map(|miner| { gossip::sendTransaction(miner.address.clone(), transaction.clone()) })
                .collect::<Vec<_>>();
        }

        fn updateChain(&self) -> Result<Chain, UpdateChainError> {
            let mut cursor = self.neighbours.iter();
            let mut cur_neighbour = cursor.next();
            while cur_neighbour != None {
                match gossip::pollChain(cur_neighbour.unwrap()) {
                    Ok(chain) => return Ok(chain),
                    Err(e) => cur_neighbour = cursor.next(),
                }
            }
            Err(UpdateChainError::NoListeners)
        }

        fn gossip(&self) {
            gossip::waitGossipInterval();
            let random_neighbours = self.getRandomNeighbours();
            for neighbour in random_neighbours {
                gossip::sendChain(neighbour.address.clone(), self.chain.clone());
                gossip::sendNewNeighbours(neighbour.address.clone(), self.new_neighbours.clone());
            }
        }

        fn listen(&self) {}

        fn getRandomNeighbours(&self) -> Vec<Neighbour> {
            let mut neighbours = vec![];
            let mut rng = rand::thread_rng();
            let float_n = self.neighbours.len() as f64;
            let n: usize = float_n.sqrt().floor() as usize;
            for i in 0..n {
                let random: usize = rng.gen_range(0..self.neighbours.len());
                neighbours.push(self.neighbours[random].clone());
            }
            neighbours
        }
    }
}

    
