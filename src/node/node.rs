pub mod node {

    use crate::{
        Wallet,
        Chain,
        Transaction,
        Miner,
        chain::block::block::block::Block,
        node::{
            neighbour::neighbour::{Neighbour, Role},
            gossip::gossip,
            gossip::gossip::GossipError,
            protocol::protocol,
            receiver::receiver::Receiver,
            reply::reply::Reply,
            theme::theme::{self, Theme},
        },
        transaction::transaction::transaction::TransactionFromBase64Error,
    };
    use tokio::sync::mpsc::error::TryRecvError;

    use std::{
        sync::{Arc, Mutex},
        collections::HashMap,
        io::{Result as IOResult, Error as IOError},
        str,
    };

    use thiserror::Error;
    use rand::prelude::*;
    use uuid::{self, Uuid};
    use tracing::{debug, info};

    const DEFAULT_ADDRESS: &str = "127.0.0.1";

    // -------------------------------
    // Error Definitions
    // -------------------------------
    
    #[derive(Error, Debug)]
    pub enum EnterAttemptError {
        #[error("Failed to enter network - No trackers listening.")]
        NoListeners,
        #[error("Failed to enter network - No trackers available.")]
        NoTrackers,
    }

    #[derive(Error, Debug)]
    pub enum UpdateChainError {
        #[error("Failed to update chain - No neighbours listening.")]
        NoListeners,
    }

    #[derive(Error, Debug, derive_more::From)]
    pub enum WrongRoleError {
        #[error("That operation requires a Node with Role Miner.")]
        NotMiner,
        #[error("That operation requires a Node with Role Tracker.")]
        NotTracker,
    }

    //#[derive(Error, Debug, derive_more::From, derive_more::Display)]
    #[derive(Error, Debug, derive_more::From)]
    pub enum ListenError {
        #[error(transparent)]
        WrongRoleError(WrongRoleError),
        #[error(transparent)]
        IOError(IOError),
    }

    #[derive(Error, Debug, derive_more::From)]
    pub enum TransactionRecvError {
        #[error(transparent)]
        TryRecvError(TryRecvError),
        #[error(transparent)]
        TransactionFromBase64Error(TransactionFromBase64Error),
    }
    
    #[derive(Error, Debug, derive_more::From)]
    pub enum NodeLoopError {
        #[error(transparent)]
        EnterAttemptError(EnterAttemptError),
        #[error(transparent)]
        GossipError(GossipError),
    }


    pub struct MiningDigest {
        block: Block,
        nonce: u64,
    }

    // -------------------------------
    // Node Structure Definition
    // -------------------------------
    
    pub struct Node {
        id: Uuid,
        role: Role,
        address: Arc<str>,
        transaction_buffer: Option<Vec<Transaction>>,
        wallet: Wallet,
        chain: Chain,
        neighbours: HashMap<Uuid, Neighbour>,
        new_neighbours: Vec<Neighbour>,
        initialized: bool,
        trackers: Option<Vec<String>>,
        receiver: Arc<Mutex<Receiver>>,
        miner: Option<Arc<Mutex<Miner>>>,
    }

    // -------------------------------
    // Node Implementation
    // -------------------------------

    impl Node {
        /// Creates a new `Node` instance.
        pub fn new(role: Role, address: String, trackers: Option<Vec<String>>, receiver: Receiver) -> Self {
            let mut transaction_buffer = None;
            let mut miner = None;

            if role == Role::Miner {
                transaction_buffer = Some(vec![]);

                miner = Some(Arc::new(Mutex::new(Miner::new(1, "miner".to_string())))); //TODO: generate id and name
            }
            Node {
                id: Uuid::new_v4(),
                role,
                address: address.into(),
                transaction_buffer,
                wallet: Wallet::new(),
                chain: Chain::new(),
                neighbours: HashMap::new(),
                new_neighbours: vec![],
                initialized: false,
                trackers,
                receiver: Arc::new(Mutex::new(receiver)),
                miner,
            }
        }

        pub fn get_address(&self) -> Arc<str> {
            self.address.clone()
        }


        /// Queues a transaction into the node's transaction buffer.
        pub fn queue_transaction(&mut self, transaction: Transaction) {
            if let Some(buffer) = &mut self.transaction_buffer {
                buffer.push(transaction);
            }
        }

        /// Returns the number of neighbors this node has.
        pub fn get_n_neighbours(&self) -> usize {
            self.neighbours.len()
        }


        // -------------------------------
        // Network Operations
        // -------------------------------

        /// Main node loop that listens and processes various activities in the network.
        pub async fn node_loop(&mut self) -> Result<(), GossipError> {
            debug!("{} starting node loop.", self.id);
            let mut theme = Theme::Chain;
            loop {
                let theme_protocol = (theme.to_protocol() + 1) % theme::N_THEMES; //TODO: Fix this.
                                                                                  //Jesus Christ.
                theme = Theme::from_protocol(theme_protocol).unwrap();
                self.initialized = true;
                let chain = self.chain.clone();
                let chain_gossip = self.chain.clone();
                let role = self.role.clone();
                let miner_clone = self.miner.as_mut().unwrap().clone();
                let receiver_clone = self.receiver.clone();
                let neighbours = self.neighbours.clone();
                let address = self.address.clone();
                let address_gossip = self.address.clone();
                let random_neighbours = self.get_random_neighbours();
                let new_neighbours = self.new_neighbours.clone();
                tokio::join!(
                    self.listen_to_peers(),
                    gossip(address_gossip, chain_gossip, random_neighbours, new_neighbours, theme.clone()),
                    listen_to_transactions(receiver_clone, neighbours, address),
                    mine(role, miner_clone, chain), //TODO: Should have to unwrap
                );
            }
        }


        /// Enters the network by contacting trackers and starts the node loop.
        pub async fn enter_and_node_loop(&mut self) -> Result<(), NodeLoopError> {
            self.enter_network().await?;
            self.node_loop().await?;
            Ok(())
        }

        /// Contacts trackers and attempts to join the network.
        pub async fn enter_network(&mut self) -> Result<(), EnterAttemptError> {
            if let Some(trackers) = &self.trackers {
                for tracker in trackers {
                    match gossip::greet(self.address.clone(), self.id.clone(), self.role, tracker).await {
                        Ok(neighbour) => {
                            self.neighbours.insert(neighbour.id.clone(), neighbour.clone());
                            self.new_neighbours.push(neighbour);
                            self.initialized = true;
                        }
                        Err(_) => {
                            debug!("Node {} failed to greet tracker", self.id);
                            continue;
                        }
                    }
                }
                if !self.initialized {
                    return Err(EnterAttemptError::NoListeners);
                }
                Ok(())
            } else {
                Err(EnterAttemptError::NoTrackers)
            }
        }

        /// Leaves the network by sending farewell messages to all neighbours.
        pub async fn leave_network(&self) {
            for neighbour in &self.neighbours {
                let _ = gossip::farewell(self.address.clone(), neighbour.1.address.clone()).await;
            }
        }

        // -------------------------------
        // Transaction and Chain Operations
        // -------------------------------

        pub async fn update_chain(&self) -> Result<Chain, UpdateChainError> {
            let mut cursor = self.neighbours.iter();
            while let Some((_id, neighbour)) = cursor.next() {
                match gossip::poll_chain(self.address.clone(), neighbour).await {
                    Ok(chain) => return Ok(chain),
                    Err(_) => continue,
                }
            }
            Err(UpdateChainError::NoListeners)
        }

        // -------------------------------
        // Gossip and Neighbor Management
        // -------------------------------

        fn get_random_neighbours(&self) -> Vec<Neighbour> {
            let mut neighbours = vec![];
            let mut rng = rand::thread_rng();
            let n = (self.neighbours.len() as f64).sqrt().floor() as usize;
            for _ in 0..n {
                let random_index = rng.gen_range(0..self.neighbours.len());
                let random_key = self.neighbours.keys().nth(random_index).unwrap();
                neighbours.push(self.neighbours.get(random_key).unwrap().clone());
            }
            neighbours
        }

        // -------------------------------
        // Listening and Chain Validation
        // -------------------------------

        /// Listens for incoming messages and processes them based on the protocol.
        pub async fn listen_to_peers(&mut self) -> Result<(), GossipError> {
            debug!("{} listening", self.id);
            if self.initialized {
                let (protocol, sender, buffer) = match gossip::listen_to_gossip(self.address.clone()).await {
                    Ok(res) => match res {
                        Some((protocol, sender, buffer)) => (protocol, sender, buffer),
                        None => return Ok(()),
                    }
                    Err(e) => return Ok(()),
                };
                debug!("Received protocol: {}", &protocol);

                let res = match protocol {
                    protocol::GREET => self.present_id(sender, buffer).await?,
                    protocol::FAREWELL => self.remove_neighbour(sender).await?,
                    protocol::NEIGHBOUR => self.add_neighbour(buffer).await?,
                    protocol::TRANSACTION => self.add_transaction(buffer).await?,
                    protocol::CHAIN => self.get_chain(buffer).await?,
                    protocol::POLLCHAIN => self.share_chain().await?,
                    _ => None, // Ignore unrecognized protocol with no error
                };

                if let Some(mut ptr) = res {
                    if let Some(chain) = ptr.as_chain() {
                        self.check_chain(chain.clone());
                    } else if let Some(transaction) = ptr.as_transaction() {
                        if let Some(miner) = &mut self.miner {
                            miner.lock().unwrap().push_transaction(transaction.clone());
                        }
                    }
                }
            }
            Ok(())
        }

        /// Updates the node's chain if the received chain is longer.
        fn check_chain(&mut self, chain: Chain) {
            if chain.len() > self.chain.len() {
                self.chain = chain;
            }
        }

        // -------------------------------
        // Neighbor Management
        // -------------------------------

        /// Handles the presentation of this node's ID when contacted by a neighbour.
        pub async fn present_id(&mut self, sender: String, mut buffer: Vec<u8>) -> IOResult<Option<Box<dyn Reply>>> {
            buffer.remove(0);
            let str_buffer = str::from_utf8(&buffer)
                .expect("Malformed request to enter network -- Unable to parse")
                .trim();
            let cleared = Node::sanitize(str_buffer.to_string());
            let neighbour: Neighbour = serde_json::from_str(&cleared)
                .expect("Malformed neighbour string -- Unable to create neighbour from enter network request");

            let hash_neighbour = neighbour.clone();
            self.neighbours.entry(hash_neighbour.id).or_insert(hash_neighbour);
            self.new_neighbours.push(neighbour);

            // Sending ID back to the sender
            gossip::send_id(self.address.clone(), self.id.clone(), sender).await;

            Ok(None)
        }

        /// Removes a neighbour from the list based on the provided sender address.
        pub async fn remove_neighbour(&mut self, sender: String) -> IOResult<Option<Box<dyn Reply>>> {
            self.neighbours.retain(|_, v| v.address != sender);
            Ok(None)
        }

        /// Adds a neighbour to this node's network from the provided buffer.
        pub async fn add_neighbour(&mut self, mut buffer: Vec<u8>) -> IOResult<Option<Box<dyn Reply>>> {
            buffer.remove(0);

            let str_buffer = str::from_utf8(&buffer)
                .expect("Malformed request to add neighbour -- Unable to parse");
            debug!("Received neighbour: {}", str_buffer);

            let cleared = Node::sanitize(str_buffer.to_string());
            let neighbour: Neighbour = serde_json::from_str(&cleared)
                .expect("Malformed neighbour string -- Unable to create neighbour from request");

            let hash_neighbour = neighbour.clone();
            self.neighbours.entry(hash_neighbour.id).or_insert(hash_neighbour);
            self.new_neighbours.push(neighbour);

            Ok(None)
        }

        // -------------------------------
        // Transaction Handling
        // -------------------------------

        /// Adds a transaction from the buffer, if this node is a miner.
        pub async fn add_transaction(&self, mut buffer: Vec<u8>) -> IOResult<Option<Box<dyn Reply>>> {
            if self.role != Role::Miner {
                return Ok(None); // We can enhance this later to return an error
            }

            buffer.remove(0);
            let str_buffer = str::from_utf8(&buffer)
                .expect("Malformed request to add transaction -- Unable to parse");

            let transaction = Transaction::try_from(str_buffer.to_string())
                .expect("Malformed transaction string -- Unable to create transaction from request");

            Ok(Some(Box::new(transaction)))
        }

        // -------------------------------
        // Chain Management
        // -------------------------------

        /// Receives a chain from the buffer and returns it.
        pub async fn get_chain(&mut self, mut buffer: Vec<u8>) -> IOResult<Option<Box<dyn Reply>>> {
            buffer.remove(0);
            let str_buffer = str::from_utf8(&buffer)
                .expect("Malformed request to check chain -- Unable to parse");

            let cleared = Node::sanitize(str_buffer.to_string());
            let chain: Chain = serde_json::from_str(&cleared)
                .expect("Malformed chain string -- Unable to create chain from request");

            Ok(Some(Box::new(chain)))
        }

        /// Shares the current chain with any requesting neighbour.
        pub async fn share_chain(&self) -> IOResult<Option<Box<dyn Reply>>> {
            Ok(None)
        }

        // -------------------------------
        // Utility Methods
        // -------------------------------

        /// Sanitizes a string by only allowing alphanumeric characters and a few special characters.
        fn sanitize(string: String) -> String {
            let accepted_chars = " \",;:.-{}[]_=/+";
            string.chars()
                .take_while(|c| c.is_alphanumeric() || accepted_chars.contains(*c))
                .collect()
        }
    }

    /// Handles mining process if the node is a miner.
    async fn mine(role: Role, miner: Arc<Mutex<Miner>>, chain: Chain) -> Option<MiningDigest> {
        let mut inner_miner = miner.lock().unwrap();
        if role == Role::Miner {
            inner_miner.set_chain_meta(
                chain.get_len(),
                chain.difficulty,
                chain.get_blocks(),
            );
            let (block, nonce) = inner_miner.mine(
                chain.get_last_block(),
            ).unwrap(); //TODO: Handle mining abort if the chain gets updated for this index
            info!("Mined block: {}", block);
            return Some(MiningDigest {
                block,
                nonce,
            });
            
            //let _ = self.chain.add_block(new_block, new_nonce);
        }
        None
    }

   /// Submits a transaction to all miner neighbours.
    pub async fn submit_transaction(
        transaction: Transaction, 
        neighbours: HashMap<Uuid, Neighbour>, 
        address: Arc<str>
    ) {
        let _ = neighbours
            .iter()
            .filter(|neighbour| neighbour.1.role == Role::Miner) // Filters only miners
            .map(|miner| async {
                gossip::send_transaction(address.clone(), miner.1.address.clone(), transaction.clone()).await
            })
            .collect::<Vec<_>>();
    }

        /// Updates the chain by polling neighbours for the latest chain.
    /// Listens for and processes incoming transactions.
    async fn listen_to_transactions(
        receiver: Arc<Mutex<Receiver>>, 
        neighbours: HashMap<Uuid, Neighbour>,
        address: Arc<str>,
    ) {
        match receive_transaction(receiver).await {
            Ok(transaction) => {
                debug!("Transaction being received: {}", transaction);
                submit_transaction(transaction, neighbours, address).await;
            },
            Err(_e) => {
                // Handle error or log it.
            },
        }
    }
        /// Handles the gossiping process with random neighbours, based on the provided theme.
    pub async fn gossip(
        address: Arc<str>, 
        chain: Chain, 
        random_neighbours: Vec<Neighbour>, 
        new_neighbours: Vec<Neighbour>,
        theme: Theme
    ) {
        gossip::wait_gossip_interval().await;
        for neighbour in random_neighbours {
            match theme {
                Theme::Chain => {
                    if chain.get_len() > 0 {
                        let _ = gossip::send_chain(
                            address.clone(),
                            neighbour.address.clone(),
                            chain.clone() //TODO: Shouldn't have to clone eveyt time.
                        ).await;
                    }
                },
                Theme::NewNeighbours => {
                    if !new_neighbours.is_empty() {
                        let _ = gossip::send_new_neighbours(
                            neighbour.id.clone(),
                            neighbour.address.clone(),
                            address.clone(),
                            new_neighbours.clone()
                        ).await;
                    }
                },
            }
        }
    }

        /// Returns a random subset of neighbours for gossiping.

   /// Receives a transaction.
   async fn receive_transaction(receiver: Arc<Mutex<Receiver>>) -> Result<Transaction, TransactionRecvError> {
       let str_transaction = receiver.lock().unwrap().recv().await?;
       match Transaction::try_from(str_transaction.to_owned()) {
           Ok(transaction) => Ok(transaction),
           Err(e) => Err(TransactionRecvError::TransactionFromBase64Error(e)),  // Consider handling this more gracefully
       }
   }
}
