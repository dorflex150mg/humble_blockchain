use std::{
    fmt,
    time::{SystemTime, 
        UNIX_EPOCH},
};
use base64::{Engine as _, engine::general_purpose};
use crate::transaction::block_entry_common::{self, TRANSACTION_BLOCK_MEMBER_IDENTIFIER};

pub const N_TRANSACTION_FIELDS: usize = 6;

#[derive(Clone, PartialEq, Eq, Debug)]
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
            block_entry_type_id: block_entry_common::TRANSACTION_BLOCK_MEMBER_IDENTIFIER, 
            sender_wallet: sender,
            receiver_wallet: receiver,
            timestamp: now,
            coins,
            signature: None,
        }
    }
}

impl TryFrom<String> for Transaction {
    type Error = block_entry_common::EntryDecodeError;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        let fields: Vec<&str> = string.as_str().split(';').collect();
        if fields.len() < N_TRANSACTION_FIELDS {
            return Err(block_entry_common::EntryDecodeError::WrongFieldCountError);
        }
        let ident = fields[0].parse::<u8>().map_err(|_| block_entry_common::EntryDecodeError::WrongTypeError)?;
        if ident != TRANSACTION_BLOCK_MEMBER_IDENTIFIER {
            return Err(block_entry_common::EntryDecodeError::WrongTypeError);
        }
        let signature = match fields[5] {
            "" => None,
            _ =>  general_purpose::STANDARD.decode(fields[5]).ok(), 
        };
        Ok(Transaction {
            block_entry_type_id: ident,
            sender_wallet: general_purpose::STANDARD.decode(fields[1])?, 
            receiver_wallet: general_purpose::STANDARD.decode(fields[2])?,
            coins: vec![fields[3].to_string().clone()],
            timestamp: fields[4].parse::<u64>()?,
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
