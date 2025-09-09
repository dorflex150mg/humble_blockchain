use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use wallet::transaction::transaction::Transaction;
use wallet::wallet::Wallet;

use wallet::transaction::block_entry_common::TRANSACTION_BLOCK_MEMBER_IDENTIFIER;

#[test]
fn round_trip() {
    let some_token = "0".repeat(64);
    let sender = Wallet::new().get_pub_key();
    let receiver = Wallet::new().get_pub_key();
    let test_transaction = Transaction::new(sender, receiver, vec![some_token]);

    let string: String = test_transaction.clone().into();

    let fields: Vec<String> = string.split(";").map(|x| x.to_owned()).collect();

    thread::sleep(Duration::from_secs(1));

    assert_eq!(
        fields[0].parse::<u8>().unwrap(),
        TRANSACTION_BLOCK_MEMBER_IDENTIFIER
    );
    assert!(Uuid::parse_str(&fields[1]).is_ok());
    assert_eq!(fields[2].len(), 88);
    assert_eq!(fields[3].len(), 88);
    assert_eq!(
        fields[4],
        "0000000000000000000000000000000000000000000000000000000000000000"
    );
    println!(
        "{} - {}",
        fields[5].parse::<u64>().unwrap(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    assert!(
        fields[5].parse::<u64>().unwrap()
            < SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
    );
    assert_eq!(fields[6], "");
    let retrieved_transaction = Transaction::try_from(string).unwrap();

    assert_eq!(retrieved_transaction, test_transaction);
}

#[test]
fn test_signature() {
    let some_token = "0".repeat(64);
    let sender_wallet = Wallet::new();
    let sender = sender_wallet.get_pub_key();
    let receiver = Wallet::new().get_pub_key();
    let test_transaction = Transaction::new(sender, receiver, vec![some_token]);
    let same_transaction = test_transaction.clone();
    let test_transaction = sender_wallet.sign(test_transaction);
    assert_ne!(test_transaction, same_transaction);
}
