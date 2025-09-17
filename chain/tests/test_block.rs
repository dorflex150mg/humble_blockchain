use chain::block::block::{Block, Hash, HASH_SIZE};
use wallet::{token::Token, transaction::transaction::Transaction, wallet::Wallet};

#[test]
fn test_hash() {
    assert!(Hash::try_from("1".to_owned()).is_err());
    assert!(Hash::try_from("â".to_owned()).is_err());
    assert!(Hash::try_from("1".repeat(64)).is_ok());
    assert!(Hash::try_from("1".repeat(65)).is_err());
}

#[test]
fn test_block() {
    let hash = Hash::try_from("1".repeat(64)).unwrap();
    let block = Block::new(0, hash, String::new(), None);
    let empty: Vec<Transaction> = vec![];
    assert_eq!(block.get_transactions(), empty);
    let token: Token = Hash::default().into();

    let transaction = Transaction::new(
        Wallet::new().get_pub_key(),
        Wallet::new().get_pub_key(),
        vec![token],
    );
    let new_block = Block::new(0, Hash::default(), transaction.clone().into(), None);
    //assert_eq!(new_block.get_transactions(), vec![transaction.clone()]);
    assert_eq!(new_block.calculate_hash().len(), HASH_SIZE);
    let new_block2 = Block::new(0, Hash::default(), transaction.clone().into(), None);
    assert_eq!(new_block.calculate_hash(), new_block2.calculate_hash());
}
