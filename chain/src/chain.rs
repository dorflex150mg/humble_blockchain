use crate::block::block::{Block, Hash, RecordOffset};
use crate::miner::miner::MiningDigest;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    fmt,
};
use tracing::debug;
use uuid::Uuid;
use wallet::block_chain::BlockChainBlock;

/// The interval (in seconds) to check for increasing difficulty. Difficulty increases if mining a block takes more than this interval.
const INTERVAL: u64 = 60;

/// Struct representing a blockchain with a vector of blocks, length, and mining difficulty.
#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Chain {
    index: HashMap<String, usize>,
    last_block_offset: usize,
    id: Uuid,
    blocks: Vec<Block>,
    len: usize,
    /// Current mining difficulty (number of leading zeros required). `difficulty` should  never surpass 256, hence
    /// the type.
    pub difficulty: u8,
}

impl PartialEq for Chain {
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len()
    }
}

impl Eq for Chain {}

impl PartialOrd for Chain {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Chain {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.len.cmp(&other.len())
    }
}

impl fmt::Display for Chain {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str_blocks: String = self
            .blocks
            .iter()
            .map(std::string::ToString::to_string)
            .collect::<Vec<String>>()
            .join(" | ");
        write!(
            f,
            "Chain[len: {}, difficulty: {}, {}]",
            self.len, self.difficulty, str_blocks
        )
    }
}

/// Enum representing possible errors when validating a block in the chain.
#[derive(Debug)]
pub enum BlockCheckError {
    /// Error for when the block's index doesn't match the expected chain index.
    WrongIndex(usize, usize),
    /// Error for when the block's hash does not satisfy the current difficulty level.
    InvalidPrefix(u8),
    /// Error for when the previous block's hash is not found in the chain.
    NotInChain {
        /// Expected previous hash.
        expected: String,
        /// Actual previous hash.
        got: String,
    },
    /// Error for when the block's hash does not match the expected hash.
    WrongHash {
        /// Expected block hash.
        expected: String,
        /// Actual block hash.
        got: String,
    },
}

impl fmt::Display for BlockCheckError {
    /// Formats error messages for `BlockCheckError` to be user-friendly.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockCheckError::WrongIndex(expected, got) => write!(
                f,
                "Wrong index. Expected index {expected}, but the mined block index was {got}",
            ),
            BlockCheckError::InvalidPrefix(difficulty) => write!(
                f,
                "Invalid prefix - Not enough \"0\"s at the beginning. Current difficulty: {difficulty}",
            ),
            BlockCheckError::NotInChain { expected, got } => write!(
                f,
                "Previous hash not in chain. Expected: {expected}, but got: {got}",
            ),
            BlockCheckError::WrongHash { expected, got } => {
                write!(f, "Wrong hash. Expected: {expected}, but got: {got}")
            }
        }
    }
}

impl Chain {
    /// Creates a new blockchain with a single genesis block.
    ///
    /// # Returns
    /// A new instance of `Chain`.
    #[allow(clippy::unwrap_used)]
    #[must_use]
    pub fn new() -> Self {
        let genesis_block = Block::new(0, Hash::default(), String::new(), Some(Hash::default()));
        let mut chain = Chain {
            index: HashMap::new(),
            last_block_offset: 0,
            id: Uuid::new_v4(),
            blocks: vec![],
            len: 0,
            difficulty: 1,
        };
        let genesis_mining_digest = MiningDigest::new(vec![], genesis_block, 0);
        #[allow(clippy::unwrap_used)]
        chain.add_block(genesis_mining_digest).unwrap();
        chain
    }

