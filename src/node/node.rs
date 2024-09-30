pub mod node {

    use crate::{
        Wallet,
        Chain,
        Transaction,
        node::{
            neighbour::neighbour::{
                Neighbour,
                Role,
            },
            gossip::gossip,
            protocol::protocol,
            receiver::receiver::Receiver,
            reply::reply::Reply,
        },
    };

    use std::{
        collections::HashMap,
        io::{
            Result as IOResult, 
            Error,
        },
        fmt,
        str,
    };

    use thiserror::Error;
    use rand::prelude::*;
    use uuid::{
        self,
        Uuid, 
    };

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

    #[derive(Error, Debug, derive_more::From)]
    pub enum WrongRoleError {
        NotMiner,
        NotTracker,
    }

    impl fmt::Display for WrongRoleError{
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                WrongRoleError::NotMiner => write!(f, "That operation requires a Node with Role Miner."),
                WrongRoleError::NotTracker => write!(f, "That operation requires a Node with Role Tracker."),
            }
        }
    }

    #[derive(Error, Debug, derive_more::From, derive_more::Display)]
    pub enum ListenError {
        WrongRoleError(WrongRoleError),
        IOError(Error),
    }

    #[derive(Error, Debug, derive_more::From, derive_more::Display)]
    pub enum TransactionRecvError {
        TryRecvError(TryRecvError),
        TransactionFromBase64Error(TransactionFromBase64Error),
    }
        


    pub struct Node {
        id: Uuid,
        role: Role,
        address: String,
        transaction_buffer: Option<Vec<Transaction>>,
        wallet: Wallet,
        chain: Chain,
        neighbours: HashMap<Uuid, Neighbour>,
        new_neighbours: Vec<Neighbour>,
        initialized: bool,
        trackers: Option<Vec<String>>,
        receiver: Receiver,
    }

    impl Node {

        pub fn new(role: Role, address: String, trackers: Option<Vec<String>>, receiver: Receiver) -> 
                Result<Self, WrongRoleError> {
            if trackers == None && role != Role::Tracker {
                return Err(WrongRoleError::NotTracker);
            }
            let mut transaction_buffer = None;
            if role == Role::Miner {
                transaction_buffer = Some(vec![]);
            }
            Ok(Node {
                id: Uuid::new_v4(),
                role, 
                address, 
                transaction_buffer, 
                wallet: Wallet::new(),
                chain: Chain::new(),
                neighbours: HashMap::new(),
                new_neighbours: vec![],
                initialized: false,
                trackers,
                receiver,
            })
        }

        async fn receive_transaction(&self) -> Result<Transaction, TransactionRecvError> {
            let str_transaction = match self.receiver.recv().await?;
            match str_transaction.try_from() {
                Ok(transaction) => Ok(transaction),
                Err(e) => panic!("Transaction err: {}", e);
            }
        }



        pub fn queue_transaction(&self, transaction: Transaction) {
            self.transaction_buffer.push(transaction);
        }

        pub fn get_n_neighbours(&self) -> usize {
            self.neighbours.len()
        }

        pub async fn init_node(&mut self) -> IOResult<()> {
            loop {
                self.initialized = true; 
                println!("{} started listening", self.id);
                self.listen().await?;
                println!("{} finished listening", self.id);
                self.gossip().await;
            }
            Ok(())
        }

        pub async fn enter_and_init_node(&mut self) -> IOResult<()> {
            self.enter_network().await.unwrap();
            self.init_node().await?;
            Ok(())
        }
                

        pub async fn enter_network(&mut self) -> Result<(), EnterAttemptError> {
            println!("{} entering network", self.id);
            match &self.trackers {
                Some(ts) => {
                    for tracker in ts {
                        match gossip::greet(
                            self.address.clone(), 
                            self.id.clone(), 
                            self.role, 
                            tracker.as_str()
                        ).await {
                            Ok(neighbour) => {
                                self.neighbours.insert(neighbour.id.clone(), neighbour);
                                self.initialized = true;
                            },
                            Err(_) => continue,
                        }
                    }
                    if ! self.initialized {
                        return Err(EnterAttemptError::NoListeners)
                    }
                    println!("number of neighbours in sender: {}", self.neighbours.len());
                    Ok(())
                },
                None => Err(EnterAttemptError::NoTrackers)
            }
        }

        pub async fn leave_network(&self) {
            for neighbour in &self.neighbours {
                let _ = gossip::farewell(self.address.clone(), neighbour.1.address.clone()).await;
            }
        }         


        pub async fn submit_transaction(&self, transaction: Transaction) {
            let _ = self.neighbours
                .iter()
                .filter(|neighbour| { neighbour.1.role == Role::Miner })
                .map(|miner| async { gossip::send_transaction(self.address.clone(), 
                        miner.1.address.clone(), 
                        transaction.clone()).await 
                })
                .collect::<Vec<_>>();
        }

        pub async fn update_chain(&self) -> Result<Chain, UpdateChainError> {
            let mut cursor = self.neighbours.iter();
            let mut cur_neighbour = cursor.next();
            while cur_neighbour != None {
                match gossip::poll_chain(self.address.clone(), cur_neighbour.unwrap().1).await {
                    Ok(chain) => return Ok(chain),
                    Err(_) => cur_neighbour = cursor.next(),
                }
            }
            Err(UpdateChainError::NoListeners)
        }

        pub async fn gossip(&mut self) {
            gossip::wait_gossip_interval().await;
            println!("starting gossip");
            let random_neighbours = self.get_random_neighbours();
            println!("N random neighbours: {}", &random_neighbours.len()); 
            for neighbour in random_neighbours {
                if self.chain.get_len() > 0 {
                    let _ = gossip::send_chain(self.address.clone(), 
                        neighbour.address.clone(), 
                        self.chain.clone()).await;
                }
                if self.new_neighbours.len() > 0 {
                    let _ = gossip::send_new_neighbours(self.address.clone(), 
                        neighbour.address.clone(), 
                        self.new_neighbours.clone()).await;
                }
                self.new_neighbours = vec![];
            }
        }


        fn get_random_neighbours(&self) -> Vec<Neighbour> {
            let mut neighbours = vec![];
            let mut rng = rand::thread_rng();
            let float_n = self.neighbours.len() as f64;
            let n: usize = float_n.sqrt().floor() as usize;
            for _ in 0..n {
                let random: usize = rng.gen_range(0..self.neighbours.len());
                let key = self.neighbours 
                    .keys()
                    .skip(random)
                    .next()
                    .unwrap();
                neighbours.push(self.neighbours.get(key).unwrap().clone());
            }
            neighbours
        }

        fn check_chain(&mut self, chain: Chain) {
            if chain.len() > self.chain.len() {
                self.chain = chain;
            }
        }


        pub async fn listen(&mut self) -> IOResult<()> {
            if self.initialized {
                let (protocol, sender, buffer) = gossip::listen_to_gossip(self.address.clone()).await?;
                let res = match protocol { 
                    protocol::GREET => self.present_id(sender, buffer).await?, 
                    protocol::FAREWELL => self.remove_neigbour(sender).await?,
                    protocol::NEIGHBOUR => self.add_neighbour(buffer).await?,
                    protocol::TRANSACTION => self.add_transaction(buffer).await?,
                    protocol::CHAIN => self.get_chain(buffer).await?,
                    protocol::POLLCHAIN => self.share_chain().await?, 
                    _ => None//TODO: Ignore with an error

                };
                match res {
                    Some(mut ptr) => match &mut ptr.as_chain() {
                        Some(chain) => self.check_chain(chain.clone()), // self.chain = chain.clone(), 
                        None => match &ptr.as_transaction() {
                            Some(transaction) => match &mut self.transaction_buffer {
                                Some(ref mut buffer) => buffer.push((**transaction).clone()),
                                None => (),
                            }
                            None => (),
                        },
                    },
                    None => (),
                }
                match self.receiver.receive_transaction() {
                    Ok(transaction) => self.send_transaction(transaction),
                    Err(e) => (),
                }
               
            }
            Ok(())
        }


        fn sanitize(string: String) -> String {
            let mut new_string = String::new();
            let accepted_chars = " \",:.-{}[]_";
            for i in string.chars() {
                if i.is_alphanumeric() || accepted_chars.contains(i) {
                    new_string.push(i);
                } else {
                    break;
                }
            }
            new_string
        }

        pub async fn present_id(&mut self, sender: String, mut buffer: Vec<u8>) -> IOResult<Option<Box<dyn Reply>>>{ 
            buffer.remove(0);
            let str_buffer = str::from_utf8(&buffer)
                .expect("Malformed request to enter network -- Unable to parse").trim();
            let cleared = Node::sanitize(str_buffer.to_string());
            let neighbour: Neighbour = serde_json::from_str(&cleared) 
                .expect("Malformed neighbour string -- Unable to create neighbour from enter network request");
            let hash_neighbour = neighbour.clone();
            self.neighbours.entry(hash_neighbour.id).or_insert(hash_neighbour);
            self.new_neighbours.push(neighbour);
            gossip::send_id(self.address.clone(), self.id.clone(), sender).await;
            println!("Listener n neighbours: {}", self.neighbours.len());
            Ok(None)
        }

        pub async fn remove_neigbour(&mut self, sender: String) -> 
                IOResult<Option<Box<dyn Reply>>>{ 
            self.neighbours.retain(|_, v| v.address != sender);
            Ok(None)
        }

        pub async fn add_neighbour(&mut self, mut buffer: Vec<u8>)  -> 
                IOResult<Option<Box<dyn Reply>>>{ 
            buffer.remove(0);
            let str_buffer = str::from_utf8(&buffer)
                .expect("Malformed request to add neighbour -- Unable to parse");
            let neighbour: Neighbour = serde_json::from_str(str_buffer) 
                .expect("Malformed neighbour string -- Unable to create neighbour from request");
            let hash_neighbour = neighbour.clone();
            self.neighbours.entry(hash_neighbour.id).or_insert(hash_neighbour);
            self.new_neighbours.push(neighbour);
            Ok(None)
        }

        pub async fn add_transaction(&self, mut buffer: Vec<u8>) -> 
                IOResult<Option<Box<dyn Reply>>>{ 
            if self.role != Role::Miner {
                return Ok(None) //TODO: we need to return some kind of error
            }            
            buffer.remove(0);
            let str_buffer = str::from_utf8(&buffer)
                .expect("Malformed request to add transaction -- Unable to parse");
            let transaction = Transaction::try_from(str_buffer.to_string())
                .expect("Malformed transaction string -- Unable to create transaction from request");
            Ok(Some(Box::new(transaction)))
        }
            
        pub async fn get_chain(&mut self, mut buffer: Vec<u8>) -> 
                IOResult<Option<Box<dyn Reply>>>{ 
            buffer.remove(0);
            let str_buffer = str::from_utf8(&buffer)
                .expect("Malformed request to check chain -- Unable to parse");
            let cleared = Node::sanitize(str_buffer.to_string());
            let chain: Chain = serde_json::from_str(&cleared)
                .expect("Malformed chain string -- Unable to create chain from request");
            Ok(Some(Box::new(chain)))
        }

        pub async fn share_chain(&self) -> IOResult<Option<Box<dyn Reply>>>{ Ok(None)}
    }
}

    
