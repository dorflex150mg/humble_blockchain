use crate::node::{
    gossip::{self, GossipError, GossipReply},
    neighbour::{Neighbour, Role},
    protocol,
    receiver::Receiver,
    reply::Reply,
    theme::Theme,
};
use chain::chain::Chain;
use chain::miner::miner::{ChainMeta, Miner};
use tokio::sync::{
    broadcast,
    mpsc::{self, error::TryRecvError, Sender},
    Mutex,
};
use wallet::transaction::block_entry_common::EntryDecodeError;
use wallet::transaction::transaction::Transaction;

use std::{
    collections::HashMap,
    io::{Error as IOError, Result as IOResult},
    str,
    sync::Arc,
};

use rand::prelude::*;
use thiserror::Error;
use tracing::{debug, info};
use uuid::{self, Uuid};

#[allow(dead_code)]
const DEFAULT_ADDRESS: &str = "127.0.0.1";

// -------------------------------
// Error Definitions
// -------------------------------

/// Errors that may occur when a node tries to enter the network.
#[derive(Error, Debug)]
pub enum EnterAttemptError {
    /// No trackers were listening when attempting to enter.
    #[error("Failed to enter network - No trackers listening.")]
    NoListeners,
    /// No trackers were available to contact.
    #[error("Failed to enter network - No trackers available.")]
    NoTrackers,
}

/// Errors that may occur when attempting to update the chain.
#[derive(Error, Debug)]
pub enum UpdateChainError {
    /// No neighbours were listening for an update.
    #[error("Failed to update chain - No neighbours listening.")]
    NoListeners,
}

/// Errors returned when a node is in the wrong role for an operation.
#[derive(Error, Debug, derive_more::From)]
pub enum WrongRoleError {
    /// The operation requires a node with role `Miner`.
    #[error("That operation requires a Node with Role Miner.")]
    NotMiner,
    /// The operation requires a node with role `Tracker`.
    #[error("That operation requires a Node with Role Tracker.")]
    NotTracker,
}

/// Errors that may occur while listening on the network.
#[derive(Error, Debug, derive_more::From)]
pub enum ListenError {
    /// The node is in the wrong role.
    #[error(transparent)]
    WrongRoleError(WrongRoleError),
    /// Underlying I/O error.
    #[error(transparent)]
    IOError(IOError),
}

/// Errors that may occur while receiving a transaction.
#[derive(Error, Debug, derive_more::From)]
pub enum TransactionRecvError {
    /// Failed to receive from channel.
    #[error(transparent)]
    TryRecvError(TryRecvError),
    /// Failed to decode transaction from base64.
    #[error(transparent)]
    TransactionFromBase64Error(EntryDecodeError),
}

/// Errors that may occur during the main node loop.
#[derive(Error, Debug, derive_more::From)]
pub enum NodeLoopError {
    /// Failed to enter the network.
    #[error(transparent)]
    EnterAttemptError(EnterAttemptError),
    /// A gossip-related error occurred.
    #[error(transparent)]
    GossipError(GossipError),
}

// -------------------------------
// Node Structure Definition
// -------------------------------

/// Represents a node in the network.  
/// A node can act as a miner or tracker, hold a blockchain,
/// manage neighbours, and handle transactions.
#[derive(Clone)]
pub struct Node {
    /// Unique identifier of the node.
    pub id: Uuid,
    /// The role of this node (Miner or Tracker).
    pub role: Role,
    /// The network address of this node.
    pub address: Arc<str>,
    /// Buffer for storing transactions before they are mined.
    pub transaction_buffer: Option<Vec<Transaction>>,
    /// The blockchain this node maintains.
    pub chain: Chain,
    /// Current neighbours of this node, keyed by their UUID.
    pub neighbours: HashMap<Uuid, Neighbour>,
    /// Recently discovered neighbours awaiting integration.
    pub new_neighbours: Vec<Neighbour>,
    /// Whether this node has successfully joined the network.
    pub initialized: bool,
    /// Optional list of tracker addresses for joining the network.
    pub trackers: Option<Vec<String>>,
    /// Receiver for incoming messages and transactions.
    pub receiver: Arc<Mutex<Receiver>>,
    /// Local miner instance if this node is a miner.
    pub miner: Option<Arc<Mutex<Miner>>>,
    /// Optional sender for logging messages.
    pub log_sender: Option<mpsc::Sender<String>>,
}

// -------------------------------
// Node Implementation
// -------------------------------

