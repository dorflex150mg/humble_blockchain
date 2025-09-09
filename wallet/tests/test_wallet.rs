use wallet::token::Token;
use wallet::wallet::Wallet;

#[test]
fn test_coins() {
    let mut wallet = Wallet::new();
    let empty: Vec<Token> = vec![];
    assert_eq!(wallet.get_coins(), empty);
    let coin = Token::try_from("0".repeat(64)).unwrap();
    wallet.add_coin(coin.clone());
    assert_eq!(wallet.get_coins(), vec![coin]);
}

#[test]
fn test_transaction() {
    let mut wallet = Wallet::new();
    let coin = Token::try_from("0".repeat(64)).unwrap();
    wallet.add_coin(coin);
    let receiver = Wallet::new();
    let transaction1 = wallet.submit_transaction(receiver.get_pub_key(), 1);
    assert!(transaction1.is_ok());
    let transaction2 = wallet.submit_transaction(receiver.get_pub_key(), 1);
    assert!(transaction2.is_err());
    let coin = Token::try_from("0".repeat(64)).unwrap();
    wallet.add_coin(coin);
    let transaction3 = wallet.submit_transaction(receiver.get_pub_key(), 0);
    assert!(transaction3.is_err());
}

#[test]
fn test_pub_key() {
    let wallet1 = Wallet::new();
    let wallet2 = Wallet::new();
    assert_ne!(wallet1.get_pub_key(), wallet2.get_pub_key());
}
