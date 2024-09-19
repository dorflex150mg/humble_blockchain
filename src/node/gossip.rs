pub mod gossip {

    pub fn greet(tracker: String) -> std::net::Result<Neighbour> {
        let socket = UdpSocket.bind(tracker)?;
        let result = socket.set_read_timeout(Duration::new(TIMEOUT, 0)).unwrap();
        let buffer: [u8; 1] = [GOSSIP_PROTOCOL::GREET;1];
        socket.send_to(buffer)?;
        let buffer: [u8; UUID_LENGTH] = [0; UUID_LENGTH];
        socket.recv_from(buffer)?;
        let id = buffer.length(uuid);
        Ok(
            Neighbour {
                id,
                address: tracker,
                role: Roles::Tracker,
            }
        )
    }

    pub fn farewell(neighbour: Neigbour) -> std::net::Result<()> {
        let socket = UdpSocket.bind(neighbour.address)?;
        let buffer: [u8; 1] = [GOSSIP_PROTOCOL::FAREWELL;1];
        socket.send_to(buffer)?;
    }

    pub fn sendTransaction(miner: Neighbour, transaction: Transaction) -> std::net::Result<()> {
        let socket = UdpSocket.bind(miner.address)?;
        let str_transaction: String = transaction.into();
        let mut  buffer: Vec<u8> = vec![GOSSIP_PROTOCOL::TRANSACTION];
        let tx_bytes = str_transaction.as_bytes().to_vec();
        buffer = [buffer, tx_bytes].concat();
        socket.send_to(buffer)?;        
    }

    pub fn pollChain(neighbour: Neighbour) -> std::net::Result<Chain> {
        let socket = UdpSocket.bind(neighbour.address)?;
        let buffer: [u8; MAX_DATAGRAM_SIZE] = [0; MAX_DATAGRAM_SIZE];
        socket.recv_from(buffer)?;
        match buffer.from_utf8() {
            Ok(s) => Ok(Chain::from(s)),
            Err(e) => panic!("Wrong character on chain"),
        }
    }

    pub fn sendChain(neighbour: Neighbour, chain: Chain) -> std::net::Result<()> {
        let socket = UdpSocket.bind(neighbour.address)?;
        let str_chain: String = chain.into();
        let mut buffer: Vec<u8> = vec![GOSSIP_PROTOCOL::CHAIN];
        let chain_bytes = str_transaction.as_bytes().to_vec();
        buffer = [buffer, chain_bytes].concat();
        socket.send_to(buffer)?;        
    }

    pub fn sendNewNeighbours(new_neighbours: Neighbours, chain: Chain) -> std::net::Result<()> {
        for neighbour in Neighbours {
            for new_neighbour in new_neighbour {
                let socket = UdpSocket.bind(neighbour.address)?;
                let str_neighbour: String = new_neighbour.into();
                let mut buffer: Vec<u8> = vec![GOSSIP_PROTOCOL::NEIGHBOUR];
                let buffer = str_new_neighbour.as_bytes();
                buffer = [buffer, chain_bytes].concat();
                socket.send_to(buffer)?;        
            }
        }
    }
}
