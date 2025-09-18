use crate::transaction::record::Record;
use crate::transaction::transaction::Transaction;

/// Dependency inversion trait that represents a Block in a Chain.
pub trait BlockChainBlock {
    /// Returns the `[BlockChainBlock]`'s data section.
    fn get_data(&self) -> &str;

    /// Returns the `[BlockChainBlock]`'s hash field.
    fn get_hash(&self) -> &str;

    /// Filters the `[BlockChainBlock]`'s data and returns its `[Record]` entries.
    fn get_records(&self) -> Vec<Record>;

    /// Filters the `[BlockChainBlock]`'s data and returns its `[Transaction]` entries.
    fn get_transactions(&self) -> Vec<Transaction>;

    /// Returns the `[BlockChainBlock]`'s `previous_hash` field, that represents the hash of the
    /// previous block in `[BlockChain]`.
    fn get_previous_hash(&self) -> &str;
}

/// Dependency inversion trait that represents a Chain.
pub trait BlockChain {
    /// Returns the `[BlockChain]`'s last `[BlockChainBlock]`.
    fn get_last_block(&self) -> &dyn BlockChainBlock;
    /// Returns the `[BlockChain]`'s `[BlockChainBlock]`s.
    fn get_blocks(&self) -> Vec<Box<dyn BlockChainBlock>>;
}
