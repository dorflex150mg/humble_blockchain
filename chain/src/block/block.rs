use crate::block::block_entry::{
    RECORD_BLOCK_MEMBER_IDENTIFIER, TRANSACTION_BLOCK_MEMBER_IDENTIFIER,
};
use wallet::token::Token;
use wallet::token::TokenConversionError;
use wallet::token::TOKEN_SIZE;
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

pub const MAX_TRANSACTIONS: usize = 8;

pub const FIELD_END: char = ';';

pub const N_RECORD_FIELDS: usize = 3;

pub const HASH_SIZE: usize = 64;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Hash(String);

#[derive(Debug, Error)]
pub enum HashError {
    #[error("Hash Strings must have ascii encoding.")]
    InvalidHashStringhError,
    #[error("Hash Strings must have exactly size {}", HASH_SIZE)]
    WrongSizeHashError,
}

#[allow(clippy::unwrap_used)] // Token is guaranteed to have valid content.
impl From<Token> for Hash {
    fn from(value: Token) -> Self {
        Hash(str::from_utf8((*value).as_slice()).unwrap().to_owned())
    }
}

#[allow(clippy::unwrap_used)] // Hash is guaranteed to have the correct size.
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

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    pub index: usize,
    pub previous_hash: Hash,
    pub hash: Hash,
    pub data: String,
    pub timestamp: u64,
    pub nonce: u64,
}

#[derive(Error, Debug)]
pub enum InvalidTransactionErr {
    IncompleteChain,
    UnknownCoin,
}

impl fmt::Display for InvalidTransactionErr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IncompleteChain => write!(
                f,
                "The last owner of this coin is not this transaction's spender."
            ),
            Self::UnknownCoin => write!(f, "The coin spent in this transaction is not valid."),
        }
    }
}

pub fn check_transaction(
    block_member: Transaction,
    blocks: &[Block],
) -> Result<Transaction, InvalidTransactionErr> {
    let coins = &block_member.coins;
    for coin in coins {
        //verify each coin is valid:
        let mut coin_found = false;
        for block in blocks.iter().rev() {
            //check each block
            for t in get_block_entries!(block, Transaction) {
                //check each transaction in the block
                if t.coins[0] == *coin {
                    coin_found = true; //if the coin gets found, check if the spender is
                                       //the last owner of the coin
                    if t.receiver_wallet != block_member.sender_wallet {
                        // fail if sender doesnt own the
                        // coin
                        return Err(InvalidTransactionErr::IncompleteChain);
                    }
                    break;
                }
            }
        }
        if !coin_found {
            // if the coin is not in any blocks, fail
            return Err(InvalidTransactionErr::UnknownCoin);
        }
    }
    Ok(block_member)
}

impl Block {
    #[allow(clippy::unwrap_used)]
    pub fn new(index: usize, previous_hash: Hash, data: String, hash: Option<Hash>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let private_hash = hash.unwrap_or_default();
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
        let mut string_entry = String::new();
        let mut current_char = iter.next()?;
        string_entry.push(current_char);
        let mut separator_count = 0;
        let item_field_count = match current_char as u8 {
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

    pub fn get_hash(&self) -> Hash {
        self.hash.clone()
    }

    #[allow(clippy::uninlined_format_args, clippy::unwrap_used)]
    pub fn calculate_hash(&self) -> Hash {
        let str_block = format!(
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
