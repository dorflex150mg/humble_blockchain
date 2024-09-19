pub mod gossip {

    use crate::Chain;
    use crate::Transaction;
    use crate::node::neighbour::neighbour::{
        Neighbour,
        Role,
    };
    use crate::node::protocol::protocol;

    use std::io::Result as IOResult; 
    use std::time::Duration;
    use std::net::UdpSocket;
    use std::str;

    use uuid::Uuid;

    pub const TIMEOUT: u64 = 1;
    pub const UUID_LENGTH: usize = 36;
    pub const MAX_DATAGRAM_SIZE: usize = 65507;

    pub fn greet(tracker: String) -> IOResult<Neighbour> {
        let socket = UdpSocket::bind(tracker.clone())?;
        let result = socket.set_read_timeout(Some(Duration::new(TIMEOUT, 0))).unwrap();
        let buffer: [u8; 1] = [protocol::GREET;1];
        socket.send_to(&buffer, tracker.clone())?;
        let mut buffer: [u8; UUID_LENGTH] = [0; UUID_LENGTH];
        socket.recv_from(&mut buffer)?;
        let str_id = str::from_utf8(&buffer).unwrap();
        Ok(
            Neighbour {
                id: Uuid::parse_str(str_id).unwrap(),
                address: tracker,
                role: Role::Tracker,
            }
        )
    }

    pub fn farewell(neighbour: String) -> IOResult<()> {
        let socket = UdpSocket::bind(&neighbour)?;
        let buffer: [u8; 1] = [protocol::FAREWELL;1];
        socket.send_to(&buffer, &neighbour)?;
        Ok(())
    }

    pub fn sendTransaction(miner: String, transaction: Transaction) -> IOResult<()> {
        let socket = UdpSocket::bind(&miner)?;
        let str_transaction: String = transaction.into();
        let mut  buffer: Vec<u8> = vec![protocol::TRANSACTION];
        let tx_bytes = str_transaction.as_bytes().to_vec();
        buffer = [buffer, tx_bytes].concat();
        socket.send_to(&buffer, &miner)?;        
        Ok(())
    }

    pub fn pollChain(neighbour: &Neighbour) -> IOResult<Chain> {
        let socket = UdpSocket::bind(&neighbour.address)?;
        let mut buffer: [u8; MAX_DATAGRAM_SIZE] = [0; MAX_DATAGRAM_SIZE];
        socket.recv_from(&mut buffer)?;
        match str::from_utf8(&buffer) {
            Ok(s) => Ok(serde_json::from_str(&s).unwrap()),
            Err(e) => panic!("Wrong character on chain"),
        }
    }

    pub fn sendChain(neighbour: String, chain: Chain) -> IOResult<()> {
        let socket = UdpSocket::bind(&neighbour)?;
        let str_chain: String = serde_json::to_string(&chain).unwrap();
        let mut buffer: Vec<u8> = vec![protocol::CHAIN];
        let chain_bytes = str_chain.as_bytes().to_vec();
        buffer = [buffer, chain_bytes].concat();
        socket.send_to(&buffer, &neighbour)?;        
        Ok(())
    }

    pub fn sendNewNeighbours(neighbour: String, new_neighbours: Vec<Neighbour>) -> IOResult<()> {
        for new_neighbour in new_neighbours {
            let socket = UdpSocket::bind(&neighbour)?;
            let str_neighbour: String = serde_json::to_string(&neighbour).unwrap();
            let mut buffer: Vec<u8> = vec![protocol::NEIGHBOUR];
            let neighbour_bytes = str_neighbour.as_bytes().to_vec();
            buffer = [buffer, neighbour_bytes].concat();
            socket.send_to(&buffer, &neighbour)?;        
        }
        Ok(())
    }

    pub fn waitGossipInterval() {}
}
