use crate::block::block_entry::{
    RECORD_BLOCK_MEMBER_IDENTIFIER, TRANSACTION_BLOCK_MEMBER_IDENTIFIER,
};
use wallet::token::Token;
use wallet::token::TOKEN_SIZE;
use wallet::transaction::record::N_RECORD_FIELDS;
use wallet::transaction::transaction::Transaction;
use wallet::transaction::transaction::N_TRANSACTION_FIELDS;

use std::fmt;
use std::iter::Peekable;
use std::ops::Deref;
use std::str::Chars;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use static_assertions::assert_impl_all;

/// Maximum amount of transactions one block can carry.
pub const MAX_TRANSACTIONS: usize = 128;

/// Separator between fields of `[Transactions]` or `[Records]`.
pub const FIELD_END: char = ';';

/// Size of `Block` hashes.
pub const HASH_SIZE: usize = 64;

/// Represents a hash of a block in the blockchain.
/// This is a wrapper around a string that ensures the string meets certain criteria for being a valid hash.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Hash(String);

/// Errors that can occur when converting to or from a `Hash`.
#[derive(Debug, Error)]
pub enum HashError {
    /// `HashError` variant for encoding failure.
    #[error("Hash Strings must have ascii encoding.")]
    InvalidHashStringhError,
    /// `HashError` variant for wrongly sized param `String`.
    #[error("Hash Strings must have exactly size {}", HASH_SIZE)]
    WrongSizeHashError,
}

#[allow(clippy::unwrap_used)] // Token is guaranteed to have valid content.
impl From<Token> for Hash {
    fn from(value: Token) -> Self {
        Hash(str::from_utf8((*value).as_slice()).unwrap().to_owned())
    }
}

#[allow(clippy::unwrap_used, clippy::from_over_into)] // Hash is guaranteed to have the correct size.
impl Into<Token> for Hash {
    fn into(self) -> Token {
        let array: [u8; TOKEN_SIZE] = self.0.as_bytes().try_into().unwrap();
        Token::new(array)
    }
}

impl TryFrom<String> for Hash {
    type Error = HashError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.len() != HASH_SIZE {
            return Err(HashError::WrongSizeHashError);
        }
        if !value.is_ascii() {
            return Err(HashError::InvalidHashStringhError);
        }
        Ok(Self(value))
    }
}

impl Default for Hash {
    fn default() -> Self {
        Self("0".repeat(HASH_SIZE))
    }
}

impl Deref for Hash {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

macro_rules! get_block_entries {
    ($block: ident, $type_name: ty) => {{
        assert_impl_all!($type_name: Into<String>);
        let mut block_entries: Vec<$type_name> = vec![];
        let mut iter = $block.data.chars().peekable();
        while iter.peek().is_some() {
            if let Some(next_string_entry) = Block::get_next_string_entry(&mut iter) {
                if let Ok(block_entry) = <$type_name>::try_from(next_string_entry) {
                    block_entries.push(block_entry);
                }
            }
        }
        block_entries
    }};
}

/// Represents a block in the blockchain.
/// A block contains a list of transactions, a hash of the previous block, and its own hash.
#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    /// The index of the block in the blockchain.
    /// This is a sequential number indicating the position of the block in the chain.
    pub index: usize,
    /// The hash of the previous block in the blockchain.
    /// This field links the current block to the previous one, ensuring the integrity of the chain.
    pub previous_hash: Hash,
    /// The hash of the current block.
    /// This field is calculated based on the block's contents and ensures data integrity.
    pub hash: Hash,
    /// The data contained in the block.
    /// This typically includes a list of transactions or other relevant information.
    pub data: String,
    /// The timestamp indicating when the block was created.
    /// This is typically represented as the number of seconds since the Unix epoch.
    pub timestamp: u64,
    /// The nonce used in the mining process.
    /// This value is adjusted during mining to achieve a valid hash for the block.
    pub nonce: u64,
}

impl Block {
    #[allow(clippy::unwrap_used)]
    /// Creates a new `Block`.
    ///
    /// # Arguments
    /// * `index` - The index of the block in the blockchain.
    /// * `previous_hash` - The hash of the previous block in the blockchain.
    /// * `data` - The data contained in the block.
    /// * `hash` - Optional hash for the block. If not provided, a default hash is used.
    ///
    /// # Returns
    /// * `Self` - The newly created block.
    pub fn new(index: usize, previous_hash: Hash, data: String, hash: Option<Hash>) -> Self {
        let timestamp: u64 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let private_hash: Hash = hash.unwrap_or_default();
        Self {
            index,
            previous_hash,
            data,
            timestamp,
            hash: private_hash,
            nonce: 0,
        }
    }

    fn get_next_string_entry(iter: &mut Peekable<Chars>) -> Option<String> {
        let mut string_entry: String = String::new();
        let mut current_char: char = iter.next()?;
        string_entry.push(current_char);
        let mut separator_count: usize = 0;
        let item_field_count: usize = match current_char as u8 {
            TRANSACTION_BLOCK_MEMBER_IDENTIFIER => N_TRANSACTION_FIELDS,
            RECORD_BLOCK_MEMBER_IDENTIFIER => N_RECORD_FIELDS,
            _ => return None,
        };
        while separator_count != item_field_count {
            current_char = iter.next()?;
            if current_char == FIELD_END {
                separator_count += 1;
            }
            string_entry.push(current_char);
        }
        Some(string_entry)
    }

    /// Retrieves all transactions contained in this block.
    ///
    /// # Returns
    /// * `Vec<Transaction>` - A vector of transactions contained in the block.
    pub fn get_transactions(&self) -> Vec<Transaction> {
        let mut transactions: Vec<Transaction> = vec![];
        let mut iter = self.data.chars().peekable();
        while iter.peek().is_some() {
            if let Some(next_string_entry) = Self::get_next_string_entry(&mut iter) {
                if let Ok(transaction) = Transaction::try_from(next_string_entry) {
                    transactions.push(transaction);
                }
            }
        }
        transactions
    }

    /// Retrieves the hash of this block.
    ///
    /// # Returns
    /// * `Hash` - The hash of the block.
    pub fn get_hash(&self) -> Hash {
        self.hash.clone()
    }

    #[allow(clippy::uninlined_format_args, clippy::unwrap_used)]
    /// Calculates the hash of this block based on its contents.
    ///
    /// # Returns
    /// * `Hash` - The calculated hash of the block.
    pub fn calculate_hash(&self) -> Hash {
        let str_block: String = format!(
            "{}{}{}{}{}{}",
            self.hash, self.previous_hash, self.data, self.timestamp, self.index, self.nonce,
        );
        let mut hasher = Sha256::new();
        hasher.update(str_block);
        let digest = hasher.finalize();
        Hash::try_from(format!("{:x}", digest)).unwrap() //guaranteed to work.
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Block(index: {}, previous hash: {}, hash: {}, timestamp: {})",
            self.index, self.previous_hash, self.hash, self.timestamp
        )
    }
}
