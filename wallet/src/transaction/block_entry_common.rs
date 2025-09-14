use std::fmt::{Debug, Display};
use std::num::ParseIntError;
use thiserror::Error;

pub const TRANSACTION_BLOCK_MEMBER_IDENTIFIER: u8 = 0;
pub const RECORD_BLOCK_MEMBER_IDENTIFIER: u8 = 1;

#[derive(Debug, Error)]
pub enum BlockIdError {
    #[error("Tried to convert invalid id {} to a BlockMemberId.", {0})]
    InvalidIdError(u8),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum BlockMemberId {
    #[default]
    Transaction,
    Record,
}

impl TryFrom<u8> for BlockMemberId {
    type Error = BlockIdError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            TRANSACTION_BLOCK_MEMBER_IDENTIFIER => Ok(Self::Transaction),
            RECORD_BLOCK_MEMBER_IDENTIFIER => Ok(Self::Record),
            _ => Err(BlockIdError::InvalidIdError(value)),
        }
    }
}

#[allow(clippy::from_over_into)]
impl Into<u8> for BlockMemberId {
    fn into(self) -> u8 {
        match self {
            Self::Transaction => TRANSACTION_BLOCK_MEMBER_IDENTIFIER,
            Self::Record => RECORD_BLOCK_MEMBER_IDENTIFIER,
        }
    }
}

#[derive(Error, Debug, derive_more::From, derive_more::Display)]
pub enum EntryDecodeError {
    Base64Error(base64::DecodeError),
    ParseError(ParseIntError),
    InvalidIdError,
    WrongTypeError,
    InvalidTypeError,
    WrongFieldCountError,
}

pub trait Sign: Debug + Display {
    fn get_payload(&self) -> Vec<u8>;
    fn set_signature(&mut self, signature: Vec<u8>);
    fn get_signature(&self) -> Option<Vec<u8>>;
}
