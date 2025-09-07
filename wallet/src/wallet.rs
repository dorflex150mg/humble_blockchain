use crate::transaction::transaction::Transaction;
use crate::transaction::block_entry_common::Sign;

use ring::rand::{SystemRandom};
use ring::signature::{KeyPair, EcdsaKeyPair, ECDSA_P256_SHA256_ASN1_SIGNING};
use std::fmt;


pub struct Wallet {
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

    #[allow(dead_code)]
    pub fn get_coins(&mut self) -> Vec<String> {
        self.coins.to_vec()
    }

    #[allow(dead_code)]
    fn check_balance(&self, amount: usize) -> Result<(), TransactionErr> {
        if amount > self.coins.len() { 
            return Err(TransactionErr::InsuficientBalance);
        }
        Ok(())
    }

    pub fn sign<T: Sign>(&self, mut entry: T) -> T {
        let vec = entry.get_payload();
        let bytes = &vec; 
        entry.set_signature(self.key_pair.sign(&self.rng, bytes).unwrap().as_ref().to_vec());
        entry 
    }
        

    #[allow(dead_code)]
    pub fn submit_transaction(&mut self, receiver: Vec<u8>, amount: usize) 
                -> Result<impl Sign, TransactionErr> {
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

impl Default for Wallet {
    fn default() -> Self {
        Wallet::new()
    }

}
