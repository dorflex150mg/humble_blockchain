pub mod transaction {
    
    use crate::node::reply::reply::Reply;

    use std::{
        fmt,
        num::ParseIntError,
        time::{SystemTime, 
            UNIX_EPOCH},
    };
    use thiserror::Error;
    use ring::signature::Signature;
    use base64::{Engine as _, engine::general_purpose, DecodeError};
    use uuid::Uuid;


    pub const FIELD_END: char = ';';


    #[derive(Error, Debug, derive_more::From, derive_more::Display)]    
    pub enum TransactionFromBase64Error {
        Base64Error(base64::DecodeError),
        ParseError(ParseIntError),
    }

    #[derive(Clone)]
    pub struct Transaction {
        pub sender: Vec<u8>,
        pub receiver: Vec<u8>,
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
                sender,
                receiver,
                timestamp: now,
                coins,
                signature: None,
            }
        }

        pub fn to_base64(&self) -> String {
            let joined_coins = self.coins.join("");
            format!("{};{};{};{};{};", 
                general_purpose::STANDARD.encode(&self.sender).to_string(), 
                general_purpose::STANDARD.encode(&self.receiver).to_string(),
                joined_coins,
                self.timestamp.to_string(),
                general_purpose::STANDARD.encode(&self.signature.as_ref().unwrap().as_slice()).to_string()
            )
        }
    }

    impl TryFrom<String> for Transaction {
        type Error = TransactionFromBase64Error;
        fn try_from(string: String) -> Result<Self, Self::Error> {
            let mut params: Vec<&str> = string.as_str().split(';').collect();
            Ok(Transaction {
                sender: general_purpose::STANDARD.decode(params[0])?, 
                receiver: general_purpose::STANDARD.decode(params[1])?,
                coins: vec![params[2].to_string().clone()],
                timestamp: params[3].parse::<u64>()?,
                signature: Some((general_purpose::STANDARD.decode(params[4])?)),
            })
        }
    }

    impl Into<String> for Transaction {
        fn into(self) -> String {
            let joined_coins = self.coins.join("");
            format!("{};{};{};{};{};", 
                general_purpose::STANDARD.encode(&self.sender).to_string(), 
                general_purpose::STANDARD.encode(&self.receiver).to_string(),
                joined_coins,
                self.timestamp.to_string(),
                general_purpose::STANDARD.encode(&self.signature.as_ref().unwrap().as_slice()).to_string()
            )
        }
    }

    impl fmt::Display for Transaction {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "timestamp: {}, sender: {:?}, receiver: {:?}, coins: {}", 
                    self.timestamp, self.sender, self.receiver, self.coins.join(" "))
        }
    }

    impl Reply for Transaction {}

}
