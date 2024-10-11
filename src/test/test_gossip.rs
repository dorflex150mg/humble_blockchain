pub mod test_gossip {

    use crate::{
        Transaction,
        Wallet,
        node::{
            neighbour::neighbour::{Neighbour, Role},
            gossip::gossip,
            protocol::protocol,
            node::node::Node,
            receiver::receiver::Receiver,
        },
    };

    use std::{
        thread,
        time::Duration,
        sync::Arc,
    };

    use tokio::sync::{
        mpsc::{self, Sender},
        Mutex,
    };

    use tracing::{debug, info};

    /// Creates a mock transaction between two wallets with a made-up token.
    ///
    /// # Returns
    /// * `Transaction` - The signed transaction.
    fn make_up_transaction() -> Transaction {
        let some_token = "0".repeat(64); // A made-up coin token represented as a string of 64 zeros
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        
        let transaction: Transaction = Transaction::new(
            wallet1.get_pub_key(),
            wallet2.get_pub_key(),
            vec![some_token], 
        );

        wallet1.sign(transaction)
    }

    /// Repeatedly sends mock transactions to a given channel.
    ///
    /// # Arguments
    /// * `tx` - The transaction sender channel.
    /// * `iterations` - Optional number of iterations. If `None`, the loop will run indefinitely.
    async fn send_transaction_loop(mut tx: Sender<String>, iterations: Option<u32>) {
        async fn _send_transaction_single(tx: Sender<String>) -> Sender<String> {
            let t1 = make_up_transaction();
            info!("sending transaction: {}", t1);
            
            // Send the transaction over the channel
            if let Err(e) = tx.send(t1.into()).await {
                info!("Error sending transaction: {}", e);
            }
                                                                  
            // Sleep for 500 milliseconds between sends
            tokio::time::sleep(Duration::from_millis(500)).await;
            tx
        }
        match iterations {
            Some(n) => {
                for i in 0..n {
                    tx = _send_transaction_single(tx).await;   
                }
            },
            None => {
                loop {
                    tx = _send_transaction_single(tx).await;
                }
            }
        }
    }

    /// Test function to simulate a gossip protocol with multiple nodes.
    ///
    /// This function spawns three nodes: one tracker, one regular node, and one miner. 
    /// It then starts sending mock transactions to test the gossip protocol between the nodes.
    pub async fn test_gossip() {
        info!("Starting gossip test");

        // Create the first node (Tracker)
        let (tx1, rx1) = mpsc::channel::<String>(1024); // Create a communication channel for transactions
        let node1 = Node::new(
            Role::Tracker,
            "127.0.0.1:8081".to_owned(),
            None, // No neighbours for the tracker
            Receiver::new(rx1),
        );
        let arc_node1 = Arc::new(Mutex::new(node1));
        let clone1 = Arc::clone(&arc_node1);

        // Create the second node (Regular Node)
        let (tx2, rx2) = mpsc::channel::<String>(1024);
        let mut node2 = Node::new(
            Role::Node,
            "127.0.0.1:8082".to_owned(),
            Some(vec!["127.0.0.1:8081".to_owned()]), // Node 2 connects to the tracker
            Receiver::new(rx2),
        );

        // Spawn the Tracker node's event loop
        tokio::spawn(async move {
            clone1.lock().await.node_loop().await;
        });

        // Spawn the second node and start its event loop
        tokio::spawn(async move {
            node2.enter_and_node_loop().await;
        });

        // Allow some time for the nodes to initialize
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Create the third node (Miner)
        let (tx3, rx3) = mpsc::channel::<String>(1024);
        let mut node3 = Node::new(
            Role::Miner,
            "127.0.0.1:8083".to_owned(),
            Some(vec!["127.0.0.1:8081".to_owned()]), // Miner connects to the tracker as well
            Receiver::new(rx3),
        );

        // Spawn the Miner node's event loop
        tokio::spawn(async move {
            node3.enter_and_node_loop().await;
        });

        // Give some time for the miner node to be added to the network
        tokio::time::sleep(Duration::from_secs(3)).await;

        // Start sending transactions from the first node (tracker)
        send_transaction_loop(tx1, Some(1_000)).await;

        // Keep the function alive to continue processing
        loop {}
    }
}

