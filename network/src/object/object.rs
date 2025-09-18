//use std::num::ParseIntError;
//
//use xxhash_rust::xxh3::xxh3_64;
//
//pub struct Object {
//    bytes: Vec<u8>,
//}
//
//impl Object {
//    pub fn get_hash(&self) -> String {
//        let hash = xxh3_64(&self.bytes);
//        format!("{:x}", hash).to_string()
//    }
//
//    pub fn get_hash_as_integer(&self) -> u64 {
//        xxh3_64(&self.bytes)
//    }
//}
//
//pub fn from_string(string: &str) -> Result<u64, ParseIntError> {
//    u64::from_str_radix(string, 16)
//}
