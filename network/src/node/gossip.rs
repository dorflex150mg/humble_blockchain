use crate::node::neighbour::{Neighbour, Role};
use crate::node::protocol;

use chain::chain::Chain;
use wallet::transaction::transaction::Transaction;

use std::{
    io::{Error as IOError, Result as IOResult},
    str,
    sync::Arc,
    time::Duration,
};

use thiserror::Error;
use tokio::{io::ErrorKind, net::UdpSocket, time::timeout};
use tracing::debug;
use uuid::Uuid;

use std::boxed::Box;

// Constants
/// Back-off for gossip messages in seconds.
pub const GOSSIP_INTERVAL: u64 = 3;
/// Length of UUID string representation.
pub const UUID_LENGTH: usize = 36;
/// Max size of a UDP datagram.
pub const MAX_DATAGRAM_SIZE: usize = 65507;

/// Enum to represent potential errors in the gossip protocol.
#[derive(Error, Debug, derive_more::From)]
pub enum GossipError {
    #[error(transparent)]
    /// IO Error.
    IOError(IOError),
    #[error("Attempted to read and got would block.")]
    /// Would block.
    WouldBlock(ErrorKind),
    /// Failed to decode the reply.
    #[error("InvalidReplyError")]
    InvalidReplyError,
}

/// Represents the reply to a Gossip message.
pub struct GossipReply {
    /// Identifiy the kind of replu.
    pub protocol: u8,
    /// Sender's ip.
    pub sender: String,
    /// Contents of the reply.
    pub buffer: Vec<u8>,
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
#[allow(clippy::unwrap_used, clippy::single_match_else)]
pub async fn greet(
    address: Arc<str>,
    id: Uuid,
    role: Role,
    tracker: &str,
) -> Result<Neighbour, GossipError> {
    let socket = UdpSocket::bind(address.as_ref()).await?;
    let greeter = Neighbour {
        id,
        address: (*address.clone()).to_owned(),
        role,
    };
    let neighbour_str: String = serde_json::to_string(&greeter).unwrap();
    let mut buffer = vec![protocol::GREET];
    buffer.extend_from_slice(neighbour_str.as_bytes());

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

    let str_id = str::from_utf8(&buffer_recv).map_err(|_| GossipError::InvalidReplyError)?;
    println!("New neighbour connected: {}", &str_id);

    Ok(Neighbour {
        id: Uuid::parse_str(str_id).map_err(|_| GossipError::InvalidReplyError)?,
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
pub async fn send_transaction(
    address: Arc<str>,
    miner: String,
    transaction: Transaction,
) -> IOResult<()> {
    let socket = UdpSocket::bind(address.as_ref()).await?;
    let str_transaction: String = transaction.into();
    let mut buffer = vec![protocol::TRANSACTION];
    buffer.extend_from_slice(str_transaction.as_bytes());
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
pub async fn poll_chain(address: Arc<str>, neighbour: &Neighbour) -> Result<Chain, GossipError> {
    let socket = UdpSocket::bind(address.as_ref()).await?;
    let buffer = [protocol::POLLCHAIN];
    socket.send_to(&buffer, &neighbour.address).await?;

    let mut recv_buffer: Box<[u8]> = vec![0; MAX_DATAGRAM_SIZE].into_boxed_slice();
    socket.recv_from(&mut recv_buffer).await?;

    let chain_str = str::from_utf8(&recv_buffer).map_err(|_| GossipError::InvalidReplyError)?;
    serde_json::from_str(chain_str).map_err(|_| GossipError::InvalidReplyError)
}

/// Sends a copy of the blockchain to a specified neighbour.
///
/// # Arguments
/// * `address` - The address to bind the local UDP socket.
/// * `neighbour` - The address of the neighbour to send the chain to.
/// * `chain` - The blockchain to be sent.
pub async fn send_chain(
    address: Arc<str>,
    neighbour: String,
    chain: Chain,
) -> Result<(), GossipError> {
    let socket = UdpSocket::bind(address.as_ref()).await?;
    let str_chain = serde_json::to_string(&chain).map_err(|_| GossipError::InvalidReplyError)?;
    let mut buffer = vec![protocol::CHAIN];
    buffer.extend_from_slice(str_chain.as_bytes());
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
) -> Result<(), GossipError> {
    for new_neighbour in new_neighbours {
        if new_neighbour.id == neighbour_id {
            continue;
        }

        debug!("Sending neighbour {} to {}", new_neighbour.id, neighbour_id);

        let socket = UdpSocket::bind(address.as_ref()).await?;
        let str_neighbour =
            serde_json::to_string(&new_neighbour).map_err(|_| GossipError::InvalidReplyError)?;
        let mut buffer = vec![protocol::NEIGHBOUR];
        buffer.extend_from_slice(str_neighbour.as_bytes());

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
#[allow(clippy::manual_let_else, clippy::single_match_else)]
pub async fn listen_to_gossip(address: Arc<str>) -> Result<Option<GossipReply>, GossipError> {
    let socket = UdpSocket::bind(address.as_ref()).await?;
    let mut buffer: Box<[u8]> = vec![0; MAX_DATAGRAM_SIZE].into_boxed_slice();

    println!("[{address}] Listening for gossip...");

    let (n_bytes, sender) = match timeout(Duration::new(3, 0), socket.recv_from(&mut buffer)).await
    {
        Ok(Ok((n_bytes, sender))) => (n_bytes, sender),
        _ => {
            println!("Got nothing here");
            return Ok(None);
        }
    };

    let protocol_type = buffer[0];
    debug!("Received protocol: {}", protocol_type);
    let reply = GossipReply {
        protocol: protocol_type,
        sender: sender.to_string(),
        buffer: buffer[..n_bytes].to_vec(),
    };
    Ok(Some(reply))
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
