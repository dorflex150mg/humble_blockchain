pub mod gossip {

    use crate::Chain;
    use crate::Transaction;
    use crate::node::neighbour::neighbour::{
        Neighbour,
        Role,
    };
    use crate::node::protocol::protocol;

    use std::{
        io::Result as IOResult, 
        time::Duration,
        str,
        thread,
    };

    use tokio::{
        net::UdpSocket,
        time::timeout,
    };
    use uuid::Uuid;

    pub const GOSSIP_INTERVAL: u64 = 3;
    pub const UUID_LENGTH: usize = 36;
    pub const MAX_DATAGRAM_SIZE: usize = 65507;


    pub async fn greet(address: String, id: Uuid, role: Role, tracker: &str) -> IOResult<Neighbour> {
        let socket = UdpSocket::bind(&address).await?;
        let greeter = Neighbour {
            id,
            address,
            role,
        };
        let neighbour_str: String = serde_json::to_string(&greeter).unwrap();
        let ptcl: Vec<u8> = vec![protocol::GREET];
        let neighbour_bytes: Vec<u8> = neighbour_str.as_bytes().to_vec();
        let buffer = [ptcl, neighbour_bytes].concat();
        //let buffer: [u8; 1] = [protocol::GREET;1];
        let mut failed = true;
        let mut buffer_recv: [u8; UUID_LENGTH] = [0; UUID_LENGTH];
        while failed {
            socket.send_to(&buffer, tracker).await?;
            failed = false;
            if let Err(_) = timeout(Duration::new(1, 0), socket.recv_from(&mut buffer_recv)).await {
                println!("Retrying recv_from");
                failed = true;
            }
        }
        let str_id = str::from_utf8(&buffer_recv).unwrap();
        println!("new neighbour");
        Ok(
            Neighbour {
                id: Uuid::parse_str(str_id).unwrap(),
                address: tracker.to_string(),
                role: Role::Tracker,
            }
        )
    }

    pub async fn farewell(address: String, neighbour: String) -> IOResult<()> {
        let socket = UdpSocket::bind(address).await?;
        let buffer: [u8; 1] = [protocol::FAREWELL;1];
        socket.send_to(&buffer, &neighbour).await?;
        Ok(())
    }

    pub async fn send_transaction(address: String, miner: String, transaction: Transaction) -> IOResult<()> {
        let socket = UdpSocket::bind(address).await?;
        let str_transaction: String = transaction.into();
        let mut  buffer: Vec<u8> = vec![protocol::TRANSACTION];
        let tx_bytes = str_transaction.as_bytes().to_vec();
        buffer = [buffer, tx_bytes].concat();
        socket.send_to(&buffer, &miner).await?;        
        Ok(())
    }

    pub async fn poll_chain(address: String, neighbour: &Neighbour) -> IOResult<Chain> {
        let socket = UdpSocket::bind(address).await?;
        let buffer: [u8; 1] = [protocol::POLLCHAIN;1];
        socket.send_to(&buffer, &neighbour.address).await?;
        let mut buffer: [u8; MAX_DATAGRAM_SIZE] = [0; MAX_DATAGRAM_SIZE];
        socket.recv_from(&mut buffer).await?;
        match str::from_utf8(&buffer) {
            Ok(s) => Ok(serde_json::from_str(&s).unwrap()),
            Err(e) => panic!("Wrong character on chain: {}", e),
        }
    }

    pub async fn send_chain(address: String, neighbour: String, chain: Chain) -> IOResult<()> {
        let socket = UdpSocket::bind(address).await?;
        let str_chain: String = serde_json::to_string(&chain).unwrap();
        let mut buffer: Vec<u8> = vec![protocol::CHAIN];
        let chain_bytes = str_chain.as_bytes().to_vec();
        buffer = [buffer, chain_bytes].concat();
        socket.send_to(&buffer, &neighbour).await?;        
        Ok(())
    }

    pub async fn send_new_neighbours(address: String, neighbour: String, new_neighbours: Vec<Neighbour>) -> IOResult<()> {
        for new_neighbour in new_neighbours {
            let socket = UdpSocket::bind(&address).await?;
            let str_neighbour: String = serde_json::to_string(&new_neighbour).unwrap();
            let mut buffer: Vec<u8> = vec![protocol::NEIGHBOUR];
            let neighbour_bytes = str_neighbour.as_bytes().to_vec();
            buffer = [buffer, neighbour_bytes].concat();
            socket.send_to(&buffer, &neighbour).await?;        

        }
        Ok(())
    }

    pub async fn wait_gossip_interval() {
        tokio::time::sleep(Duration::new(GOSSIP_INTERVAL, 0)).await;
    }


    pub async fn listen_to_gossip(address: String) -> IOResult<(u8, String, Vec<u8>)> {
        let socket = UdpSocket::bind(address).await?;
        let mut buffer: [u8; MAX_DATAGRAM_SIZE] = [0; MAX_DATAGRAM_SIZE];
        let (_n_bytes, sender) = socket.recv_from(&mut buffer).await?;
        let ptcl = buffer[0];
        Ok((ptcl, sender.to_string(), buffer.to_vec())) //TODO:vec should be only n_bytes long?
    }

    pub async fn send_id(address: String, id: Uuid, sender: String) -> IOResult<()> {
        let socket = UdpSocket::bind(address).await?;
        let str_id = id.to_string();
        let bytes_id = str_id.as_bytes();
        socket.send_to(&bytes_id, &sender).await?;
        Ok(())
    } 
}

