use base64::{Engine as _, engine::general_purpose};
use uuid::Uuid;
use crate::transaction::block_entry_common::{EntryDecodeError, Sign, RECORD_BLOCK_MEMBER_IDENTIFIER};

const N_RECORD_FIELDS: usize = 6; 

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Record {
    block_entry_type_id: u8,
    record_id: Uuid,
    poster: Vec<u8>, 
    key: String,
    value: Vec<u8>,
    signature: Option<Vec<u8>>,
}

impl Record {
    pub fn new(poster: Vec<u8>, key: impl Into<String>, value: Vec<u8>) -> Self {
        Record {
            block_entry_type_id: RECORD_BLOCK_MEMBER_IDENTIFIER,
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
        let ident = fields[0]
            .parse::<u8>()
            .map_err(|_| EntryDecodeError::WrongTypeError)?;
        if ident != RECORD_BLOCK_MEMBER_IDENTIFIER {
            return Err(EntryDecodeError::WrongTypeError);
        }
        let signature = match fields[5] {
            "" => None,
            _ =>  general_purpose::STANDARD.decode(fields[5]).ok(), 
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
        let signature = match &self.signature {
            Some(_) => general_purpose::STANDARD.encode(self
                .signature
                .as_ref()
                .unwrap()
                .as_slice()
            ).to_string(),
            None => "".to_string(),
        };

        format!("{};{};{};{};{};{}",
            self.block_entry_type_id, 
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
        ].concat()
    }
    fn set_signature(&mut self, signature: Vec<u8>) {
       self.signature = Some(signature);
    }
}

