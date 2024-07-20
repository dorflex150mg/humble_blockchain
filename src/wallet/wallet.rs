pub mod wallet {

    use crate::transaction::transaction::transaction::Transaction;
 
    use ring::rand::{SystemRandom};
    use ring::signature::{Ed25519KeyPair, KeyPair};
    use std::fmt;

    pub struct Wallet {
        pub name: String,
        pub key_pair: Ed25519KeyPair,
        pub coins: Vec<String>,
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
                coins: vec![],
                key_pair,
            }
        }


        pub fn get_pub_key(&self) -> Vec<u8> {
            self.key_pair.public_key().as_ref().to_vec().clone() 
        }

        pub fn add_coin(&mut self, coin: String) {
            self.coins.push(coin);
        }

        pub fn get_coins(&mut self) -> Vec<String> {
            self.coins.iter().map(|coin| {
                                coin.clone()
                            }).collect()
         }

        fn check_balance(&self, amount: usize) -> Result<(), TransactionErr> {
            if amount > self.coins.len() { 
                return Err(TransactionErr::InsuficientBalance);
            }
            Ok(())
        }

        pub fn sign(&self, mut transaction: Transaction) -> Transaction {
            let arr_sender: &[u8] = &transaction.sender.clone();
            let arr_receiver: &[u8] = &transaction.receiver.clone();
            let members = [arr_sender,
                           arr_receiver, 
                           &transaction.timestamp.to_ne_bytes()];
            let mut vec: Vec<u8> = members.concat();
            let coins: Vec<Vec<u8>> = transaction.coins.iter().map(|coin| {
                                                       coin.as_bytes().clone().to_vec()
                                                    }).collect();
            for mut i in coins {
                vec.append(&mut i);
            }
            let bytes = &vec; 
            transaction.signature = Some(self.key_pair.sign(bytes));
            transaction
        }
            
        pub fn submit_transaction(&mut self, receiver: Vec<u8>, amount: usize) 
                                   -> Result<Transaction, TransactionErr> {
            self.check_balance(amount)?;
            let coins: Vec<String> = (0..amount).map(|_| {
                                                         self.coins.pop().unwrap()
                                                       }).collect();
                                   
            Ok(
                self.sign(
                    Transaction::new(
                        self.key_pair.public_key().as_ref().to_vec(), 
                        receiver, 
                        coins,
                    )
                )
            )
        }
    }

    impl fmt::Display for Wallet {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
            let joint_coins = self.coins.join(",\n");
            write!(f, "{}: {{\n{}}}", self.name, joint_coins)
        }
    }
}

    

    
