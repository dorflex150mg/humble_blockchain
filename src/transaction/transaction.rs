pub mod transaction {
    use std::fmt;
    use std::time::{SystemTime, UNIX_EPOCH};
    use ring::signature::Signature;

    pub struct Transaction {
        pub sender: Vec<u8>,
        pub receiver: Vec<u8>,
        pub timestamp: u64,
        pub coins: Vec<String>,
        pub signature: Option<Signature>,
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
    }

    impl fmt::Display for Transaction {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "timestamp: {}", 
                    self.timestamp)
        }
   }

}
