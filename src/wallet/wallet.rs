pub mod wallet {
 
    use ring::rand::{SystemRandom, SecureRandom};
    use ring::signature::{Ed25519KeyPair, KeyPair, Signature, ED25519};

    pub struct Wallet {
        name: String,
        pub_key: String,
        priv_key: String,
        balance: f64,
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
                pub_key: String::from(""),
                priv_key: String::from(""),
                balance: 0.0,
            }
        }
    }
}

    

    
