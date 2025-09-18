use chain::chain::Chain;
use wallet::transaction::transaction::Transaction;

/// Trait to wrap datastructure to be sent through the gossip protocol as a trait object.
pub trait Reply {
    /// Unwraps into a `[Transaction]`.
    fn as_transaction(&mut self) -> Option<&mut Transaction>;
    /// Unwraps into a `[Chain]`.
    fn as_chain(&mut self) -> Option<&mut Chain>;
}

/// Implementation of the `Reply` trait for the `Chain` struct, allowing it to be used in message replies.
impl Reply for Chain {
    /// Converts the chain to a transaction, which is not applicable here.
    ///
    /// # Returns
    /// None, as a chain is not a transaction.
    fn as_transaction(&mut self) -> Option<&mut Transaction> {
        None
    }
    /// Converts the chain into a mutable reference to itself.
    ///
    /// # Returns
    /// A mutable reference to the chain.
    fn as_chain(&mut self) -> Option<&mut Chain> {
        Some(self)
    }
}

impl Reply for Transaction {
    fn as_transaction(&mut self) -> Option<&mut Transaction> {
        Some(self)
    }
    fn as_chain(&mut self) -> Option<&mut Chain> {
        None
    }
}
