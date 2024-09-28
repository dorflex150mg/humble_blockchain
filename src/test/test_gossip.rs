pub mod test_gossip {
    
    use crate::node::{
        node::node::Node as Node,
        neighbour::neighbour::{
            Neighbour as Neighbour,
            Role as Role,
        },
        gossip::gossip as gossip,
        protocol::protocol as protocol,
    };

    use std::{
        thread,
        time::Duration,
        sync::{Arc, Mutex},
    };


    pub async fn test_gossip() {
        println!("Starting gossip test");
        let mut node1 = Node::new(Role::Tracker, "127.0.0.1:8081".to_owned(), None).unwrap(); 
        let mut node2 = Node::new(Role::Node, "127.0.0.1:8082".to_owned(), Some(vec!["127.0.0.1:8081".to_owned()])).unwrap(); 
        tokio::spawn(async move {
            node1.listen_to_greet().await;
            tokio::time::sleep(Duration::new(2, 0)).await;
        });
        tokio::time::sleep(Duration::new(1, 0)).await;
        tokio::spawn(async move {
            node2.enter_network().await;
        });
        loop {}
    }
}
