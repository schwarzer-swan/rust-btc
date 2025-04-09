use crate::U256;
use serde;
use sha256::digest;
use std::fmt;

#[derive(Clone, Copy, serde::Serialize, serde::Deserialize, Debug, PartialEq, Eq, Hash)]
pub struct Hash(U256);

impl Hash {
    pub fn hash<T: serde::Serialize>(data: &T) -> Self {
        let mut serialized: Vec<u8> = vec![];
        if let Err(e) = ciborium::into_writer(data, &mut serialized) {
            panic!("Failed to serialize data: {:?}. This should not happen", e);
        }
        let hash = digest(&serialized); // compute hash of the CBOR
        let hash_bytes = hex::decode(hash).unwrap(); // decode to hex
        let hash_array: [u8; 32] = hash_bytes.as_slice().try_into().unwrap(); // as array of bytes
        Hash(U256::try_from(hash_array).unwrap()) // conver to U256
    }
    pub fn as_bytes(&self) -> [u8; 32] {
        let mut bytes: Vec<u8> = vec![0; 32];
        self.0.to_little_endian(&mut bytes);
        bytes.as_slice().try_into().unwrap()
    }
    pub fn matches_target(
        &self,
        target: U256, // network difficutly
    ) -> bool {
        self.0 < target
    }
    pub fn zero() -> Self {
        Hash(U256::zero())
    }
}
impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:x}", self.0)
    }
}
