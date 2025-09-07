use wallet::transaction::record::Record;
use wallet::transaction::transaction::Transaction;

pub const TRANSACTION_BLOCK_MEMBER_IDENTIFIER: u8 = 0;
pub const RECORD_BLOCK_MEMBER_IDENTIFIER: u8 = 1;

pub trait BlockEntry: Into<String> {}
impl BlockEntry for Transaction {}
impl BlockEntry for Record {}
