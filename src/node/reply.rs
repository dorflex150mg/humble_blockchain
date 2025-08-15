use crate::Transaction;
use crate::Chain;

pub trait Reply {
    fn as_transaction(&mut self) -> Option<&mut Transaction>;
    fn as_chain(&mut self) -> Option<&mut Chain>;
}
