use crate::transaction::block_entry_common::{BlockMemberId, EntryDecodeError, Sign};
use base64::{engine::general_purpose, Engine as _};
use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

pub const N_TRANSACTION_FIELDS: usize = 7;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Transaction {
    pub block_entry_type_id: BlockMemberId,
    pub transaction_id: Uuid,
    pub sender_pk: Vec<u8>,
    pub receiver_pk: Vec<u8>,
    pub timestamp: u64,
    pub coins: Vec<String>,
    pub signature: Option<Vec<u8>>,
}

impl Transaction {
    pub fn new(sender: Vec<u8>, receiver: Vec<u8>, coins: Vec<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Transaction {
            block_entry_type_id: BlockMemberId::Transaction,
            transaction_id: Uuid::new_v4(),
            sender_pk: sender,
            receiver_pk: receiver,
            timestamp: now,
            coins,
            signature: None,
        }
    }

    pub fn get_sender_pk(&self) -> Vec<u8> {
        self.sender_pk.clone()
    }
}

impl TryFrom<String> for Transaction {
    type Error = EntryDecodeError;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        let fields: Vec<&str> = string.as_str().split(';').collect();
        if fields.len() < N_TRANSACTION_FIELDS {
            return Err(EntryDecodeError::WrongFieldCountError);
        }
        let ident: BlockMemberId = fields[0]
            .parse::<u8>()
            .map_err(|_| EntryDecodeError::InvalidTypeError)?
            .try_into()
            .map_err(|_| EntryDecodeError::InvalidTypeError)?;
        if ident != BlockMemberId::Transaction {
            return Err(EntryDecodeError::WrongTypeError);
        }
        let signature = match fields[6] {
            "" => None,
            _ => general_purpose::STANDARD.decode(fields[6]).ok(),
        };
        Ok(Transaction {
            block_entry_type_id: ident,
            transaction_id: Uuid::parse_str(fields[1])
                .map_err(|_| EntryDecodeError::InvalidIdError)?,
            sender_pk: general_purpose::STANDARD.decode(fields[2])?,
            receiver_pk: general_purpose::STANDARD.decode(fields[3])?,
            coins: vec![fields[4].to_string().clone()],
            timestamp: fields[5].parse::<u64>()?,
            signature,
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for Transaction {
    fn into(self) -> String {
        let joined_coins = self.coins.join("");
        let block_entry_type_id: u8 = self.block_entry_type_id.into();

        let signature = match &self.signature {
            Some(s) => general_purpose::STANDARD.encode(s.as_slice()).to_string(),
            None => String::new(),
        };

        format!(
            "{};{};{};{};{};{};{};",
            block_entry_type_id,
            self.transaction_id.as_hyphenated(),
            general_purpose::STANDARD.encode(&self.sender_pk),
            general_purpose::STANDARD.encode(&self.receiver_pk),
            joined_coins,
            self.timestamp,
            signature,
        )
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "timestamp: {}, sender: {:?}, receiver: {:?}, coins: {}",
            self.timestamp,
            self.sender_pk,
            self.receiver_pk,
            self.coins.join(" ")
        )
    }
}

impl Sign for Transaction {
    fn get_payload(&self) -> Vec<u8> {
        [
            self.transaction_id.as_bytes().as_slice(),
            self.sender_pk.as_ref(),
            self.receiver_pk.as_ref(),
            self.coins.join(";").as_bytes(),
        ]
        .concat()
    }

    fn set_signature(&mut self, signature: Vec<u8>) {
        self.signature = Some(signature);
    }

    fn get_signature(&self) -> Option<Vec<u8>> {
        self.signature.clone()
    }
}
