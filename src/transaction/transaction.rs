pub mod transaction {
    use std::fmt;
    use std::time::{SystemTime, UNIX_EPOCH, Duration};

    pub struct Transaction {
        sender_pub_key: String,
        receiver_pub_key: String,
        timestamp: u64,
        amount: f64,
    }

    impl Transaction {
        pub fn new(sender_pub_key: String, receiver_pub_key: String, amount: f64) -> Self {
            let now = SystemTime::now()
                         .duration_since(UNIX_EPOCH)
                         .unwrap()
                         .as_secs();
            Transaction {
                sender_pub_key,
                receiver_pub_key,
                timestamp: now,
                amount,
            }
        }
    }

    impl fmt::Display for Transaction {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "sender: {}, receiver: {}, timestamp: {}, amount: {}", 
                    self.sender_pub_key, 
                    self.receiver_pub_key, 
                    self.timestamp, 
                    self.amount)
        }
   }

}
