pub mod wallet;
pub mod transaction {
    #[allow(clippy::module_inception)]
    pub mod transaction;
    pub mod record;
    pub mod block_entry_common;
}
