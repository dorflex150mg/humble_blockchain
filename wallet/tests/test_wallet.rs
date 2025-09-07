use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use uuid::Uuid;

use wallet::transaction::transaction::Transaction;
use wallet::wallet::Wallet;

use wallet::transaction::block_entry_common::TRANSACTION_BLOCK_MEMBER_IDENTIFIER;

#[test]
fn test_coins() {
    let mut wallet = Wallet::new();
    let empty: Vec<String> = vec![];
    assert_eq!(wallet.get_coins(), empty);
    let coin = "0".repeat(64);
    wallet.add_coin(coin.clone());
    assert_eq!(wallet.get_coins(), vec![coin]);
}

#[test]
fn test_transaction() {
    let mut wallet = Wallet::new();
    let coin = "0".repeat(64);
    wallet.add_coin(coin);
    let mut receiver = Wallet::new();
    let transaction1 = wallet.submit_transaction(receiver.get_pub_key(), 1);
    assert!(transaction1.is_ok());
    let transaction2 = wallet.submit_transaction(receiver.get_pub_key(), 1);
    assert!(transaction2.is_err());
    let coin = "0".repeat(64);
    wallet.add_coin(coin);
    let transaction3 = wallet.submit_transaction(receiver.get_pub_key(), 0);
    assert!(transaction3.is_err());
}

#[test]
fn test_pub_key() {
    let mut wallet1 = Wallet::new();
    let mut wallet2 = Wallet::new();
    assert_ne!(wallet1.get_pub_key(), wallet2.get_pub_key());
}
