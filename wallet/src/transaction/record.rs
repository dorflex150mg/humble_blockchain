use crate::{
    token::Token,
    transaction::block_entry_common::{
        BlockEntry, BlockMemberId, ConcreteBlockEntry, EntryDecodeError,
    },
};
use base64::{engine::general_purpose, Engine as _};
use std::fmt::{Debug, Display};
use uuid::Uuid;

/// Number of fields in a Record.
pub const N_RECORD_FIELDS: usize = 6;

#[allow(clippy::struct_field_names)]
/// A key value entry to be recorded in `[BlockChainBlock]`.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Record {
    block_entry_type_id: BlockMemberId,
    record_id: Uuid,
    poster_pk: Vec<u8>,
    key: String,
    value: Vec<u8>,
    tokens: Vec<Token>,
    signature: Option<Vec<u8>>,
}

impl Record {
    /// Creates a new Record with the poster `[Wallet]` public key, the key and value.
    pub fn new(
        poster: Vec<u8>,
        key: impl Into<String>,
        value: Vec<u8>,
        tokens: Vec<Token>,
    ) -> Self {
        Record {
            block_entry_type_id: BlockMemberId::Record,
            record_id: Uuid::new_v4(),
            poster_pk: poster,
            key: key.into(),
            value,
            tokens,
            signature: None,
        }
    }

    /// Returns the sender `[Wallet]` public key.
    #[must_use]
    pub fn get_sender_pk(&self) -> Vec<u8> {
        self.poster_pk.clone()
    }
}

impl TryFrom<String> for Record {
    type Error = EntryDecodeError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let fields: Vec<&str> = value.split(';').collect();
        if fields.len() < N_RECORD_FIELDS {
            return Err(EntryDecodeError::WrongFieldCountError);
        }
        let ident: BlockMemberId = fields[0]
            .parse::<u8>()
            .map_err(|_| EntryDecodeError::InvalidTypeError)?
            .try_into()
            .map_err(|_| EntryDecodeError::InvalidTypeError)?;
        if ident != BlockMemberId::Record {
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
        Ok(Record {
            block_entry_type_id: ident,
            record_id: Uuid::parse_str(fields[1]).map_err(|_| EntryDecodeError::InvalidIdError)?,
            poster_pk: general_purpose::STANDARD.decode(fields[2])?,
            key: fields[3].to_owned(),
            value: general_purpose::STANDARD.decode(fields[4])?,
            tokens,
            signature,
        })
    }
}

#[allow(clippy::from_over_into, clippy::unwrap_used)]
impl Into<String> for Record {
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
            "{};{};{};{};{};{};{}",
            block_entry_type_id,
            self.record_id.as_hyphenated(),
            general_purpose::STANDARD.encode(self.poster_pk),
            self.key,
            general_purpose::STANDARD.encode(self.value),
            joined_tokens,
            signature,
        )
    }
}

impl Display for Record {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.record_id)
    }
}

impl ConcreteBlockEntry for Record {
    fn get_payload(&self) -> Vec<u8> {
        [
            self.record_id.as_bytes().as_slice(),
            self.poster_pk.as_ref(),
            self.key.as_bytes(),
            self.value.as_ref(),
        ]
        .concat()
    }

    fn set_signature(&mut self, signature: Vec<u8>) {
        self.signature = Some(signature);
    }

    fn get_signature(&self) -> Option<Vec<u8>> {
        self.signature.clone()
    }

    fn get_tokens(&self) -> Vec<Token> {
        self.tokens.clone()
    }

    fn get_sender_pk(&self) -> Vec<u8> {
        self.poster_pk.clone()
    }
}
