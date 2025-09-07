pub mod wallet;
pub mod transaction {
    pub mod block_entry_common;
    pub mod record;
    #[allow(clippy::module_inception)]
    pub mod transaction;
}
pub mod token;
