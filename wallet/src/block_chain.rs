use crate::transaction::record::Record;
use crate::transaction::transaction::Transaction;

pub trait BlockChainBlock {
    fn get_data(&self) -> &str;

    fn get_hash(&self) -> &str;

    fn get_records(&self) -> Vec<Record>;

    fn get_transactions(&self) -> Vec<Transaction>;

    fn get_previous_hash(&self) -> &str;
}

pub trait BlockChain {
    fn get_last_block(&self) -> &dyn BlockChainBlock;
    fn get_blocks(&self) -> Vec<Box<dyn BlockChainBlock>>;
}
