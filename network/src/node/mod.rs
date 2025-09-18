/// Contains gossip protocol functions.
pub mod gossip;
/// Contains the `[Neighbour]` struct.
pub mod neighbour;
#[allow(clippy::module_inception)]
/// Contains the `[Node]` struct.
pub mod node;
/// Contains the gossip protocol message-byte pairing.
pub mod protocol;
/// Custom receiver type that wraps a `mspc::Receiver<String>`.
pub mod receiver;
/// Contains the `[Reply]` trait that create trait objects for datastructure that get sent through
/// the gossip protocol.
pub mod reply;
/// Contain the `[Theme]` enum that classifies gossip message types.
pub mod theme;
