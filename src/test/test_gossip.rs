pub mod test_gossip {
    
    use crate::{
        Transaction,
        Wallet,
        node::{
            neighbour::neighbour::{
                Neighbour as Neighbour,
                Role as Role,
            },
            gossip::gossip as gossip,
            protocol::protocol as protocol,
            node::node::Node as Node,
            receiver::receiver::Receiver,
        },
    };

    use std::{
        thread,
        time::Duration,
        sync::Arc,
    };

    use tokio::sync::{
        mpsc,
        Mutex,
    };


    pub async fn test_gossip() {
        println!("Starting gossip test");
        let (tx1, mut rx1) = mpsc::channel::<String>(1024);
        let mut node1 = Node::new( //The first node must be a tracker.
            Role::Tracker, 
            "127.0.0.1:8081".to_owned(), 
            None, 
            Receiver::new(rx1),
        );
        let mut arc_node1 = Arc::new(Mutex::new(node1));
        let mut clone1 = arc_node1.clone();
        let (tx2, mut rx2) = mpsc::channel::<String>(1024);
        let mut node2 = Node::new(
            Role::Node, 
            "127.0.0.1:8082".to_owned(), 
            Some(vec!["127.0.0.1:8081".to_owned()]), 
            Receiver::new(rx2),
        );
        tokio::spawn(async move {
            clone1.lock().await.node_loop().await;
        });
        tokio::spawn(async move {
            node2.enter_and_node_loop().await;
        });
        tokio::time::sleep(Duration::new(3, 0)).await;
        let (tx3, mut rx3) = mpsc::channel::<String>(1024);
        let mut node3 = Node::new( 
            Role::Miner, 
            "127.0.0.1:8083".to_owned(), 
            Some(vec!["127.0.0.1:8081".to_owned()]), 
            Receiver::new(rx3),
        );
        //tokio::time::sleep(Duration::new(3, 0)).await;
        tokio::spawn(async move {
            println!("spawning third node");
            node3.enter_and_node_loop().await;
        });
        tokio::time::sleep(Duration::new(3, 0)).await; //Waiting for the miner node to be added
        let some_token = "0".repeat(64); //some made up coin token
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let transaction: Transaction = Transaction::new(
            wallet1.get_pub_key(), 
            wallet2.get_pub_key(), 
            vec![some_token]
        );
        let signed_t = wallet1.sign(transaction);
        println!("sending transaction: {}", signed_t);
        tx1.send(signed_t.into()).await;
        println!("finished");
        loop {}
    }
}
