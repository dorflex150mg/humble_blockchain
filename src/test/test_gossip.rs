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
        sync::Arc,
    };

    use tokio::sync::Mutex;


    pub async fn test_gossip() {
        println!("Starting gossip test");
        let mut node1 = Node::new(Role::Tracker, "127.0.0.1:8081".to_owned(), None).unwrap(); 
        let mut arc_node1 = Arc::new(Mutex::new(node1));
        let mut clone1 = arc_node1.clone();
        let mut clone2 = arc_node1.clone();
        let mut node2 = Node::new(Role::Node, "127.0.0.1:8082".to_owned(), Some(vec!["127.0.0.1:8081".to_owned()])).unwrap(); 
        tokio::spawn(async move {
            clone1.lock().await.listen_to_greet().await;
        });
        tokio::spawn(async move {
            clone2.lock().await.gossip().await;
        });
        tokio::time::sleep(Duration::new(1, 0)).await;
        tokio::spawn(async move {
            node2.enter_network().await;
        });
        loop {}
    }
}
