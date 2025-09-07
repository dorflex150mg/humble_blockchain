use crate::block::block::Block;
use crate::miner::miner::MiningDigest;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    cmp::{Eq, Ord, PartialEq, PartialOrd},
    fmt,
};
use tracing::debug;
use uuid::Uuid;

/// The interval (in seconds) to check for increasing difficulty. Difficulty increases if mining a block takes more than this interval.
const INTERVAL: u64 = 60;

/// Struct representing a blockchain with a vector of blocks, length, and mining difficulty.
#[derive(Clone, Serialize, Deserialize)]
pub struct Chain {
    id: Uuid,
    blocks: Vec<Block>,    // List of blocks in the chain
    len: usize,            // Current length of the chain
    pub difficulty: usize, // Current mining difficulty (number of leading zeros required)
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
            .map(|b| b.to_string())
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
    InvalidPrefix(usize),
    /// Error for when the previous block's hash is not found in the chain.
    NotInChain { expected: String, got: String },
    /// Error for when the block's hash does not match the expected hash.
    WrongHash { expected: String, got: String },
}

impl fmt::Display for BlockCheckError {
    /// Formats error messages for `BlockCheckError` to be user-friendly.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            BlockCheckError::WrongIndex(expected, got) => write!(
                f,
                "Wrong index. Expected index {}, but the mined block index was {}",
                expected, got
            ),
            BlockCheckError::InvalidPrefix(difficulty) => write!(
                f,
                "Invalid prefix - Not enough \"0\"s at the beginning. Current difficulty: {}",
                difficulty
            ),
            BlockCheckError::NotInChain { expected, got } => write!(
                f,
                "Previous hash not in chain. Expected: {}, but got: {}",
                expected, got
            ),
            BlockCheckError::WrongHash { expected, got } => {
                write!(f, "Wrong hash. Expected: {}, but got: {}", expected, got)
            }
        }
    }
}

impl Chain {
    /// Creates a new blockchain with a single genesis block.
    ///
    /// # Returns
    /// A new instance of `Chain`.
    pub fn new() -> Self {
        let genesis_block = Block::new(0, "0".repeat(64), String::from(""), Some("0".repeat(64)));
        let id = Uuid::new_v4();
        let mut chain = Chain {
            id,
            blocks: vec![],
            len: 0,
            difficulty: 1,
        };
        let genesis_mining_digest = MiningDigest::new(genesis_block, 0);
        chain.add_block(genesis_mining_digest).unwrap();
        chain
    }

    /// Returns the current length of the chain.
    ///
    /// # Returns
    /// The number of blocks in the chain.
    pub fn len(&self) -> usize {
        self.len
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
    fn check_block_data(
        &self,
        data: String,
        previous_hash: &String,
        block_hash: &String,
        block_index: usize,
    ) -> Result<(), BlockCheckError> {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let digest = hasher.finalize();
        let digest_str = format!("{:x}", digest);

        if block_index != self.len + 1 {
            return Err(BlockCheckError::WrongIndex(self.len + 1, block_index));
        }
        if !digest_str.starts_with(&"0".repeat(self.difficulty)) {
            return Err(BlockCheckError::InvalidPrefix(self.difficulty));
        }
        let last_chain_hash = self.blocks.last().unwrap().hash.clone();
        if *previous_hash != last_chain_hash {
            return Err(BlockCheckError::NotInChain {
                expected: previous_hash.to_string(),
                got: last_chain_hash,
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
        if block_timestamp < self.blocks.iter().last().unwrap().timestamp + INTERVAL {
            self.difficulty += 1;
            debug!("Difficulty increased: {}", self.difficulty);
        }
    }

    /// Retrieves the last block in the chain.
    ///
    /// # Returns
    /// The last `Block` in the chain.
    pub fn get_last_block(&self) -> Block {
        self.blocks.iter().last().unwrap().clone() // It is impossible to have a chain with 0 blocks.
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
            let last_block = self.blocks.iter().last().unwrap();
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
        Ok(())
    }

    /// Returns the length of the chain (number of blocks).
    pub fn get_len(&self) -> usize {
        self.len
    }

    /// Prints details of the last block in the chain.
    pub fn print_last_block(&self) {
        println!("{}", self.blocks.last().unwrap());
    }

    /// Retrieves all the blocks in the chain.
    ///
    /// # Returns
    /// A vector of `Block`s.
    pub fn get_blocks(&self) -> Vec<Block> {
        self.blocks.to_vec() // creates a new vec.
    }
}

impl Default for Chain {
    fn default() -> Self {
        Chain::new()
    }
}
