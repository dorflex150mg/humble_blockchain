use std::fmt::{Debug, Display};
use std::num::ParseIntError;
use thiserror::Error;

use crate::token::{Token, TokenConversionError};

/// A `[u8]` that represents a `[Transaction]` block entry.
pub const TRANSACTION_BLOCK_MEMBER_IDENTIFIER: u8 = 0;
/// A `[u8]` that represents a `[Record]` block entry.
pub const RECORD_BLOCK_MEMBER_IDENTIFIER: u8 = 1;

/// Error type for `[BlockEntry]` trait object id conversion from [u8].
#[derive(Debug, Error)]
pub enum BlockIdError {
    #[error("Tried to convert invalid id {} to a BlockMemberId.", {0})]
    /// The id does not correspond to a valid `[BlockEntry]` trait object id.
    InvalidIdError(u8),
}

/// `[BlockMemberId]` identifies a `[BlockEntry]` trait object as `[Transaction]` or `[Record]`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum BlockEntryId {
    #[default]
    /// Identifies a `[BlockEntry]` trait object as `[Transaction]`.
    Transaction,
    /// Identifies a `[BlockEntry]` trait object as `[Record]`.
    Record,
}

impl TryFrom<u8> for BlockEntryId {
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
impl Into<u8> for BlockEntryId {
    fn into(self) -> u8 {
        match self {
            Self::Transaction => TRANSACTION_BLOCK_MEMBER_IDENTIFIER,
            Self::Record => RECORD_BLOCK_MEMBER_IDENTIFIER,
        }
    }
}

/// Error type for Conversion from `String` to `[BlockEntry]` trait objects.  
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
    /// Attempted to convert to the wrong `[BlockEntry]` trait object.
    WrongTypeError,
    /// Attempted to convert to a non-existant `[BlockEntry]` trait object.
    InvalidTypeError,
    /// String field count does not match this `[BlockEntry]` trait object.
    WrongFieldCountError,
}

/// `[BlockEntry]` represents objects that can be signed by a `[Wallet]`.
/// It is used to distinguish objects that can be part of Block data.
pub trait BlockEntry: Debug + Display + Send {
    /// Returns a payload containg the data to be signed.
    fn get_payload(&self) -> Vec<u8>;
    /// Adds the signature from the `[Wallet]` into the `[BlockEntry]` trait object.
    fn set_signature(&mut self, signature: Vec<u8>);
    /// Returns Some with the signature a `[Wallet]` has set to a `[BlockEntry]` if it has one.
    /// Otherwise returns None.
    fn get_signature(&self) -> Option<Vec<u8>>;

    /// Returns a vector with the `[Token]`s.
    fn get_tokens(&self) -> Vec<Token>;

    /// Returns the sender `[Wallet]`s public key.
    fn get_sender_pk(&self) -> Vec<u8>;

    /// Creates a boxed clone of the concrete type.
    fn clone_box(&self) -> Box<dyn BlockEntry>;

    /// Returns the String representation.
    fn to_string(&self) -> String;

    /// Returns the Entry type id, an variant of `[BlockEntryId]`.
    fn get_entry_type(&self) -> BlockEntryId;

    /// Returns some type-specific unique key.
    fn get_key(&self) -> String;
}

impl<T> BlockEntry for T
where
    T: 'static + Clone + ConcreteBlockEntry + Debug + Display + ToString + Send,
{
    fn clone_box(&self) -> Box<dyn BlockEntry> {
        Box::new(self.clone())
    }
    fn get_payload(&self) -> Vec<u8> {
        self.get_payload()
    }
    fn get_signature(&self) -> Option<Vec<u8>> {
        self.get_signature()
    }
    fn set_signature(&mut self, signature: Vec<u8>) {
        self.set_signature(signature);
    }
    fn get_tokens(&self) -> Vec<Token> {
        self.get_tokens()
    }
    fn get_sender_pk(&self) -> Vec<u8> {
        self.get_sender_pk()
    }
    fn to_string(&self) -> String {
        self.to_string()
    }
    fn get_key(&self) -> String {
        self.get_key()
    }
    fn get_entry_type(&self) -> BlockEntryId {
        self.get_entry_type()
    }
}

/// Helper trait for concrete `[BlockEntry]` implementing types.
pub trait ConcreteBlockEntry {
    /// Returns a payload containg the data to be signed.
    fn get_payload(&self) -> Vec<u8>;
    /// Adds the signature from the `[Wallet]` into the `[BlockEntry]` trait object.
    fn get_signature(&self) -> Option<Vec<u8>>;
    /// Returns Some with the signature a `[Wallet]` has set to a `[BlockEntry]` if it has one.
    /// Otherwise returns None.
    fn set_signature(&mut self, signature: Vec<u8>);
    /// Returns a vector with the `[Token]`s.
    fn get_tokens(&self) -> Vec<Token>;
    /// Returns the sender `[Wallet]`s public key.
    fn get_sender_pk(&self) -> Vec<u8>;
    /// Returns the Entry type id, an variant of `[BlockEntryId]`.
    fn get_entry_type(&self) -> BlockEntryId;
    /// Returns some type-specific unique key.
    fn get_key(&self) -> String;
}
