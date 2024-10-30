pub mod peer {

    use uuid::Uuid;

    const DEFAULT_START_KEY: &str = "00000000";
    const DEFAULT_END_KEY: &str = "FFFFFFFF";


    #[derive(Debug)]
    pub enum Type {
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
                key_start: DEFAULT_START_KEY.to_string(),
                key_end: DEFAULT_END_KEY.to_string(),
            } 
        }
    }

    #[derive(Default)]
    pub struct PeerBuilder {
        id: Option<Uuid>,
        peer_type: Option<Type>,
        key_start: Option<String>,
        key_end: Option<String>,
    }
    
    impl PeerBuilder {
        pub fn new() -> Self {
            PeerBuilder::default()
        }

        pub fn with_id(mut self, id: Uuid) -> Self {
            self.id = Some(id);
            self
        }

        pub fn with_type(mut self, peer_type: Type) -> Self {
            self.peer_type = Some(peer_type);
            self
        }

        pub fn with_keys(mut self, key_start: String, key_end: String) -> Self {
            self.key_start = Some(key_start);
            self.key_end = Some(key_end);
            self
        }

        pub fn build(self) -> Peer {
            Peer {
                id: self.id.unwrap_or(Uuid::new_v4()),
                peer_type: self.peer_type.unwrap_or(Type::Tracker),
                key_start: self.key_start.unwrap_or(DEFAULT_START_KEY.to_string()),
                key_end: self.key_end.unwrap_or(DEFAULT_END_KEY.to_string()),
            }
        }
    }
}
