use std::thread;
use std::time::Duration;

use uuid::Uuid;

use wallet::transaction::record::Record;
use wallet::wallet::Wallet;

use wallet::transaction::block_entry_common::RECORD_BLOCK_MEMBER_IDENTIFIER;

#[test]
fn round_trip() {
    let poster = Wallet::new().get_pub_key();
    let test_record = Record::new(poster, "some id", "some data".as_bytes().to_vec());

    let string: String = test_record.clone().into();

    let fields: Vec<&str> = string.split(";").collect();

    thread::sleep(Duration::from_secs(1));

    assert_eq!(
        fields[0].parse::<u8>().unwrap(),
        RECORD_BLOCK_MEMBER_IDENTIFIER
    );
    assert!(Uuid::parse_str(fields[1]).is_ok());
    assert_eq!(fields[2].len(), 88);
    assert_eq!(fields[3], "some id");
    assert_eq!(fields[5], "");
    let retrieved_record = Record::try_from(string).unwrap();

    assert_eq!(retrieved_record, test_record);
}

#[test]
fn test_signature() {
    let poster_wallet = Wallet::new();
    let test_record = Record::new(
        poster_wallet.get_pub_key(),
        "some id",
        "some data".as_bytes().to_vec(),
    );
    let same_record = test_record.clone();
    let test_record = poster_wallet.sign(test_record);
    assert_ne!(test_record, same_record);
}
