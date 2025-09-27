use crate::node::{
    gossip::{self, GossipError},
    neighbour::{Neighbour, Role},
    protocol,
    receiver::Receiver,
    reply::{Reply, ReplySign},
    theme::Theme,
};
use chain::chain::Chain;
use chain::miner::miner::Miner;
use rand::prelude::*;
use std::{
    collections::HashMap,
    io::{Error as IOError, Result as IOResult},
    str,
    sync::{self, Arc},
};
use thiserror::Error;
use tokio::sync::{
    broadcast,
    mpsc::{self, error::TryRecvError, Sender},
    Mutex,
};
use tracing::{debug, info};
use uuid::{self, Uuid};
use wallet::transaction::block_entry_common::EntryDecodeError;
use wallet::transaction::{block_entry_common::BlockEntry, transaction::Transaction};
#[allow(dead_code)]
const DEFAULT_ADDRESS: &str = "127.0.0.1";

// ------------------------------- // Error Definitions // -------------------------------
/// Errors that can occur when attempting to enter the network
#[derive(Error, Debug)]
pub enum EnterAttemptError {
    #[error("Failed to enter network - No trackers listening.")]
    /// No trackers listening.
    NoListeners,
    /// No trackers available.
    #[error("Failed to enter network - No trackers available.")]
    NoTrackers,
}

/// Errors that can occur when attempting to update the chain
#[derive(Error, Debug)]
pub enum UpdateChainError {
    #[error("Failed to update chain - No neighbours listening.")]
    /// No neighbouts listening.
    NoListeners,
}

/// Errors that occur when attempting operations with wrong node role
#[derive(Error, Debug, derive_more::From)]
pub enum WrongRoleError {
    #[error("That operation requires a Node with Role Miner.")]
    /// This node must be a `Role::Miner`.
    NotMiner,
    #[error("That operation requires a Node with Role Tracker.")]
    /// This node must be a `Role::Tracker`.
    NotTracker,
}

/// Errors that can occur during listening operations
#[derive(Error, Debug, derive_more::From)]
pub enum ListenError {
    #[error(transparent)]
    /// This node must have had another `Role`.
    WrongRoleError(WrongRoleError),
    #[error(transparent)]
    /// This node must have had another `IOError`.
    IOError(IOError),
}

/// Errors that can occur when receiving transactions
#[derive(Error, Debug, derive_more::From)]
pub enum TransactionRecvError {
    #[error(transparent)]
    /// Failed to receive `Transaction`.
    TryRecvError(TryRecvError),
    #[error(transparent)]
    /// Failed to decode `BlockEntry`.
    TransactionFromBase64Error(EntryDecodeError),
}

/// Errors that can occur in the main node loop
#[derive(Error, Debug, derive_more::From)]
pub enum NodeLoopError {
    #[error(transparent)]
    /// Failed to enter the network.
    EnterAttemptError(EnterAttemptError),
    #[error(transparent)]
    /// Gossip error.
    GossipError(GossipError),
}
// ------------------------------- // Node Structure Definition // -------------------------------
/// Represents a node in a peer-to-peer blockchain network
///
/// A Node can be either a regular peer or a miner, and maintains
/// connections to other nodes in the network.
#[allow(unused_variables)]
#[derive(Clone)]
pub struct Node {
    id: Uuid,
    role: Role,
    address: Arc<str>,
    transaction_buffer: Option<Vec<Transaction>>,
    chain: Chain,
    neighbours: HashMap<Uuid, Neighbour>,
    new_neighbours: Vec<Neighbour>,
    initialized: bool,
    trackers: Option<Vec<String>>,
    transaction_receiver: Arc<Mutex<Receiver>>,
    miner: Option<Arc<Mutex<Arc<sync::Mutex<Miner>>>>>, // Inner arc for blocking threads.
    log_sender: Option<mpsc::Sender<String>>,
}
// ------------------------------- // Node Implementation // -------------------------------
impl Node {
    /// Creates a new Node instance
    ///
    /// # Arguments
    /// * `role` - The role of this node (Miner or Tracker)
    /// * `address` - Network address of this node
    /// * `trackers` - List of tracker addresses for network entry
    /// * `receiver` - Channel receiver for transactions
    /// * `log_sender` - Optional channel for sending log messages
    #[must_use]
    pub fn new(
        role: Role,
        address: String,
        trackers: Option<Vec<String>>,
        receiver: Receiver,
        log_sender: Option<Sender<String>>,
    ) -> Self {
        let transaction_buffer = None;
        let miner = None;
        Node {
            id: Uuid::new_v4(),
            role,
            address: address.into(),
            transaction_buffer,
            chain: Chain::new(),
            neighbours: HashMap::new(),
            new_neighbours: vec![],
            initialized: false,
            trackers,
            transaction_receiver: Arc::new(Mutex::new(receiver)),
            miner,
            log_sender,
        }
    }

