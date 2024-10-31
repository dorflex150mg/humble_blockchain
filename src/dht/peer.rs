pub mod peer {

    use uuid::Uuid;
    use crate::object::object::object::{self, Object};

    const DEFAULT_START_KEY: &str = "00000000";
    const DEFAULT_END_KEY: &str = "FFFFFFFF";
    const DEFAULT_ADDRESS: &str = "127.0.0.1";

    #[derive(Debug, derive_more::From)]
    pub enum PeerSendError {
    }

    #[derive(Debug)]
    pub enum Type {
        Tracker,
        Normal,
    }

    #[derive(Debug)]
    pub struct Peer {
        id: Uuid,
        address: String,
        peers: Vec<Peer>,
        peer_type: Type,
        key_start: String,
        key_end: String,
    }


    impl Peer {
        pub fn new(id: Uuid, 
            peer_type: Type,
            address: impl Into<String>,
            key_start: impl Into<String>,
            key_end: impl Into<String>) -> Self {
            Peer {
                id,
                peers: vec![],
                address: address.into(),
                peer_type,
                key_start: key_start.into(),
                key_end: key_end.into(),
            }
        }

        pub fn send_object(&self, object: Object) -> Result<(), PeerSendError> {
            let hash = object.get_hash_as_integer();
            let mut index = 0;
            while hash < object::from_string(self.peers[index].key_start.clone()) {
                index += 1;
            }
            self.transport_object(object, self.peers[index].address.clone())?;
            Ok(())
        }        

        pub fn transport_object(&self, object: Object, address: String) -> Result<(), PeerSendError> {
            Ok(())
        }
    }

    impl Default for Peer {
        fn default() -> Self {
            Self {
                id: Uuid::new_v4(),
                peers: vec![],
                address: DEFAULT_ADDRESS.to_string(),
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
        address: Option<String>,
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

        pub fn with_address(mut self, address: String) -> Self {
            self.address = Some(address);
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
                peers: vec![],
                address: self.address.unwrap_or(DEFAULT_ADDRESS.to_string()),
                peer_type: self.peer_type.unwrap_or(Type::Tracker),
                key_start: self.key_start.unwrap_or(DEFAULT_START_KEY.to_string()),
                key_end: self.key_end.unwrap_or(DEFAULT_END_KEY.to_string()),
            }
        }


    }
}
