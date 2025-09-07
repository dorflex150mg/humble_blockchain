use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;
use crate::transaction::block_entry_common::{EntryDecodeError, RECORD_BLOCK_MEMBER_IDENTIFIER};

const N_RECORD_FIELDS: usize = 5; 


#[derive(Debug, Default)]
pub struct Record {
    block_entry_type_id: u8,
    record_id: Uuid,
    poster: Vec<u8>, 
    key: String,
    value: Vec<u8>,
}

impl Record {
    pub fn new(poster: Vec<u8>, key: impl Into<String>, value: Vec<u8>) -> Self {
        Record {
            block_entry_type_id: RECORD_BLOCK_MEMBER_IDENTIFIER,
            record_id: Uuid::new_v4(),
            poster, 
            key: key.into(),
            value,
        }
    }
}

impl TryFrom<String> for Record {
    type Error = EntryDecodeError;
    fn try_from(value: String) -> Result<Self, Self::Error> {
        let fields: Vec<String> = value.split(';').map(|s| s.to_owned()).collect();
        if fields.len() < N_RECORD_FIELDS {
            return Err(EntryDecodeError::WrongFieldCountError);
        }
        let ident = fields[0].parse::<u8>().map_err(|_| EntryDecodeError::WrongTypeError)?;
        if ident != RECORD_BLOCK_MEMBER_IDENTIFIER {
            return Err(EntryDecodeError::WrongTypeError);
        }
        Ok(Record {
            block_entry_type_id: ident,
            record_id: Uuid::parse_str(fields[1].as_str()).map_err(|_| EntryDecodeError::InvalidIdError)?,
            poster: general_purpose::STANDARD.decode(&fields[2])?,
            key: fields[3].clone(),
            value: general_purpose::STANDARD.decode(&fields[4])?,
        })
    }   
}

#[allow(clippy::from_over_into)]
impl Into<String> for Record {
    fn into(self) -> String {
        format!("{};{};{};{};{}",
            self.block_entry_type_id, 
            self.record_id.as_hyphenated(),
            general_purpose::STANDARD.encode(self.poster),
            self.key,
            general_purpose::STANDARD.encode(self.value),
        )
    }
}

