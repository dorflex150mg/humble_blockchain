use wallet::transaction::record::Record;
use wallet::transaction::transaction::Transaction;

/// Identifier for transaction block members.
/// This constant is used to identify entries as transactions within a block.
pub const TRANSACTION_BLOCK_MEMBER_IDENTIFIER: u8 = 0;

/// Identifier for record block members.
/// This constant is used to identify entries as records within a block.
pub const RECORD_BLOCK_MEMBER_IDENTIFIER: u8 = 1;

/// Trait representing an entry in a block.
/// This trait is implemented by types that can be converted into a string representation for storage in a block.
pub trait BlockEntry: Into<String> {}

impl BlockEntry for Transaction {}
impl BlockEntry for Record {}
