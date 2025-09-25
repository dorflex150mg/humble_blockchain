#[cfg(test)]
mod tests {

    use chain::chain::Chain;
    use store::store::Store;

    #[test]
    fn round_trip() {
        let chain = Chain::new();
        let str_chain = serde_json::to_string(&chain).unwrap();
        println!("chain: {}", str_chain);
        let mut store = Store::new(None);
        assert!(store.store(str_chain.as_str()).is_ok());
        let str_chain = store.load().unwrap();
        //assert!(res.is_ok());
        //let str_chain = res.unwrap();
        let new_chain: Chain = serde_json::from_str(&str_chain).unwrap();
        assert_eq!(chain, new_chain);
    }
}
