use std::fmt::{Debug, Display};
use std::num::ParseIntError;
use thiserror::Error;

use crate::token::TokenConversionError;

/// A `[u8]` that represents a `[Transaction]` block entry.
pub const TRANSACTION_BLOCK_MEMBER_IDENTIFIER: u8 = 0;
/// A `[u8]` that represents a `[Record]` block entry.
pub const RECORD_BLOCK_MEMBER_IDENTIFIER: u8 = 1;

/// Error type for `[Sign]` trait object id conversion from [u8].
#[derive(Debug, Error)]
pub enum BlockIdError {
    #[error("Tried to convert invalid id {} to a BlockMemberId.", {0})]
    /// The id does not correspond to a valid `[Sign]` trait object id.
    InvalidIdError(u8),
}

/// `[BlockMemberId]` identifies a `[Sign]` trait object as `[Transaction]` or `[Record]`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum BlockMemberId {
    #[default]
    /// Identifies a `[Sign]` trait object as `[Transaction]`.
    Transaction,
    /// Identifies a `[Sign]` trait object as `[Record]`.
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

/// Error type for Conversion from `String` to `[Sign]` trait objects.  
#[derive(Error, Debug, derive_more::From, derive_more::Display)]
pub enum EntryDecodeError {
    /// Base64 Decode Error.
    Base64Error(base64::DecodeError),
    /// Int Parse Error.
    ParseError(ParseIntError),
    /// Failed to convert a `String` to `[Token]`.
    InvalidTokenError(TokenConversionError),
    /// Invalid Block Id Error.
    InvalidIdError,
    /// Attempted to convert to the wrong `[Sign]` trait object.
    WrongTypeError,
    /// Attempted to convert to a non-existant `[Sign]` trait object.
    InvalidTypeError,
    /// String field count does not match this `[Sign]` trait object.
    WrongFieldCountError,
}

/// `[Sign]` represents objects that can be signed by a `[Wallet]`.
/// It is used to distinguish objects that can be part of Block data.
pub trait Sign: Debug + Display {
    /// Returns a payload containg the data to be signed.
    fn get_payload(&self) -> Vec<u8>;
    /// Adds the signature from the `[Wallet]` into the `[Sign]` trait object.
    fn set_signature(&mut self, signature: Vec<u8>);
    /// Returns Some with the signature a `[Wallet]` has set to a `[Sign]` if it has one.
    /// Otherwise returns None.
    fn get_signature(&self) -> Option<Vec<u8>>;
}