    /// Sends a log message if logging is enabled
    ///
    /// # Arguments
    /// * `log_msg` - The message to log
    pub async fn update_log(&self, log_msg: impl Into<String>) {
        if let Some(log_sender) = &self.log_sender {
            let _ = log_sender.send(log_msg.into()).await;
        }
    }

    /// Gets the network address of this node
    ///
    /// # Returns
    /// The node's address as an Arc<str>
    #[must_use]
    pub fn get_address(&self) -> Arc<str> {
        self.address.clone()
    }

    /// Queues a transaction into the node's transaction buffer
    ///
    /// # Arguments
    /// * `transaction` - The transaction to queue
    pub fn queue_transaction(&mut self, transaction: Transaction) {
        if let Some(buffer) = &mut self.transaction_buffer {
            buffer.push(transaction);
        }
    }

    /// Returns the number of neighbors this node has
    #[must_use]
    pub fn get_n_neighbours(&self) -> usize {
        self.neighbours.len()
    }

    // ------------------------------- // Network Operations // -------------------------------
    /// Main node loop that listens and processes various activities in the network
    ///
    /// This is the primary processing loop that handles:
    /// - Spreading updates to neighbors
    /// - Listening for transactions
    /// - Mining blocks (if miner)
    pub async fn node_loop(&mut self) -> Result<(), GossipError> {
        debug!("{} starting node loop.", self.id);
        let mut theme = Theme::default();
        let (mining_sender, mut mining_receiver) = mpsc::channel(1024);
        let mining_sender: &'static Sender<Chain> = Box::leak(Box::new(mining_sender));
        let (sender, mut receiver) = broadcast::channel(16);
        loop {
            //Task 1: Spread update to neighbours.
            println!("{} spreading updates.", self.id);
            theme.next();
            let chain_gossip = self.chain.clone();
            let address_gossip = self.address.clone();
            let random_neighbours = self.get_random_neighbours();
            let new_neighbours = self.new_neighbours.clone();
            let gossip_receiver = sender.subscribe();
            tokio::spawn(gossip(
                address_gossip,
                chain_gossip,
                random_neighbours,
                new_neighbours,
                theme,
                gossip_receiver,
            ));
            //Task 2: Add local transactions to local miner or send them to remote miners.
            println!("{} listening to transactions (miner).", self.id);
            let receiver_clone = self.transaction_receiver.clone();
            let neighbours = self.neighbours.clone();
            let address = self.address.clone();
            let miner_transaction_handle = self.miner.clone();
            let log_sender = self.log_sender.clone();
            tokio::spawn(listen_to_transactions(
                receiver_clone,
                neighbours,
                address,
                miner_transaction_handle,
                log_sender,
            ));
            //Task 3: If this is miner, try to mine a block.
            if self.role == Role::Miner {
                if self.miner.is_none() {
                    let chain = self.chain.clone();
                    self.transaction_buffer = Some(vec![]);
                    self.miner = Some(Arc::new(Mutex::new(Arc::new(sync::Mutex::new(
                        Miner::new(1, "miner".to_string(), chain),
                    ))))); //TODO: generate id and name
                }
                let miner_worker_handle = self.miner.clone();
                println!("trying to mine...");
                tokio::spawn(try_mine(
                    self.id,
                    miner_worker_handle,
                    self.chain.clone(),
                    mining_sender,
                ));
            }
            //Task 3: Listen to possible updates the peers might have shared.
            let _ = self
                .listen_to_peers(&sender, &mut mining_receiver, &mut receiver)
                .await;
        }
    }

    /// Enters the network and starts the main node loop
    ///
    /// # Returns
    /// Result indicating success or failure to enter network
    pub async fn enter_and_node_loop(&mut self) -> Result<(), NodeLoopError> {
        self.enter_network().await?;
        self.node_loop().await?;
        Ok(())
    }

