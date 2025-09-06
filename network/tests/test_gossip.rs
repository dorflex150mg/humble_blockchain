#[cfg(test)]
use network::{
    node::{
        neighbour::Role,
        node::Node,
        receiver::Receiver,
    },
};

use std::time::Duration;

use tokio::sync::mpsc;
use std::sync::{Arc, Mutex};

use transaction::transaction::Transaction;
use wallet::wallet::Wallet;

use tracing::{info, debug};

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
async fn send_transaction_loop(mut tx: mpsc::Sender<String>, iterations: Option<u32>) {
    async fn _send_transaction_single(tx: mpsc::Sender<String>) -> mpsc::Sender<String> {
        let t1 = make_up_transaction();
        println!("[transaction loop]: Sending transaction...");
        // Send the transaction over the channel
        if let Err(e) = tx.send(t1.into()).await {
            println!("Error sending transaction: {}", e);
        }
        // Sleep for 500 milliseconds between sends
        tokio::time::sleep(Duration::from_millis(500)).await;
        tx
    }
    match iterations {
        Some(n) => {
            for _ in 0..n {
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
#[tokio::test]
pub async fn test_gossip() {
    println!("Starting gossip test");

    // Create the first node (Tracker)
    let (_, rx1) = mpsc::channel::<String>(1024); // Create a communication channel for transactions
    let node1 = Node::new(
        Role::Tracker,
        "127.0.0.1:8081".to_owned(),
        None, // No neighbours for the tracker
        Receiver::new(rx1),
        None,
    );
    let arc_node1 = Arc::new(Mutex::new(node1));
    let clone1 = Arc::clone(&arc_node1);

    // Create the second node (Regular Node)
    let (log_sender, mut log_receiver) = mpsc::channel::<String>(1024);
    let (_, rx2) = mpsc::channel::<String>(1024);
    let mut node2 = Node::new(
        Role::Node,
        "127.0.0.1:8082".to_owned(),
        Some(vec!["127.0.0.1:8081".to_owned()]), // Node 2 connects to the tracker
        Receiver::new(rx2),
        Some(log_sender),
    );

    // Spawn the Tracker node's event loop
    tokio::spawn(async move {
        let mut node = {
            let guard= clone1.lock().unwrap();
            guard.clone()
        };
        let _ = node.node_loop().await;
    });


    // Spawn the second node and start its event loop
    tokio::spawn(async move {
        let _ = node2.enter_and_node_loop().await;
    });


    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(15)) => {
            panic!("Assertion failed");
        },

        log = log_receiver.recv() => {
            let res = log.unwrap();
            println!("res: {}", res);
            assert_eq!(res, "NeighbourAdded");
        }
    };
            
    // Allow some time for the nodes to initialize
    tokio::time::sleep(Duration::from_secs(3)).await;

    // Create the third node (Miner)
    let (tx1, rx3) = mpsc::channel::<String>(1024);
    let (log_sender, log_receiver) = mpsc::channel::<String>(1024);
    let mut node3 = Node::new(
        Role::Miner,
        "127.0.0.1:8083".to_owned(),
        Some(vec!["127.0.0.1:8081".to_owned()]), // Miner connects to the tracker as well
        Receiver::new(rx3),
        Some(log_sender),
    );


    async fn recv_3(mut log_receiver: mpsc::Receiver<String>) -> Vec<String> {
        let mut recv = 0;
        let mut logs = vec![];
        while recv < 3 {
            let log = log_receiver.recv().await.unwrap();
            if let Ok(mined_blocks) = log.parse::<usize>() {
                recv = mined_blocks;
            } 
            logs.push(log);
        }
        logs
    }

    // Spawn the Miner node's event loop
    tokio::spawn(async move {
        let _ = node3.enter_and_node_loop().await;
    });

    // Give some time for the miner node to be added to the network
    tokio::time::sleep(Duration::from_secs(3)).await;
    tokio::spawn(send_transaction_loop(tx1, None));
    tokio::time::sleep(Duration::from_secs(3)).await;


    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(15)) => {
            panic!("Assertion failed");
        },
        log_strings = recv_3(log_receiver) => {
            let mut rev = log_strings.iter().rev().peekable();
            let mut mined_blocks = 0;
            let mut transaction_ack = None;
            let mut neighbour_added = 0; 
            while rev.peek().is_some() {
                let next = rev.next().unwrap();
                if next.parse::<usize>().is_ok() {
                    mined_blocks += 1;
                } else if next == "Transaction Received" { 
                    transaction_ack = Some(next);
                } else if next == "NeighbourAdded" {
                    neighbour_added += 1;
                }
            }
            assert!(transaction_ack.is_some());
            assert!(mined_blocks >= 2);
            assert_eq!(neighbour_added, 1);
        }
    };
            
    // Start sending transactions from the first node (tracker)

    // Keep the function alive to continue processing

}