impl Node {
    /// Creates a new `Node` instance.
    pub fn new(
        role: Role,
        address: String,
        trackers: Option<Vec<String>>,
        receiver: Receiver,
        log_sender: Option<Sender<String>>,
    ) -> Self {
        let mut transaction_buffer = None;
        let mut miner = None;

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
            receiver: Arc::new(Mutex::new(receiver)),
            miner,
            log_sender,
        }
    }

    /// Sends a log message through the node's optional logger.
    pub async fn update_log(&self, log_msg: impl Into<String>) {
        if let Some(log_sender) = &self.log_sender {
            let _ = log_sender.send(log_msg.into()).await;
        }
    }

    /// Returns the node's address.
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
        let (sender, _) = broadcast::channel::<Chain>(16);
        let (mut mining_sender, mut mining_receiver) = mpsc::channel::<Chain>(16);

        let mut receiver = sender.subscribe();

        loop {
            self.listen_to_peers(sender.clone(), &mut mining_receiver, receiver.resubscribe())
                .await?;

            if self.role == Role::Miner {
                if let Some(miner) = &self.miner {
                    let mut miner = miner.lock().await;
                    let mut buffer = self.transaction_buffer.take().unwrap_or_default();

                    miner.update_chain(&self.chain);
                    miner.add_transactions(&mut buffer);
                    miner.set_tx(buffer);

                    let miner_sender = mining_sender.clone();
                    tokio::spawn(async move {
                        if let Some(chain) = miner.start_mining().await {
                            miner_sender.send(chain).await.unwrap();
                        }
                    });

                    self.transaction_buffer = Some(miner.take_tx());
                }
            }
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
            if trackers.is_empty() {
                return Err(EnterAttemptError::NoTrackers);
            }

            for tracker in trackers {
                let tracker = tracker.clone();
                let reply = protocol::present_id(self.id, &self.address, &tracker)
                    .await
                    .ok();

                if let Some(reply) = reply {
                    let neighbour = Neighbour::new(tracker, Role::Tracker);
                    self.neighbours.insert(reply.id, neighbour);
                    self.initialized = true;
                    return Ok(());
                }
            }

            Err(EnterAttemptError::NoListeners)
        } else {
            Err(EnterAttemptError::NoTrackers)
        }
    }

    /// Leaves the network by sending farewell messages to all neighbours.
    pub async fn leave_network(&self) {
        for neighbour in self.neighbours.values() {
            protocol::remove_neighbour(&self.id, &self.address, neighbour).await;
        }
    }

    // -------------------------------
    // Transaction and Chain Operations
    // -------------------------------

    /// Polls neighbours for chain updates and returns the first valid chain found.  
    /// Returns an error if no neighbour responds.
    pub async fn update_chain(&self) -> Result<Chain, UpdateChainError> {
        for neighbour in self.neighbours.values() {
            if let Some(reply) = protocol::request_chain(&self.address, neighbour).await.ok() {
                if let GossipReply::Chain(chain) = reply {
                    return Ok(chain);
                }
            }
        }
        Err(UpdateChainError::NoListeners)
    }

    // -------------------------------
    // Gossip and Neighbor Management
    // -------------------------------

    /// Returns a random subset of neighbours to gossip with.  
    /// The number of neighbours chosen is proportional to √N.
    pub fn get_random_neighbours(&self) -> Vec<Neighbour> {
        let n = self.neighbours.len();
        let k = (n as f64).sqrt().ceil() as usize;
        let mut rng = thread_rng();
        self.neighbours
            .values()
            .cloned()
            .choose_multiple(&mut rng, k)
    }

    // -------------------------------
    // Listening and Chain Validation
    // -------------------------------

    /// Listens for incoming messages and processes them based on the protocol.  
    /// Handles gossip replies, neighbour management, chain updates, and transactions.
    pub async fn listen_to_peers(
        &mut self,
        sender: broadcast::Sender<Chain>,
        mining_receiver: &mut mpsc::Receiver<Chain>,
        mut receiver: broadcast::Receiver<Chain>,
    ) -> Result<(), GossipError> {
        let mut rec = self.receiver.lock().await;

        if let Ok((sender_addr, buffer)) = rec.try_recv() {
            if let Some(reply) = protocol::handle_message(
                self,
                sender_addr.clone(),
                buffer.clone(),
                &sender,
                mining_receiver,
                &mut receiver,
            )
            .await?
            {
                protocol::send_reply(&sender_addr, reply).await?;
            }
        }

        Ok(())
    }

    /// Updates the node's chain if a mined chain from the local miner is longer,
    /// and broadcasts the updated chain.
    pub fn check_mined_chain_and_broadcast(
        &mut self,
        sender: &broadcast::Sender<Chain>,
        mining_receiver: &mut mpsc::Receiver<Chain>,
    ) {
        if let Ok(chain) = mining_receiver.try_recv() {
            if chain.meta().len > self.chain.meta().len {
                self.chain = chain.clone();
                let _ = sender.send(chain);
            }
        }
    }

    /// Updates the node's chain if a longer chain is received from peers,
    /// and broadcasts it.
    pub fn check_remote_chain_and_broadcast(
        &mut self,
        chain: Chain,
        sender: &broadcast::Sender<Chain>,
    ) {
        if chain.meta().len > self.chain.meta().len {
            self.chain = chain.clone();
            let _ = sender.send(chain);
        }
    }

    // -------------------------------
    // Neighbor Management
    // -------------------------------

    /// Handles the presentation of this node's ID when contacted by a neighbour.  
    /// Adds the neighbour to the list and sends this node's ID back.
    pub async fn present_id(
        &mut self,
        sender: String,
        buffer: Vec<u8>,
    ) -> Result<Option<Box<dyn Reply>>, GossipError> {
        let id = Uuid::from_slice(&buffer)?;
        let neighbour = Neighbour::new(sender.clone(), Role::Miner);
        self.neighbours.insert(id, neighbour);
        Ok(Some(Box::new(self.id)))
    }

    /// Removes a neighbour from the list based on the provided sender address.
    pub async fn remove_neighbour(&mut self, sender: String) -> IOResult<Option<Box<dyn Reply>>> {
        self.neighbours.retain(|_, n| n.address != sender);
        Ok(None)
    }

    /// Adds a neighbour to this node's network from the provided buffer.
    pub async fn add_neighbour(
        &mut self,
        buffer: Vec<u8>,
    ) -> Result<Option<Box<dyn Reply>>, GossipError> {
        let neighbour: Neighbour = bincode::deserialize(&buffer)?;
        self.new_neighbours.push(neighbour);
        Ok(None)
    }

    // -------------------------------
    // Transaction Handling
    // -------------------------------

    /// Adds a transaction from the buffer, if this node is a miner.  
    /// Returns the transaction as a `Reply` if valid.
    pub async fn add_transaction(
        &self,
        buffer: Vec<u8>,
    ) -> Result<Option<Box<dyn Reply>>, GossipError> {
        let tx: Transaction = bincode::deserialize(&buffer)?;
        if let Some(buffer) = &self.transaction_buffer {
            let mut buffer = buffer.clone();
            buffer.push(tx.clone());
            Ok(Some(Box::new(tx)))
        } else {
            Ok(None)
        }
    }

    // -------------------------------
    // Chain Management
    // -------------------------------

    /// Receives a chain from the buffer and returns it as a `Reply`.
    pub async fn get_chain(
        &mut self,
        buffer: Vec<u8>,
    ) -> Result<Option<Box<dyn Reply>>, GossipError> {
        let chain: Chain = bincode::deserialize(&buffer)?;
        Ok(Some(Box::new(chain)))
    }

    /// Shares the current chain with any requesting neighbour.
    pub async fn share_chain(&self) -> IOResult<Option<Box<dyn Reply>>> {
        Ok(Some(Box::new(self.chain.clone())))
    }
}

// -------------------------------
// Public Free Functions
// -------------------------------

/// Submits a transaction to all neighbours with role `Miner`.
pub async fn submit_transaction(
    transaction: Transaction,
    neighbours: HashMap<Uuid, Neighbour>,
    address: Arc<str>,
) {
    for neighbour in neighbours.values() {
        if neighbour.role == Role::Miner {
            let _ = protocol::submit_transaction(&transaction, &address, neighbour).await;
        }
    }
}

/// Handles the gossiping process with random neighbours,
/// spreading either the chain or new neighbours depending on the theme.
pub async fn gossip(
    address: Arc<str>,
    chain: Chain,
    random_neighbours: Vec<Neighbour>,
    new_neighbours: Vec<Neighbour>,
    theme: Theme,
    _receiver: broadcast::Receiver<Chain>,
) {
    for neighbour in random_neighbours {
        let _ =
            protocol::gossip(&address, &chain, &new_neighbours, theme.clone(), &neighbour).await;
    }
}
