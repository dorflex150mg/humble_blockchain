pub mod peer {

    use uuid::Uuid;


    #[derive(Debug)]
    enum Type {
        Tracker,
        Normal,
    }

    #[derive(Debug)]
    pub struct Peer {
        id: Uuid,
        peer_type: Type,
        key_start: String,
        key_end: String,
    }

    impl Peer {
        pub fn new(id: Uuid, 
            peer_type: Type,
            key_start: impl Into<String>,
            key_end: impl Into<String>) -> Self {
            Peer {
                id,
                peer_type,
                key_start: key_start.into(),
                key_end: key_end.into(),
            }
        }

    }

    impl Default for Peer {
        fn default() -> Self {
            Self {
                id: Uuid::new_v4(),
                peer_type: Type::Tracker,
                key_start: "00000000".to_string(),
                key_end: "FFFFFFFF".to_string(),
            } 
        }
    }


}
