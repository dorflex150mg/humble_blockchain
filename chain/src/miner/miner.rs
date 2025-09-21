use crate::block::block::{self, Block, Hash};
use crate::chain::Chain;

use wallet::block_chain::BlockChainBlock;
use wallet::token::Token;
use wallet::transaction::block_entry_common::BlockEntry;
use wallet::transaction::transaction::Transaction;
use wallet::wallet::Wallet;

use rand::{self, Rng};
use std::cmp;
use std::fmt;

use thiserror::Error;

/// A zeroed-out wallet public key.
/// This constant is used to represent a zero wallet, often used in transactions involving mining rewards.
pub const ZERO_WALLET_PK: [u8; 64] = [0u8; 64];

/// Metadata about the blockchain.
#[derive(Clone)]
pub struct ChainMeta {
    /// The length of the blockchain.
    pub len: usize,
    /// The current difficulty for mining new blocks.
    pub difficulty: usize,
    /// The list of blocks in the blockchain.
    pub blocks: Vec<Block>,
}

/// A digest of mining information.
/// Contains a block and the nonce used to mine it.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MiningDigest {
    block: Block,
    nonce: u64,
}

impl MiningDigest {
    /// Creates a new `MiningDigest`.
    ///
    /// # Arguments
    /// * `block` - The block that was mined.
    /// * `nonce` - The nonce used to mine the block.
    ///
    /// # Returns
    /// * `Self` - The newly created `MiningDigest`.
    #[must_use]
    pub fn new(block: Block, nonce: u64) -> Self {
        MiningDigest { block, nonce }
    }

    /// Retrieves the block from the mining digest.
    ///
    /// # Returns
    /// * `Block` - The block that was mined.
    #[must_use]
    pub fn get_block(&self) -> Block {
        self.block.clone()
    }

    /// Retrieves the nonce from the mining digest.
    ///
    /// # Returns
    /// * `u64` - The nonce used to mine the block.
    #[must_use]
    pub fn get_nonce(&self) -> u64 {
        self.nonce
    }
}

/// Errors that can occur during the mining process.
#[derive(Error, Debug, derive_more::From, derive_more::Display)]
pub enum MiningError {
    /// Indicates an error related to uninitialized chain metadata.
    UninitializedChainMetaErr(UninitializedChainMetaErr),
}

/// Error indicating that the chain metadata has not been initialized.
#[derive(Error, Debug)]
pub struct UninitializedChainMetaErr;

impl fmt::Display for UninitializedChainMetaErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "This miner's ChainMeta has not been initialized. Consider set_chain_meta(len, last_hash, difficulty)")
    }
}

/// A miner in the blockchain network.
/// Responsible for mining new blocks and managing transactions.
pub struct Miner {
    id: u64,
    name: String,
    /// The `[Miner]`'s `[Wallet]`. Newly mined `[Token]`s are added here.
    pub wallet: Wallet,
    /// `[Transaction]` buffer to insert at a `[Block]`.
    pub entries: Vec<Box<dyn BlockEntry>>,
    /// `[Chain]` to which this miner submits newly mined `[Block]`s.
    pub chain: Chain,
}

impl Miner {
    /// Creates a new `Miner`.
    ///
    /// # Arguments
    /// * `id` - The unique identifier for the miner.
    /// * `name` - The name of the miner.
    /// * `chain_meta` - The metadata about the blockchain.
    ///
    /// # Returns
    /// * `Self` - The newly created miner.
    #[must_use]
    pub fn new(id: u64, name: String, chain: Chain) -> Self {
        Miner {
            id,
            name,
            wallet: Wallet::new(),
            entries: vec![],
            chain,
        }
    }

    /// Retrieves the name of the miner.
    ///
    /// # Returns
    /// * `String` - The name of the miner.
    #[allow(dead_code)]
    #[must_use]
    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    /// Mines a new block.
    ///
    /// # Arguments
    /// * `block` - The block to be mined.
    ///
    /// # Returns
    /// * `Result<MiningDigest, MiningError>` - The result of the mining operation.
    pub fn mine(&mut self, mut block: Block) -> Result<MiningDigest, MiningError> {
        self.filter_entries()?;
        loop {
            let mut rng = rand::thread_rng();
            block.nonce = rng.gen_range(0..=u64::MAX);
            let str_digest: Hash = block.calculate_hash();
            if str_digest.starts_with(&"0".repeat(self.chain.difficulty as usize)) {
                let token: Token = str_digest.clone().into();
                let prize_transaction = Transaction::new(
                    ZERO_WALLET_PK.to_vec(),
                    self.wallet.get_pub_key(),
                    vec![token],
                );
                let signed_prize = self.wallet.sign(prize_transaction);
                self.entries
                    .push(Box::new(signed_prize) as Box<dyn BlockEntry>); //TODO: this should be the 1st tx
                return Ok(MiningDigest::new(
                    self.create_new_block(str_digest, block.hash.clone()),
                    block.nonce,
                ));
            }
        }
    }

    /// Sets the chain metadata for the miner.
    pub fn set_chain_meta(&mut self, chain: Chain) {
        self.chain = chain;
    }

    /// Adds a new transaction to the miner's list of transactions.
    pub fn push_entry(&mut self, entry: Box<dyn BlockEntry>) {
        self.entries.push(entry);
    }

    /// Checks the validity of the miner's entries and removes the invalid ones.
    ///
    /// # Returns
    /// * `Result<(), MiningError>` - `[MiningError]` when the entry is not correct.
    pub fn filter_entries(&mut self) -> Result<(), MiningError> {
        let filtered: Vec<Box<dyn BlockEntry>> = self
            .entries
            .iter()
            .filter_map(|transaction| {
                let boxed_blocks: Vec<Box<dyn BlockChainBlock>> = self
                    .chain
                    .get_blocks()
                    .iter()
                    .map(|b| Box::new(b.clone()) as Box<dyn BlockChainBlock>)
                    .collect();
                Wallet::check_transaction_tokens(transaction, boxed_blocks.as_slice())
                    .and(Ok(transaction.clone_box()))
                    .ok()
            })
            .collect();
        self.entries = filtered;
        Ok(())
    }

    /// Creates a new block and adds it to the blockchain.
    ///
    /// # Arguments
    /// * `hash` - The hash of the new block.
    /// * `previous_hash` - The hash of the previous block.
    ///
    /// # Returns
    /// * `Block` - The newly created block.
    pub fn create_new_block(&mut self, hash: Hash, previous_hash: Hash) -> Block {
        let index: usize = self.chain.get_len() + 1;
        let cap: usize = cmp::min(self.entries.len(), block::MAX_TRANSACTIONS);
        let capped_entries: Vec<Box<dyn BlockEntry>> = self.entries.drain(0..cap).collect();
        let encoded_entries: Vec<String> = capped_entries
            .iter()
            .map(|entry| entry.clone_box().to_string())
            .collect();
        let data: String = encoded_entries.join("");
        self.wallet.add_coin(hash.clone().into());

        Block::new(index, previous_hash, data, Some(hash))
    }
}

impl fmt::Display for Miner {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let joint_coins: String = self
            .wallet
            .coins
            .iter()
            .map(|coin| {
                (*coin)
                    .clone()
                    .try_into()
                    .unwrap_or("***Invalid Coin***".to_owned())
            })
            .collect::<Vec<String>>()
            .join(",");

        write!(
            f,
            "id: {}, name: {}, coins: {}",
            self.id, self.name, joint_coins
        )
    }
}
