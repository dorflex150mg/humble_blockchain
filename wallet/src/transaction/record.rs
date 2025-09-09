use crate::transaction::block_entry_common::{BlockMemberId, EntryDecodeError, Sign};
use base64::{engine::general_purpose, Engine as _};
use uuid::Uuid;

const N_RECORD_FIELDS: usize = 6;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Record {
    block_entry_type_id: BlockMemberId,
    record_id: Uuid,
    poster: Vec<u8>,
    key: String,
    value: Vec<u8>,
    signature: Option<Vec<u8>>,
}

impl Record {
    pub fn new(poster: Vec<u8>, key: impl Into<String>, value: Vec<u8>) -> Self {
        Record {
            block_entry_type_id: BlockMemberId::Record,
            record_id: Uuid::new_v4(),
            poster,
            key: key.into(),
            value,
            signature: None,
        }
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
        let signature = match fields[5] {
            "" => None,
            _ => general_purpose::STANDARD.decode(fields[5]).ok(),
        };
        Ok(Record {
            block_entry_type_id: ident,
            record_id: Uuid::parse_str(fields[1]).map_err(|_| EntryDecodeError::InvalidIdError)?,
            poster: general_purpose::STANDARD.decode(fields[2])?,
            key: fields[3].to_owned(),
            value: general_purpose::STANDARD.decode(fields[4])?,
            signature,
        })
    }
}

#[allow(clippy::from_over_into)]
impl Into<String> for Record {
    fn into(self) -> String {
        let block_entry_type_id: u8 = self.block_entry_type_id.into();
        let signature = match &self.signature {
            Some(s) => general_purpose::STANDARD.encode(s.as_slice()).to_string(),
            None => String::new(),
        };

        format!(
            "{};{};{};{};{};{}",
            block_entry_type_id,
            self.record_id.as_hyphenated(),
            general_purpose::STANDARD.encode(self.poster),
            self.key,
            general_purpose::STANDARD.encode(self.value),
            signature,
        )
    }
}

impl Sign for Record {
    fn get_payload(&self) -> Vec<u8> {
        [
            self.record_id.as_bytes().as_slice(),
            self.poster.as_ref(),
            self.key.as_bytes(),
            self.value.as_ref(),
        ]
        .concat()
    }

    fn set_signature(&mut self, signature: Vec<u8>) {
        self.signature = Some(signature);
    }
}