    /// Contacts trackers and attempts to join the network
    ///
    /// # Returns
    /// Result indicating success or failure to enter network
    pub async fn enter_network(&mut self) -> Result<(), EnterAttemptError> {
        if let Some(trackers) = &self.trackers {
            for tracker in trackers {
                match gossip::greet(self.address.clone(), self.id, self.role, tracker).await {
                    Ok(neighbour) => {
                        self.neighbours.insert(neighbour.id, neighbour.clone());
                        self.new_neighbours.push(neighbour);
                        self.initialized = true;
                        self.update_log("NeighbourAdded").await;
                    }
                    Err(_) => {
                        println!("Node {} failed to greet tracker", self.id);
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

    /// Leaves the network by sending farewell messages to all neighbours
    pub async fn leave_network(&self) {
        for neighbour in &self.neighbours {
            let _ = gossip::farewell(self.address.clone(), neighbour.1.address.clone()).await;
        }
    }

    // ------------------------------- // Transaction and Chain Operations // -------------------------------
    /// Updates the node's chain by polling neighbors
    ///
    /// # Returns
    /// Updated chain if successful, or error if no neighbors available
    pub async fn update_chain(&self) -> Result<Chain, UpdateChainError> {
        let cursor = self.neighbours.iter();
        for (_id, neighbour) in cursor {
            if let Ok(chain) = gossip::poll_chain(self.address.clone(), neighbour).await {
                return Ok(chain);
            }
        }
        Err(UpdateChainError::NoListeners)
    }

    // ------------------------------- // Gossip and Neighbor Management // -------------------------------
    /// Gets a random subset of neighbors for gossip purposes
    ///
    /// The number of neighbors returned is approximately the square root
    /// of the total number of neighbors.
    ///
    /// # Returns
    /// Vector of randomly selected neighbors
    #[allow(
        clippy::unwrap_used,
        clippy::cast_precision_loss,
        clippy::cast_sign_loss,
        clippy::cast_possible_truncation
    )]
    // Random index guaranteed to be in range.
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

    // ------------------------------- // Listening and Chain Validation // -------------------------------
    /// Listens for incoming messages and processes them based on the protocol
    ///
    /// Handles various message types including:
    /// - Greetings from new nodes
    /// - Farewell messages
    /// - Neighbor information
    /// - Transactions
    /// - Chain updates
    /// - Chain polls
    ///
    /// # Arguments
    /// * `sender` - Broadcast channel for chain updates
    /// * `mining_receiver` - Channel for receiving mined chains
    /// * `receiver` - Broadcast receiver for chain updates
    ///
    /// # Returns
    /// Result indicating success or gossip error
    pub async fn listen_to_peers(
        &mut self,
        sender: &broadcast::Sender<Chain>,
        mining_receiver: &mut mpsc::Receiver<Chain>,
        receiver: &mut broadcast::Receiver<Chain>,
    ) -> Result<(), GossipError> {
        loop {
            self.check_mined_chain_and_broadcast(sender, mining_receiver);
            self.check_peer_mined_chains(receiver);
            println!("{} updating chain len.", self.id);
            self.update_log(self.chain.len().to_string()).await;
            let gossip_reply = match gossip::listen_to_gossip(self.address.clone()).await {
                Ok(res) => match res {
                    Some(gossip_reply) => {
                        if gossip_reply.protocol == protocol::GREET {
                            self.update_log("GossipGreetReply").await;
                        }
                        gossip_reply
                    }
                    None => return Ok(()),
                },
                Err(_) => return Ok(()),
            };
            {
                let res = match gossip_reply.protocol {
                    protocol::GREET => {
                        self.present_id(gossip_reply.sender, gossip_reply.buffer)
                            .await?
                    }
                    protocol::FAREWELL => self.remove_neighbour(gossip_reply.sender)?,
                    protocol::NEIGHBOUR => self.add_neighbour(gossip_reply.buffer)?,
                    protocol::TRANSACTION => self.add_block_entry(gossip_reply.buffer)?,
                    protocol::CHAIN => self.get_chain(gossip_reply.buffer)?,
                    protocol::POLLCHAIN => self.share_chain()?,
                    _ => None,
                    // Ignore unrecognized protocol with no errors.
                };
                if let Some(mut ptr) = res {
                    if let Some(chain) = ptr.as_chain() {
                        self.check_remote_chain_and_broadcast(chain.clone(), sender);
                    } else if let Some(entry) = ptr.as_sign() {
                        if let Some(miner) = self.miner.as_mut() {
                            push_transaction(miner, entry.clone_box()).await;
                        }
                    }
                }
            }
        }
    }

    /// Checks for newly mined chains and broadcasts them if valid
    ///
    /// # Arguments
    /// * `sender` - Broadcast channel for chain updates
    /// * `mining_receiver` - Channel for receiving mined chains
    fn check_mined_chain_and_broadcast(
        &mut self,
        sender: &broadcast::Sender<Chain>,
        mining_receiver: &mut mpsc::Receiver<Chain>,
    ) {
        match mining_receiver.try_recv() {
            Ok(mined_chain) => {
                if mined_chain > self.chain {
                    self.chain = mined_chain;
                    let _ = sender.send(self.chain.clone());
                }
            }
            Err(TryRecvError::Empty | TryRecvError::Disconnected) => (),
        }
    }

    /// Checks received chains and broadcasts them if they're valid and longer
    ///
    /// # Arguments
    /// * `chain` - The received chain to check
    /// * `sender` - Broadcast channel for chain updates
    fn check_remote_chain_and_broadcast(
        &mut self,
        chain: Chain,
        sender: &broadcast::Sender<Chain>,
    ) {
        if chain > self.chain {
            self.chain = chain;
            let _ = sender.send(self.chain.clone());
        }
    }

    // ------------------------------- // Neighbor Management // -------------------------------
    /// Handles the presentation of this node's ID when contacted by a neighbour
    ///
    /// # Arguments
    /// * `sender` - Address of the sending node
    /// * `buffer` - Data buffer containing neighbor information
    ///
    /// # Returns
    /// Optional reply or error
    pub async fn present_id(
        &mut self,
        sender: String,
        mut buffer: Vec<u8>,
    ) -> Result<Option<Box<dyn Reply>>, GossipError> {
        buffer.remove(0);
        let str_buffer = str::from_utf8(&buffer)
            .map_err(|_| GossipError::InvalidReplyError)?
            .trim();
        let cleared = Node::sanitize(str_buffer);
        let neighbour: Neighbour =
            serde_json::from_str(&cleared).map_err(|_| GossipError::InvalidReplyError)?;
        let hash_neighbour = neighbour.clone();
        self.neighbours
            .entry(hash_neighbour.id)
            .or_insert(hash_neighbour);
        self.new_neighbours.push(neighbour);
        // Sending ID back to the sender
        let _ = gossip::send_id(self.address.clone(), self.id, sender).await;
        Ok(None)
    }

    /// Removes a neighbour from the list based on the provided sender address
    ///
    /// # Arguments
    /// * `sender` - Address of the neighbor to remove
    ///
    /// # Returns
    /// Optional reply or IO error
    #[allow(clippy::needless_pass_by_value)]
    pub fn remove_neighbour(&mut self, sender: String) -> IOResult<Option<Box<dyn Reply>>> {
        self.neighbours.retain(|_, v| v.address != sender);
        Ok(None)
    }

    /// Adds a neighbour to this node's network from the provided buffer
    ///
    /// # Arguments
    /// * `buffer` - Data buffer containing neighbor information
    ///
    /// # Returns
    /// Optional reply or gossip error
    pub fn add_neighbour(
        &mut self,
        mut buffer: Vec<u8>,
    ) -> Result<Option<Box<dyn Reply>>, GossipError> {
        buffer.remove(0);
        let str_buffer = str::from_utf8(&buffer).map_err(|_| GossipError::InvalidReplyError)?;
        debug!("Received neighbour: {}", str_buffer);
        let cleared = Node::sanitize(str_buffer);
        let neighbour: Neighbour =
            serde_json::from_str(&cleared).map_err(|_| GossipError::InvalidReplyError)?;
        let hash_neighbour = neighbour.clone();
        self.neighbours
            .entry(hash_neighbour.id)
            .or_insert(hash_neighbour);
        self.new_neighbours.push(neighbour);
        Ok(None)
    }

    // ------------------------------- // Transaction Handling // -------------------------------
    /// Adds a transaction from the buffer, if this node is a miner
    ///
    /// # Arguments
    /// * `buffer` - Data buffer containing transaction
    ///
    /// # Returns
    /// Optional reply containing transaction or gossip error
    pub fn add_block_entry(
        &self,
        mut buffer: Vec<u8>,
    ) -> Result<Option<Box<dyn Reply>>, GossipError> {
        if self.role != Role::Miner {
            return Ok(None);
            // We can enhance this later to return an error
        }
        buffer.remove(0);
        let str_buffer = str::from_utf8(&buffer).map_err(|_| GossipError::InvalidReplyError)?;
        let transaction = Transaction::try_from(str_buffer.to_string())
            .map_err(|_| GossipError::InvalidReplyError)?;
        Ok(Some(Box::new(ReplySign(Box::new(transaction)))))
    }

    // ------------------------------- // Chain Management // -------------------------------
    /// Receives a chain from the buffer and returns it
    ///
    /// # Arguments
    /// * `buffer` - Data buffer containing chain
    ///
    /// # Returns
    /// Optional reply containing chain or gossip error
    pub fn get_chain(
        &mut self,
        mut buffer: Vec<u8>,
    ) -> Result<Option<Box<dyn Reply>>, GossipError> {
        buffer.remove(0);
        let str_buffer = str::from_utf8(&buffer).map_err(|_| GossipError::InvalidReplyError)?;
        let cleared = Node::sanitize(str_buffer);
        let chain: Chain =
            serde_json::from_str(&cleared).map_err(|_| GossipError::InvalidReplyError)?;
        Ok(Some(Box::new(chain)))
    }

    /// Shares the current chain with any requesting neighbour
    ///
    /// # Returns
    /// Optional reply or IO error
    pub fn share_chain(&self) -> IOResult<Option<Box<dyn Reply>>> {
        Ok(None)
    }

    // ------------------------------- // Utility Methods // -------------------------------
    /// Sanitizes a string by only allowing alphanumeric characters and a few special characters
    fn sanitize(string: &str) -> String {
        let accepted_chars = " \",;:.-{}[]_=/+";
        string
            .chars()
            .take_while(|c| c.is_alphanumeric() || accepted_chars.contains(*c))
            .collect()
    }

    /// Checks for new blocks received from peers
    ///
    /// # Arguments
    /// * `receiver` - Broadcast receiver for chain updates
    fn check_peer_mined_chains(&mut self, receiver: &mut broadcast::Receiver<Chain>) {
        let chain = receiver.try_recv();
        if let Ok(recv_chain) = chain {
            if recv_chain > self.chain {
                self.chain = recv_chain;
            }
        }
    }
}

/// Performs the mining operation for a miner node
///
/// # Arguments
/// * `miner` - The miner instance to use
/// * `chain` - The current blockchain
///
/// # Returns
/// Updated chain with new block if mining successful
#[allow(clippy::unwrap_used)]
fn mine(miner: &Arc<sync::Mutex<Miner>>, mut chain: Chain) -> Chain {
    let mut mining_in_progress = true;
    while mining_in_progress {
        miner.lock().unwrap().set_chain_meta(chain.clone());
        if let Ok(mining_digest) = miner.lock().unwrap().mine(chain.get_last_block()) {
            info!("Mined block: {}", mining_digest.get_block());
            let _ = chain.add_block(mining_digest);
            mining_in_progress = false;
        }
    }
    chain
}

/// Submits a transaction to all miner neighbours
///
/// # Arguments
/// * `transaction` - The transaction to submit
/// * `neighbours` - Map of neighbor nodes
/// * `address` - This node's address
#[allow(clippy::implicit_hasher)]
pub fn submit_transaction(
    transaction: &Transaction,
    neighbours: &HashMap<Uuid, Neighbour>,
    address: &Arc<str>,
) {
    let _ = neighbours
        .iter()
        .filter(|neighbour| neighbour.1.role == Role::Miner)
        // Filters only miners
        .map(|miner| async {
            gossip::send_transaction(
                address.clone(),
                miner.1.address.clone(),
                transaction.clone(),
            )
            .await
        })
        .collect::<Vec<_>>();
}

/// Handles transaction reception and processing
///
/// # Arguments
/// * `receiver` - Channel receiver for transactions
/// * `neighbours` - Map of neighbor nodes
/// * `address` - This node's address
/// * `miner` - Optional miner instance if this node is a miner
/// * `log_sender` - Optional channel for sending log messages
async fn listen_to_transactions(
    receiver: Arc<Mutex<Receiver>>,
    neighbours: HashMap<Uuid, Neighbour>,
    address: Arc<str>,
    miner: Option<Arc<Mutex<Arc<sync::Mutex<Miner>>>>>,
    log_sender: Option<Sender<String>>,
) {
    match receive_transaction(receiver).await {
        Ok(transaction) => {
            println!("[{address}] Transaction being received: {}", &transaction);
            match miner {
                Some(m) => {
                    let mut miner_ref = m.clone();
                    if let Some(sender) = log_sender {
                        let _ = sender.send("Transaction Received".to_string()).await;
                    }
                    push_transaction(&mut miner_ref, Box::new(transaction)).await;
                }
                _ => submit_transaction(&transaction, &neighbours, &address),
            }
        }
        Err(e) => println!("[{address}] receive transaction failed: {e}"),
    }
}

/// Handles the gossiping process with random neighbours
///
/// # Arguments
/// * `address` - This node's address
/// * `chain` - Current blockchain
/// * `random_neighbours` - Neighbors to gossip with
/// * `new_neighbours` - Newly discovered neighbors
/// * `theme` - Current gossip theme (what to gossip about)
/// * `_receiver` - Broadcast receiver for chain updates
pub async fn gossip(
    address: Arc<str>,
    chain: Chain,
    random_neighbours: Vec<Neighbour>,
    new_neighbours: Vec<Neighbour>,
    theme: Theme,
    _receiver: broadcast::Receiver<Chain>,
) {
    gossip::wait_gossip_interval().await;
    for neighbour in random_neighbours {
        match theme {
            Theme::Chain => {
                if chain.get_len() > 0 {
                    let _ = gossip::send_chain(
                        address.clone(),
                        neighbour.address.clone(),
                        chain.clone(),
                        //TODO: Shouldn't have to clone eveyt time.
                    )
                    .await;
                }
            }
            Theme::NewNeighbours => {
                if !new_neighbours.is_empty() {
                    let _ = gossip::send_new_neighbours(
                        neighbour.id,
                        neighbour.address.clone(),
                        address.clone(),
                        new_neighbours.clone(),
                    )
                    .await;
                }
            }
        }
    }
}

/// Receives a transaction from the receiver channel
///
/// # Arguments
/// * `receiver` - Channel receiver for transactions
///
/// # Returns
/// Received transaction or error
async fn receive_transaction(
    receiver: Arc<Mutex<Receiver>>,
) -> Result<Transaction, TransactionRecvError> {
    let mut inner_receiver = receiver.lock().await;
    let str_transaction = { inner_receiver.recv().await? };
    match Transaction::try_from(str_transaction.clone()) {
        Ok(transaction) => Ok(transaction),
        Err(e) => Err(TransactionRecvError::TransactionFromBase64Error(e)), // Consider handling this more gracefully
    }
}

/// Attempts to mine a new block
///
/// # Arguments
/// * `node_id` - ID of this node
/// * `miner_opt` - Optional miner instance
/// * `chain` - Current blockchain
/// * `mining_sender` - Channel for sending mined chains
#[allow(clippy::unwrap_used)]
async fn try_mine(
    node_id: Uuid,
    miner_opt: Option<Arc<Mutex<Arc<sync::Mutex<Miner>>>>>,
    chain: Chain,
    mining_sender: &'static mpsc::Sender<Chain>,
) {
    if let Some(miner) = miner_opt {
        let current_chain = chain;
        let loop_miner = {
            let guard = miner.lock().await;
            guard.clone()
        };
        println!("about to mine...");
        let new_chain = tokio::task::spawn_blocking(move || {
            println!("actually mining...");
            let new_chain = mine(&loop_miner, current_chain.clone());
            info!(
                "node {} has succefully mined a block and now it is: {}",
                node_id, new_chain
            );
            new_chain
        })
        .await
        .unwrap();
        match mining_sender.send(new_chain).await {
            Ok(()) => (),
            Err(e) => println!("Failed to send new chain due to {e}"),
        }
    } else {
        println!("No miner...");
    }
}

/// Pushes a transaction to a miner's transaction queue
///
/// # Arguments
/// * `miner` - The miner instance
/// * `transaction` - The transaction to push
#[allow(clippy::unwrap_used)]
async fn push_transaction(
    miner: &mut Arc<Mutex<Arc<sync::Mutex<Miner>>>>,
    transaction: Box<dyn BlockEntry>,
) {
    let guard = miner.lock().await;
    let mut inner = guard.lock().unwrap();
    inner.push_entry(transaction);
}
