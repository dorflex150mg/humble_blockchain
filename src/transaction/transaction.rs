pub mod transaction {
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


    #[derive(Error, Debug, derive_more::From, derive_more::Display)]    
    pub enum TransactionFromBase64Error {
        Base64Error(base64::DecodeError),
        ParseError(ParseIntError),
    }

    pub struct Transaction {
        //pub id: Uuid,
        pub sender: Vec<u8>,
        pub receiver: Vec<u8>,
        pub timestamp: u64,
        pub coins: Vec<String>,
        pub signature: Option<Vec<u8>>,
        //pub signature: &[u8],
    }

    impl Transaction {
        pub fn new(sender: Vec<u8>, receiver: Vec<u8>, coins: Vec<String>) -> Self {
            let now = SystemTime::now()
                         .duration_since(UNIX_EPOCH)
                         .unwrap()
                         .as_secs();
            Transaction {
                //id: Uuid::new_v4(),
                sender,
                receiver,
                timestamp: now,
                coins,
                signature: None,
            }
        }

        pub fn to_base64(&self) -> String {
            let joined_coins = self.coins.join("");

            format!("{}{}{}{}{}", 
                general_purpose::STANDARD.encode(&self.sender).to_string(), 
                general_purpose::STANDARD.encode(&self.receiver).to_string(),
                joined_coins,
                self.timestamp.to_string(),
                general_purpose::STANDARD.encode(&self.signature.as_ref().unwrap().as_slice()).to_string()
            )
        }

        pub fn from_base64(raw: String) -> Result<Self, TransactionFromBase64Error> {
            Ok(Transaction {
                sender: general_purpose::STANDARD.decode(&raw[0..63])?,             // 64     
                receiver: general_purpose::STANDARD.decode(&raw[64..127])?,          // 64 
                coins: vec![(&raw[128..191]).to_string()],                                        // 64 
                timestamp: (&raw[192..195]).to_string().parse::<u64>()?,              // 4 
                signature: Some((general_purpose::STANDARD.decode(&raw[196..259])?)),  // 64
            })
        }
    }

    impl fmt::Display for Transaction {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "timestamp: {}", 
                    self.timestamp)
        }
   }

}
