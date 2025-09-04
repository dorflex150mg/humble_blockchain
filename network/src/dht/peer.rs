use std::sync::Arc;
use uuid::Uuid;
use crate::object::object::{self, Object};
use crate::node::node::Node;

const DEFAULT_START_KEY: &str = "00000000";
const DEFAULT_END_KEY: &str = "FFFFFFFF";

#[derive(Debug, derive_more::From)]
pub enum PeerSendError {
    InvalidNode,
    TransportError,
}

#[derive(Debug)]
pub enum Type {
    Tracker,
    Normal,
}

#[allow(dead_code)]
pub struct Peer {
    id: Uuid,
    node: Node,
    peers: Vec<Peer>,
    peer_type: Type,
    key_start: Arc<str>,
    key_end: Arc<str>,
}


impl Peer {
    pub fn new(id: Uuid, 
        peer_type: Type,
        node: Node,
        key_start: impl Into<String>,
        key_end: impl Into<String>) -> Self {
        let key_start = key_start.into();
        let key_end = key_end.into();
        Peer {
            id,
            node,
            peers: vec![],
            peer_type,
            key_start: key_start.into(),
            key_end: key_end.into(),
        }
    }

    pub fn send_object(&self, object: Object) -> Result<(), PeerSendError> {
        let hash = object.get_hash_as_integer();
        let mut index = 0;
        while hash < object::from_string(self
            .peers[index]
            .key_start
            .clone()
            .as_ref()) {
                index += 1;
        }
        let address = self.peers[index].node.get_address();
        
        self.transport_object(object, address)?;
        Ok(())
    }        

    #[allow(unused_variables)]
    pub fn transport_object(&self, object: Object, address: Arc<str>) -> Result<(), PeerSendError> {
        Ok(())
    }
}

//impl Default for Peer {
//    fn default() -> Self {
//        Self {
//            id: Uuid::new_v4(),
//            peers: vec![],
//            node: Node::default(), 
//            peer_type: Type::Tracker,
//            key_start: DEFAULT_START_KEY.into(),
//            key_end: DEFAULT_END_KEY.into(),
//        } 
//    }
//}

#[derive(Default)]
pub struct PeerBuilder {
    id: Option<Uuid>,
    node: Option<Node>,
    peer_type: Option<Type>,
    address: Option<String>,
    key_start: Option<Arc<str>>,
    key_end: Option<Arc<str>>,
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
        self.key_start = Some(key_start.into());
        self.key_end = Some(key_end.into());
        self
    }


    pub fn with_node(mut self, node: Node) -> Self {
        self.node = Some(node);
        self
    }

    pub fn build(self) -> Peer {
        let this_key_start: Arc<str> = DEFAULT_START_KEY.into();
        let this_key_end: Arc<str> = DEFAULT_END_KEY.into();
        Peer {
            id: self.id.unwrap_or(Uuid::new_v4()),
            peers: vec![],
            node: self.node.unwrap(), 
            peer_type: self.peer_type.unwrap_or(Type::Tracker),
            key_start: self.key_start.unwrap_or(this_key_start),
            key_end: self.key_end.unwrap_or(this_key_end),
        }
    }

}

