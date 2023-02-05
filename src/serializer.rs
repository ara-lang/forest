use bincode::config;
use bincode::config::Configuration;

use crate::error::Error;
use crate::tree::SignedTree;

pub trait Serializer: Send + Sync {
    fn serialize(&self, signed_tree: &SignedTree) -> Result<Vec<u8>, Error>;
    fn deserialize(&self, data: &[u8]) -> Result<SignedTree, Error>;
}

pub struct BincodeSerializer {
    config: Configuration,
}

impl BincodeSerializer {
    pub fn new() -> Self {
        Self {
            config: config::standard(),
        }
    }
}

impl Serializer for BincodeSerializer {
    fn serialize(&self, tree: &SignedTree) -> Result<Vec<u8>, Error> {
        Ok(bincode::encode_to_vec(tree, self.config)?)
    }

    fn deserialize(&self, data: &[u8]) -> Result<SignedTree, Error> {
        let (signed_tree, _): (SignedTree, _) = bincode::decode_from_slice(data, self.config)?;

        Ok(signed_tree)
    }
}
