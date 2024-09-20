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

    use std::{
        fmt,
        io::Result as IOResult, 
    };


    use uuid::{
        self,
        Uuid, 
    };
    use thiserror::Error;
    use rand::prelude::*;

    #[derive(Error, Debug)] 
    pub enum EnterAttemptError {
        NoListeners,
        NoTrackers,
    }

    impl fmt::Display for EnterAttemptError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                EnterAttemptError::NoListeners => write!(f, "Failed to enter network - No trackers listening."),
                EnterAttemptError::NoTrackers => write!(f, "Failed to enter network - No trackers available."),
            }
        }
    }

    #[derive(Error, Debug)] 
    pub enum UpdateChainError {
        NoListeners
    }

    impl fmt::Display for UpdateChainError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "Failed to update chain - No neighbours listening.")
        }
    }
    
    pub struct Node {
        id: Uuid,
        role: Role,
        wallet: Wallet,
        chain: Chain,
        neighbours: Vec<Neighbour>,
        new_neighbours: Vec<Neighbour>,
        initialized: bool,
        trackers: Option<Vec<String>>,
    }

    impl Node {

        pub async fn enterNetwork(&mut self) -> Result<(), EnterAttemptError> {
            let mut cleared = false;
            match &self.trackers {
                Some(ts) => {
                    for tracker in ts {
                        match gossip::greet(tracker.as_str()).await {
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
                },
                None => Err(EnterAttemptError::NoTrackers)
            }
        }

        pub async fn leaveNetwork(&self) {
            for neighbour in &self.neighbours {
                gossip::farewell(neighbour.address.clone()).await;
            }
        }         


        pub async fn submitTransaction(&self, transaction: Transaction) {
            self.neighbours
                .iter()
                .filter(|neighbour| { neighbour.role == Role::Miner })
                .map(|miner| async { gossip::sendTransaction(miner.address.clone(), transaction.clone()).await })
                .collect::<Vec<_>>();
        }

        pub async fn updateChain(&self) -> Result<Chain, UpdateChainError> {
            let mut cursor = self.neighbours.iter();
            let mut cur_neighbour = cursor.next();
            while cur_neighbour != None {
                match gossip::pollChain(cur_neighbour.unwrap()).await {
                    Ok(chain) => return Ok(chain),
                    Err(e) => cur_neighbour = cursor.next(),
                }
            }
            Err(UpdateChainError::NoListeners)
        }

        pub async fn gossip(&self) {
            gossip::waitGossipInterval().await;
            let random_neighbours = self.getRandomNeighbours();
            for neighbour in random_neighbours {
                gossip::sendChain(neighbour.address.clone(), self.chain.clone()).await;
                gossip::sendNewNeighbours(neighbour.address.clone(), self.new_neighbours.clone()).await;
            }
        }


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

        pub async fn listen(&mut self) -> IOResult<()> {
            loop {
                if self.initialized {
                    let (protocol, sender) = gossip::listenToGossip().await?;
                    match protocol {
                        protocol::GREET => self.presentId(sender).await?, 
                        protocol::FAREWELL => self.removeNeighbour(sender).await?,
                        protocol::NEIGHBOUR => self.addNeighbour().await?,
                        protocol::TRANSACTION => self.addTransaction().await?,
                        protocol::CHAIN => self.checkChain().await?,
                        protocol::POLLCHAIN => self.shareChain().await?, 
                        _ => () //TODO: Ignore with an error
                    }

                }
                self.enterNetwork().await;
            }
        }

        pub async fn presentId(&self, sender: String) -> IOResult<()>{ 
            let buffer = self.id.as_bytes(); 
            gossip::sendId(buffer, sender).await;
            Ok(())
        }

        pub async fn removeNeighbour(&mut self, sender: String) -> IOResult<()>{ 
            for i in 0..self.neighbours.len() {
                if self.neighbours[i].address == sender { // Rationale: Its better to match by ip than send an id
                    self.neighbours.remove(i);
                    return Ok(());
                }
            }
            Ok(())
        }

        pub async fn addNeighbour(&self) -> IOResult<()>{ 
            
            Ok(())
        }

        pub async fn addTransaction(&self) -> IOResult<()>{ Ok(())}

        pub async fn checkChain(&self) -> IOResult<()>{ Ok(())}

        pub async fn shareChain(&self) -> IOResult<()>{ Ok(())}
    }
}

    
