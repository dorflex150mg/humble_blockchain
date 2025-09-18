use crate::{
    token::{Token, TokenConversionError},
    transaction::block_entry_common::{BlockMemberId, EntryDecodeError, Sign},
};
use base64::{engine::general_purpose, Engine as _};
use std::{
    fmt,
    time::{SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

/// Number of fields in a Transaction.
pub const N_TRANSACTION_FIELDS: usize = 7;

#[allow(clippy::struct_field_names)]
#[derive(Clone, PartialEq, Eq, Debug)]
/// `[Transaction]`s change ownership a `[Token]` vector from a  `[Wallet]`, idetified by its public
/// key `sender_pk` to a receiver `[Wallet]`, identified by its public key `receiver_pk`. For a
/// transaction to be valid, the sender must add its signature to the `signature` field and a miner
/// must submit it to a `[BlockChain]`.
pub struct Transaction {
    block_entry_type_id: BlockMemberId,
    transaction_id: Uuid,
    sender_pk: Vec<u8>,
    /// The public key of the receiver's `[Wallet]`.
    pub receiver_pk: Vec<u8>,
    timestamp: u64,
    /// The `[Token]`s given by the sender to the receiver.
    pub tokens: Vec<Token>,
    signature: Option<Vec<u8>>,
}

impl Transaction {
    /// Creates a new Transaction.
    #[must_use]
    pub fn new(sender: Vec<u8>, receiver: Vec<u8>, coins: Vec<Token>) -> Self {
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
            tokens: coins,
            signature: None,
        }
    }

    /// Returns the sender's public key.
    #[must_use]
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

        let tokens: Vec<Token> = fields[5]
            .split(',')
            .map(|t| {
                let token: Result<Token, EntryDecodeError> = t
                    .to_string()
                    .try_into()
                    .map_err(EntryDecodeError::InvalidTokenError);
                token
            })
            .collect::<Result<_, _>>()?;

        Ok(Transaction {
            block_entry_type_id: ident,
            transaction_id: Uuid::parse_str(fields[1])
                .map_err(|_| EntryDecodeError::InvalidIdError)?,
            sender_pk: general_purpose::STANDARD.decode(fields[2])?,
            receiver_pk: general_purpose::STANDARD.decode(fields[3])?,
            timestamp: fields[4].parse::<u64>()?,
            tokens,
            signature,
        })
    }
}

#[allow(clippy::from_over_into, clippy::unwrap_used)]
impl Into<String> for Transaction {
    fn into(self) -> String {
        let str_tokens: Vec<String> = self
            .tokens
            .iter()
            .map(|t| {
                let s: String = String::try_from(t.clone()).unwrap();
                s
            })
            .collect();

        let joined_tokens = str_tokens.join(",");
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
            self.timestamp,
            joined_tokens,
            signature,
        )
    }
}

impl fmt::Display for Transaction {
    #[allow(clippy::unwrap_used)]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let str_tokens: Vec<String> = self
            .tokens
            .iter()
            .map(|t| {
                let s: String = String::try_from(t.clone()).unwrap();
                s
            })
            .collect();

        write!(
            f,
            "timestamp: {}, sender: {:?}, receiver: {:?}, coins: {}",
            self.timestamp,
            self.sender_pk,
            self.receiver_pk,
            str_tokens.join(", "),
        )
    }
}

impl Sign for Transaction {
    #[allow(clippy::unwrap_used)]
    fn get_payload(&self) -> Vec<u8> {
        let str_tokens: Vec<String> = self
            .tokens
            .iter()
            .map(|t| {
                let s: String = String::try_from(t.clone()).unwrap();
                s
            })
            .collect();
        [
            self.transaction_id.as_bytes().as_slice(),
            self.sender_pk.as_ref(),
            self.receiver_pk.as_ref(),
            str_tokens.join(";").as_bytes(),
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
