//use crate::node::node::Node;
//use crate::object::object::{self, Object};
//use std::sync::Arc;
//use uuid::Uuid;
//
//const DEFAULT_START_KEY: &str = "00000000";
//const DEFAULT_END_KEY: &str = "FFFFFFFF";
//
//#[derive(Debug, derive_more::From)]
//pub enum PeerSendError {
//    InvalidNode,
//    TransportError,
//    InvalidKey,
//}
//
//#[derive(Debug)]
//pub enum Type {
//    Tracker,
//    Normal,
//}
//
//#[allow(dead_code)]
//pub struct Peer {
//    id: Uuid,
//    node: Node,
//    peers: Vec<Peer>,
//    peer_type: Type,
//    key_start: Arc<str>,
//    key_end: Arc<str>,
//}
//
//impl Peer {
//    pub fn new(
//        id: Uuid,
//        peer_type: Type,
//        node: Node,
//        key_start: impl Into<String>,
//        key_end: impl Into<String>,
//    ) -> Self {
//        let key_start = key_start.into();
//        let key_end = key_end.into();
//        Peer {
//            id,
//            node,
//            peers: vec![],
//            peer_type,
//            key_start: key_start.into(),
//            key_end: key_end.into(),
//        }
//    }
//
//    pub fn send_object(&self, object: Object) -> Result<(), PeerSendError> {
//        let hash = object.get_hash_as_integer();
//        let mut index = 0;
//        while hash
//            < object::from_string(self.peers[index].key_start.clone().as_ref())
//                .map_err(|_| PeerSendError::InvalidKey)?
//        {
//            index += 1;
//        }
//        let address = self.peers[index].node.get_address();
//
//        self.transport_object(object, address)?;
//        Ok(())
//    }
//
//    #[allow(unused_variables)]
//    pub fn transport_object(&self, object: Object, address: Arc<str>) -> Result<(), PeerSendError> {
//        Ok(())
//    }
//}
