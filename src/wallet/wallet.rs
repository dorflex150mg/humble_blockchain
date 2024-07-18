pub mod wallet {

    use crate::transaction::transaction::transaction::Transaction;
 
    use ring::rand::{SystemRandom, SecureRandom};
    use ring::signature::{Ed25519KeyPair, KeyPair};

    pub struct Wallet {
        pub name: String,
        pub balance: f64,
        pub key_pair: Ed25519KeyPair,
    }

    pub enum TransactionErr {
        InsuficientBalance,
    }

    fn generate_key_pair() -> Ed25519KeyPair {
        let rng = SystemRandom::new();
        let pkcs8_bytes = Ed25519KeyPair::generate_pkcs8(&rng).unwrap(); // pkcs#8 key syntax
                                                                         // with Edwards curve
                                                                         // algorithm
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8_bytes.as_ref())
                                        .unwrap();  //key struct from bytes
        println!("printing the key pair: {:#?}", key_pair);
        key_pair
    }
    
    impl Wallet {
        pub fn new(name: String) -> Self{
            let key_pair = generate_key_pair();
            Wallet {
                name,
                balance: 0.0,
                key_pair,
            }
        }

        fn check_balance(&self, amount: f64) -> Result<(), TransactionErr> {
            if amount > self.balance { 
                return Err(TransactionErr::InsuficientBalance);
            }
            Ok(())
        }

        fn sign(&self, mut transaction: Transaction) -> Transaction {
            let arr_sender: &[u8] = &transaction.sender.clone();
            let arr_receiver: &[u8] = &transaction.receiver.clone();
            let members = [arr_sender,
                           arr_receiver, 
                           &transaction.timestamp.to_ne_bytes(),//contrived 
                           &transaction.amount.to_ne_bytes()];
            let vec: Vec<u8> = members.concat();
            let bytes = &vec; //little trick to concat all bytes
            transaction.signature = Some(self.key_pair.sign(bytes));
            transaction
        }
            
        pub fn submit_transaction(&self, receiver: Vec<u8>, amount: f64) 
                                   -> Result<Transaction, TransactionErr> {
            self.check_balance(amount)?;
            Ok(
                self.sign(
                    Transaction::new(
                        self.key_pair.public_key().as_ref().to_vec(), 
                        receiver, 
                        amount,
                    )
                )
            )
        }
    }
}

    

    
