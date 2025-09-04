use transaction::transaction::Transaction;
use chain::chain::Chain;

pub trait Reply {
    fn as_transaction(&mut self) -> Option<&mut Transaction>;
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

