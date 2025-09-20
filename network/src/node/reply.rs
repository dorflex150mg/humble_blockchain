use std::ops::{Deref, DerefMut};

use chain::chain::Chain;
use wallet::transaction::{
    block_entry_common::BlockEntry, record::Record, transaction::Transaction,
};

/// Holds either a `[Transaction]` or a `[Record]`.
pub enum BlockEntryReply {
    /// Transaction variant.
    Transaction(Transaction),
    /// Record variant.
    Record(Record),
}

/// Trait to wrap datastructure to be sent through the gossip protocol as a trait object.
pub trait Reply: Send {
    /// Unwraps into a `[Transaction]`.
    fn as_sign(&mut self) -> Option<Box<dyn BlockEntry>>;
    /// Unwraps into a `[Chain]`.
    fn as_chain(&mut self) -> Option<&mut Chain>;
}

/// Implementation of the `Reply` trait for the `Chain` struct, allowing it to be used in message replies.
impl Reply for Chain {
    /// Converts the chain to a transaction, which is not applicable here.
    ///
    /// # Returns
    /// None, as a chain is not a transaction.
    fn as_sign(&mut self) -> Option<Box<dyn BlockEntry>> {
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

/// Wrapper trait for `BlockEntry`. Avoids overlapping a blanket
/// implementation for all `BlockEntry` types with the implementation
/// for `Chain`.
pub struct ReplySign<T: BlockEntry>(pub(crate) Box<T>);

impl<T: BlockEntry> Reply for ReplySign<T> {
    // With the ?Sized bound, ReplySign can wrap around
    // a trait object.
    fn as_sign(&mut self) -> Option<Box<dyn BlockEntry>> {
        Some(self.clone_box())
    }

    fn as_chain(&mut self) -> Option<&mut Chain> {
        None
    }
}

impl<T: BlockEntry> Deref for ReplySign<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: BlockEntry> DerefMut for ReplySign<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
