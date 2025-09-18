use wallet::token::Token;
use wallet::wallet::Wallet;

mod tests {

    use wallet::transaction::transaction::Transaction;

    use super::*;

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
        let transaction1 = wallet.submit_block_entry(receiver.get_pub_key(), 1);
        assert!(transaction1.is_ok());
        let transaction2 = wallet.submit_block_entry(receiver.get_pub_key(), 1);
        assert!(transaction2.is_err());
        let coin = Token::try_from("0".repeat(64)).unwrap();
        wallet.add_coin(coin);
        let transaction3 = wallet.submit_block_entry(receiver.get_pub_key(), 0);
        assert!(transaction3.is_err());
    }

    #[test]
    fn test_pub_key() {
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        assert_ne!(wallet1.get_pub_key(), wallet2.get_pub_key());
    }

    #[test]
    fn test_sign_verify() {
        let mut wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        let coin = Token::try_from("0".repeat(64)).unwrap();
        wallet1.add_coin(coin.clone());
        let transaction1 =
            Transaction::new(wallet1.get_pub_key(), wallet2.get_pub_key(), vec![coin]);
        let signed = wallet1.sign(transaction1);
        assert!(wallet1.verify(&signed, None).is_ok());
        assert!(wallet2.verify(&signed, None).is_err());
        let right_pk = wallet1.get_pub_key();
        assert!(wallet2.verify(&signed, Some(right_pk)).is_ok());
        let wrong_pk = wallet2.get_pub_key();
        assert!(wallet1.verify(&signed, Some(wrong_pk)).is_err());
        let another_coin = Token::try_from("0".repeat(64)).unwrap();
        wallet1.add_coin(another_coin.clone());

        let unsigned_tx = Transaction::new(
            wallet1.get_pub_key(),
            wallet2.get_pub_key(),
            vec![another_coin],
        );
        assert!(wallet1.verify(&unsigned_tx, None).is_err());
    }
}
