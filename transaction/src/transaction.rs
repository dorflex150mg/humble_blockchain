pub const N_TRANSACTION_FIELDS: usize = 6;
pub const TRANSACTION_BLOCK_MEMBER_IDENTIFIER: u8 = 0; //TODO: Repeated. Put somewhere accesible.

use std::{
    fmt,
    num::ParseIntError,
    time::{SystemTime, 
        UNIX_EPOCH},
};
use thiserror::Error;
use base64::{Engine as _, engine::general_purpose};


#[derive(Error, Debug, derive_more::From, derive_more::Display)]    
pub enum TransactionFromBase64Error {
    Base64Error(base64::DecodeError),
    ParseError(ParseIntError),
}

#[derive(Clone)]
pub struct Transaction {
    pub block_entry_type_id: u8,
    pub sender_wallet: Vec<u8>,
    pub receiver_wallet: Vec<u8>,
    pub timestamp: u64,
    pub coins: Vec<String>,
    pub signature: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new(sender: Vec<u8>, receiver: Vec<u8>, coins: Vec<String>) -> Self {
        let now = SystemTime::now()
                     .duration_since(UNIX_EPOCH)
                     .unwrap()
                     .as_secs();
        Transaction {
            block_entry_type_id: TRANSACTION_BLOCK_MEMBER_IDENTIFIER, 
            sender_wallet: sender,
            receiver_wallet: receiver,
            timestamp: now,
            coins,
            signature: None,
        }
    }
}

impl TryFrom<String> for Transaction {
    type Error = TransactionFromBase64Error;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        let params: Vec<&str> = string.as_str().split(';').collect();
        let signature = general_purpose::STANDARD.decode(params[5]).ok();
        Ok(Transaction {
            block_entry_type_id: TRANSACTION_BLOCK_MEMBER_IDENTIFIER,
            sender_wallet: general_purpose::STANDARD.decode(params[1])?, 
            receiver_wallet: general_purpose::STANDARD.decode(params[2])?,
            coins: vec![params[3].to_string().clone()],
            timestamp: params[4].parse::<u64>()?,
            signature,
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for Transaction {
    fn into(self) -> String {
        let joined_coins = self.coins.join("");
        let signature = match &self.signature {
            Some(_) => general_purpose::STANDARD.encode(self
                .signature
                .as_ref()
                .unwrap()
                .as_slice()
            ).to_string(),
            None => "".to_string(),
        };
        format!("{};{};{};{};{};{};", 
            self.block_entry_type_id,
            general_purpose::STANDARD.encode(&self.sender_wallet), 
            general_purpose::STANDARD.encode(&self.receiver_wallet),
            joined_coins,
            self.timestamp,
            signature,
        )
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "timestamp: {}, sender: {:?}, receiver: {:?}, coins: {}", 
                self.timestamp, self.sender_wallet, self.receiver_wallet, self.coins.join(" "))
    }
}

