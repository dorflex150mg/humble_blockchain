use thiserror::Error;
use std::num::ParseIntError;

pub const TRANSACTION_BLOCK_MEMBER_IDENTIFIER: u8 = 0; 
pub const RECORD_BLOCK_MEMBER_IDENTIFIER: u8 = 0; 

#[derive(Error, Debug, derive_more::From, derive_more::Display)]    
pub enum EntryDecodeError {
    Base64Error(base64::DecodeError),
    ParseError(ParseIntError),
    InvalidIdError,
    WrongTypeError,
    WrongFieldCountError,
}

pub trait Sign {
    fn get_payload(&self) -> Vec<u8>; 
    fn set_signature(&mut self, signaure: Vec<u8>);  
}
