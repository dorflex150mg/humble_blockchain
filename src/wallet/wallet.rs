pub mod wallet {

    use crate::transaction::transaction::transaction::Transaction;
 
    use ring::rand::{SystemRandom};
    use ring::signature::{KeyPair, EcdsaKeyPair, ECDSA_P256_SHA256_ASN1_SIGNING};
    use std::fmt;

    pub struct Wallet {
        //pub key_pair: Ed25519KeyPair,
        pub key_pair: EcdsaKeyPair,
        pub coins: Vec<String>,
        rng: SystemRandom,
    }

    pub enum TransactionErr {
        InsuficientBalance,
    }

    fn generate_key_pair() -> (EcdsaKeyPair, SystemRandom) {
        let rng = SystemRandom::new();
        let pkcs8_bytes = EcdsaKeyPair::generate_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, &rng).unwrap();
        let key_pair = EcdsaKeyPair::from_pkcs8(&ECDSA_P256_SHA256_ASN1_SIGNING, pkcs8_bytes.as_ref(), &rng)
        .unwrap();  
        (key_pair, rng)
    }


    impl Wallet {
        pub fn new() -> Self{
            let (key_pair, rng) = generate_key_pair();
            Wallet {
                coins: vec![],
                key_pair,
                rng,
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
            let coins: Vec<Vec<u8>> = transaction.coins
                .iter()
                .map(|coin| { coin.as_bytes().to_vec() })
                .collect();
            for mut i in coins {
                vec.append(&mut i);
            }
            let bytes = &vec; 
            transaction.signature = Some(self.key_pair.sign(&self.rng, bytes).unwrap().as_ref().to_vec());
            transaction
        }
            
        pub fn submit_transaction(&mut self, receiver: Vec<u8>, amount: usize) 
                    -> Result<Transaction, TransactionErr> {
            self.check_balance(amount)?;
            let coins: Vec<String> = (0..amount).map(|_| {
                self.coins.pop().unwrap()
            }).collect();
                                   
            Ok(self.sign(Transaction::new(
                self.key_pair.public_key().as_ref().to_vec(), 
                receiver, 
                coins,
            )))
        }
    }

    impl fmt::Display for Wallet {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result { 
            let joint_coins = self.coins.join(",\n");
            write!(f, "{{\n{}}}", joint_coins)
        }
    }
}

    