    /// Returns the current length of the chain.
    ///
    /// # Returns
    /// The number of blocks in the chain.
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }

    /// Checks if the chain is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Verifies the validity of a block based on its data, previous hash, and current difficulty.
    ///
    ///
    /// # Arguments
    /// * `data` - The block data.
    /// * `previous_hash` - Hash of the previous block.
    /// * `block_hash` - Hash of the current block.
    /// * `block_index` - Index of the current block.
    ///
    /// # Returns
    /// A `Result` which is `Ok` if the block is valid or contains a `BlockCheckError` if invalid.
    pub fn check_block_data(
        &self,
        data: String,
        previous_hash: &String,
        block_hash: &String,
        block_index: usize,
    ) -> Result<(), BlockCheckError> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();
        let digest_str = format!("{digest:x}");

        if block_index != self.len + 1 {
            return Err(BlockCheckError::WrongIndex(self.len + 1, block_index));
        }
        if !digest_str.starts_with(&"0".repeat(self.difficulty as usize)) {
            return Err(BlockCheckError::InvalidPrefix(self.difficulty));
        }
        let last_chain_hash = self.get_last_block().hash.clone();
        if *previous_hash != *last_chain_hash {
            return Err(BlockCheckError::NotInChain {
                expected: previous_hash.to_string(),
                got: last_chain_hash.to_string(),
            });
        }
        if digest_str != *block_hash {
            return Err(BlockCheckError::WrongHash {
                expected: digest_str,
                got: block_hash.to_string(),
            });
        }
        debug!("Block successfully validated!");
        Ok(())
    }

    /// Adjusts the difficulty level based on the block's timestamp. If the time taken is less than the interval, difficulty is increased.
    ///
    /// # Arguments
    /// * `block_timestamp` - The timestamp of the block being checked.
    fn check_difficulty(&mut self, block_timestamp: u64) {
        if block_timestamp < self.get_last_block().timestamp + INTERVAL {
            self.difficulty += 1;
            debug!("Difficulty increased: {}", self.difficulty);
        }
    }

    /// Retrieves the last block in the chain.
    ///
    /// # Returns
    /// The last `Block` in the chain.
    #[allow(clippy::unwrap_used)]
    #[must_use]
    pub fn get_last_block(&self) -> Block {
        self.blocks.iter().last().unwrap().clone() // It is impossible to have a chain with 0 blocks.
    }

    /// Updates the index.
    #[allow(clippy::unwrap_used)]
    fn update_index(&mut self, offsets: &Vec<RecordOffset>) {
        let modified_keys: Vec<String> = offsets.iter().map(RecordOffset::get_key).collect();
        let new_index: HashMap<String, usize> = self
            .index
            .iter()
            .filter(|(key, _)| modified_keys.contains(key))
            .map(|(key, _)| {
                (
                    key.clone(),
                    offsets
                        .iter()
                        .find(|e| e.get_key() == *key)
                        .unwrap()
                        .get_offset()
                        + self.last_block_offset,
                )
            })
            .collect();
        self.index = new_index;
    }

    /// Adds a new block to the chain after validating its data, hash, and index.
    ///
    /// # Arguments
    /// * `block` - The new `Block` to be added.
    /// * `nonce` - The nonce used during mining.
    ///
    /// # Returns
    /// A `Result` which is `Ok` if the block is added successfully or contains a `BlockCheckError` if the block is invalid.
    pub fn add_block(&mut self, mining_digest: MiningDigest) -> Result<(), BlockCheckError> {
        let block = mining_digest.get_block();
        let nonce = mining_digest.get_nonce();
        if block.index != 0 {
            let last_block = self.get_last_block();
            let str_block = format!(
                "{}{}{}{}{}{}",
                last_block.hash,
                last_block.previous_hash,
                last_block.data,
                last_block.timestamp,
                last_block.index,
                nonce, // Include the mined nonce
            );
            let data = str_block.clone();
            let previous_hash = &block.previous_hash;
            let block_hash = &block.hash;
            let block_index = block.index;
            self.check_block_data(data, previous_hash, block_hash, block_index)?;
            self.check_difficulty(block.timestamp);
        }
        self.blocks.push(block);
        self.len += 1;
        self.update_index(&mining_digest.get_record_offsets());
        Ok(())
    }

    /// Returns the length of the chain (number of blocks).
    #[must_use]
    pub fn get_len(&self) -> usize {
        self.len
    }

    /// Prints details of the last block in the chain.
    pub fn print_last_block(&self) {
        println!("{}", self.get_last_block());
    }

    /// Retrieves all the blocks in the chain.
    ///
    /// # Returns
    /// A vector of `Block`s.
    #[must_use]
    pub fn get_blocks(&self) -> Vec<Block> {
        self.blocks.clone()
    }

    /// Searches for a key and returns the last record that contains it.
    #[must_use]
    pub fn search(&self, key: &str) -> Option<Vec<u8>> {
        for block in &self.blocks {
            if let Some(record_match) = block
                .get_records()
                .iter()
                .rev()
                .find(|r| r.get_key() == key)
            {
                if record_match.tombstone() {
                    return None;
                }
                return Some(record_match.get_value());
            }
        }
        None
    }
}

impl Default for Chain {
    fn default() -> Self {
        Chain::new()
    }
}
