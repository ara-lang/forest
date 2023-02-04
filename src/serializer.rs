use bincode::config;
use bincode::config::Configuration;

use ara_parser::tree::definition::DefinitionTree;

use crate::error::Error;

pub trait Serializer: Send + Sync {
    fn serialize(&self, definitions: &DefinitionTree) -> Result<Vec<u8>, Error>;
    fn deserialize(&self, encoded: &[u8]) -> Result<DefinitionTree, Error>;
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
    fn serialize(&self, definitions: &DefinitionTree) -> Result<Vec<u8>, Error> {
        Ok(bincode::encode_to_vec(definitions, self.config)?)
    }

    fn deserialize(&self, data: &[u8]) -> Result<DefinitionTree, Error> {
        let (definitions, _): (DefinitionTree, _) = bincode::decode_from_slice(data, self.config)?;

        Ok(definitions)
    }
}
