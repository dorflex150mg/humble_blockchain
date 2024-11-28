pub mod gossip {
    use crate::{Chain, Transaction};
    use crate::node::neighbour::neighbour::{Neighbour, Role};
    use crate::node::protocol::protocol;

    use std::{
        io::{Result as IOResult, Error as IOError},
        sync::Arc,
        time::Duration,
        str,
    };

    use tokio::{
        net::UdpSocket,
        time::timeout,
        io::ErrorKind,
    };
    use uuid::Uuid;
    use thiserror::Error;
    use tracing::debug;

    // Constants
    pub const GOSSIP_INTERVAL: u64 = 3;
    pub const UUID_LENGTH: usize = 36;
    pub const MAX_DATAGRAM_SIZE: usize = 65507;

    /// Enum to represent potential errors in the gossip protocol.
    #[derive(Error, Debug, derive_more::From)]
    pub enum GossipError {
        #[error(transparent)]
        IOError(IOError),
        #[error("Attempted to read and got would block.")]
        WouldBlock(ErrorKind),
    }

    /// Sends a greeting message to a tracker to introduce a new neighbour.
    ///
    /// # Arguments
    /// * `address` - The address to bind the local UDP socket.
    /// * `id` - The UUID of the new neighbour.
    /// * `role` - The role of the neighbour (e.g., Tracker, Node).
    /// * `tracker` - The address of the tracker to send the greeting to.
    ///
    /// # Returns
    /// * `IOResult<Neighbour>` - The tracker as a `Neighbour` instance.
    pub async fn greet(address: Arc<str>, id: Uuid, role: Role, tracker: &str) -> IOResult<Neighbour> {
        let socket = UdpSocket::bind(address.as_ref()).await?;
        let greeter = Neighbour { 
            id, 
            address: (*address.clone()).to_owned(), 
            role 
        };
        let neighbour_str: String = serde_json::to_string(&greeter).unwrap();
        let mut buffer = vec![protocol::GREET];
        buffer.extend_from_slice(&neighbour_str.as_bytes());

        let mut buffer_recv: [u8; UUID_LENGTH] = [0; UUID_LENGTH];
        let mut retry = true;

        while retry {
            socket.send_to(&buffer, tracker).await?;
            retry = match timeout(Duration::new(1, 0), socket.recv_from(&mut buffer_recv)).await {
                Ok(_) => false,
                Err(_) => {
                    debug!("Retrying recv_from");
                    true
                }
            };
        }

        let str_id = str::from_utf8(&buffer_recv).unwrap();
        debug!("New neighbour connected");

        Ok(Neighbour {
            id: Uuid::parse_str(str_id).unwrap(),
            address: tracker.to_string(),
            role: Role::Tracker,
        })
    }

    /// Sends a farewell message to a neighbour, indicating that it is leaving the network.
    ///
    /// # Arguments
    /// * `address` - The address to bind the local UDP socket.
    /// * `neighbour` - The address of the neighbour to send the farewell to.
    pub async fn farewell(address: Arc<str>, neighbour: String) -> IOResult<()> {
        let socket = UdpSocket::bind(address.as_ref()).await?;
        let buffer = [protocol::FAREWELL];
        socket.send_to(&buffer, &neighbour).await?;
        Ok(())
    }

    /// Sends a transaction to a miner for processing.
    ///
    /// # Arguments
    /// * `address` - The address to bind the local UDP socket.
    /// * `miner` - The address of the miner to send the transaction to.
    /// * `transaction` - The transaction to be sent.
    pub async fn send_transaction(address: Arc<str>, miner: String, transaction: Transaction) -> IOResult<()> {
        let socket = UdpSocket::bind(address.as_ref()).await?;
        let str_transaction: String = transaction.into();
        let mut buffer = vec![protocol::TRANSACTION];
        buffer.extend_from_slice(&str_transaction.as_bytes());
        socket.send_to(&buffer, &miner).await?;
        Ok(())
    }

    /// Requests a copy of the blockchain from a neighbour.
    ///
    /// # Arguments
    /// * `address` - The address to bind the local UDP socket.
    /// * `neighbour` - The neighbour to request the chain from.
    ///
    /// # Returns
    /// * `IOResult<Chain>` - The chain received from the neighbour.
    pub async fn poll_chain(address: Arc<str>, neighbour: &Neighbour) -> IOResult<Chain> {
        let socket = UdpSocket::bind(address.as_ref()).await?;
        let buffer = [protocol::POLLCHAIN];
        socket.send_to(&buffer, &neighbour.address).await?;

        let mut recv_buffer: [u8; MAX_DATAGRAM_SIZE] = [0; MAX_DATAGRAM_SIZE];
        socket.recv_from(&mut recv_buffer).await?;

        let chain_str = str::from_utf8(&recv_buffer).unwrap();
        Ok(serde_json::from_str(&chain_str).unwrap())
    }

    /// Sends a copy of the blockchain to a specified neighbour.
    ///
    /// # Arguments
    /// * `address` - The address to bind the local UDP socket.
    /// * `neighbour` - The address of the neighbour to send the chain to.
    /// * `chain` - The blockchain to be sent.
    pub async fn send_chain(address: Arc<str>, neighbour: String, chain: Chain) -> IOResult<()> {
        let socket = UdpSocket::bind(address.as_ref()).await?;
        let str_chain = serde_json::to_string(&chain).unwrap();
        let mut buffer = vec![protocol::CHAIN];
        buffer.extend_from_slice(&str_chain.as_bytes());
        socket.send_to(&buffer, &neighbour).await?;
        Ok(())
    }

    /// Sends new neighbours information to a specific neighbour.
    ///
    /// # Arguments
    /// * `neighbour_id` - The UUID of the neighbour to send to.
    /// * `neighbour_address` - The address of the neighbour.
    /// * `address` - The local address to bind the socket.
    /// * `new_neighbours` - The list of new neighbours to be sent.
    pub async fn send_new_neighbours(
        neighbour_id: Uuid,
        neighbour_address: String,
        address: Arc<str>,
        new_neighbours: Vec<Neighbour>,
    ) -> IOResult<()> {
        for new_neighbour in new_neighbours {
            if new_neighbour.id == neighbour_id {
                continue;
            }

            debug!("Sending neighbour {} to {}", new_neighbour.id, neighbour_id);

            let socket = UdpSocket::bind(address.as_ref()).await?;
            let str_neighbour = serde_json::to_string(&new_neighbour).unwrap();
            let mut buffer = vec![protocol::NEIGHBOUR];
            buffer.extend_from_slice(&str_neighbour.as_bytes());

            let bytes_sent = socket.send_to(&buffer, &neighbour_address).await?;
            debug!("Sent {} bytes to {}", bytes_sent, neighbour_address);
        }
        Ok(())
    }

    /// Pauses the execution for the duration of the gossip interval.
    pub async fn wait_gossip_interval() {
        tokio::time::sleep(Duration::new(GOSSIP_INTERVAL, 0)).await;
    }

    /// Listens for incoming gossip messages on the specified address.
    ///
    /// # Arguments
    /// * `address` - The address to bind the UDP socket.
    ///
    /// # Returns
    /// * `Result<Option<(u8, String, Vec<u8>)>, GossipError>` - The gossip message protocol, sender, and data.
    pub async fn listen_to_gossip(address: Arc<str>) -> Result<Option<(u8, String, Vec<u8>)>, GossipError> {
        let socket = UdpSocket::bind(address.as_ref()).await?;
        let mut buffer: [u8; MAX_DATAGRAM_SIZE] = [0; MAX_DATAGRAM_SIZE];

        debug!("Listening for gossip...");

        let (n_bytes, sender) = match timeout(Duration::new(3, 0), socket.recv_from(&mut buffer)).await {
            Ok(Ok((n_bytes, sender))) => (n_bytes, sender),
            _ => {
                debug!("Got nothing here");
                return Ok(None);
            },
        };

        let protocol_type = buffer[0];
        debug!("Received protocol: {}", protocol_type);

        Ok(Some((protocol_type, sender.to_string(), buffer[..n_bytes].to_vec())))
    }

    /// Sends the UUID of the current node to the sender of a message.
    ///
    /// # Arguments
    /// * `address` - The address to bind the UDP socket.
    /// * `id` - The UUID to be sent.
    /// * `sender` - The address of the sender to send the UUID to.
    pub async fn send_id(address: Arc<str>, id: Uuid, sender: String) -> IOResult<()> {
        let socket = UdpSocket::bind(address.as_ref()).await?;
        let id_str = id.to_string();
        socket.send_to(id_str.as_bytes(), &sender).await?;
        Ok(())
    }
}

