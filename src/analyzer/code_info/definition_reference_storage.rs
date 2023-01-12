use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum DefinitionKind {
    Namespace,
    Use(String),
    UseFunction(String),
    UseConstant(String),
    Constant,
    TypeAlias,
    Function,
    Interface,
    Class,
    UnitEnum,
    StringBackedEnum,
    IntBackedEnum,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct DefinitionReference {
    pub fq_name: String,
    pub unqualified_name: String,
    pub kind: DefinitionKind,
    pub source: String,
    pub position: (usize, usize),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct DefinitionReferenceStorage {
    pub map: FxHashMap<String, Vec<DefinitionReference>>,
}

impl DefinitionReferenceStorage {
    pub fn new() -> DefinitionReferenceStorage {
        DefinitionReferenceStorage {
            map: FxHashMap::default(),
        }
    }

    pub fn add(&mut self, definition: DefinitionReference) {
        let symbols = self
            .map
            .entry(definition.source.clone())
            .or_insert_with(Vec::new);

        symbols.push(definition);
    }

    pub fn add_constant(
        &mut self,
        fq_name: String,
        unqualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            fq_name,
            unqualified_name,
            kind: DefinitionKind::Constant,
            source,
            position,
        });
    }

    pub fn add_type_alias(
        &mut self,
        fq_name: String,
        unqualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            fq_name,
            unqualified_name,
            kind: DefinitionKind::TypeAlias,
            source,
            position,
        });
    }

    pub fn add_function(
        &mut self,
        fq_name: String,
        unqualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            fq_name,
            unqualified_name,
            kind: DefinitionKind::Function,
            source,
            position,
        });
    }

    pub fn add_interface(
        &mut self,
        fq_name: String,
        unqualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            fq_name,
            unqualified_name,
            kind: DefinitionKind::Interface,
            source,
            position,
        });
    }

    pub fn add_class(
        &mut self,
        fq_name: String,
        unqualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            fq_name,
            unqualified_name,
            kind: DefinitionKind::Class,
            source,
            position,
        });
    }

    pub fn add_unit_enum(
        &mut self,
        fq_name: String,
        unqualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            fq_name,
            unqualified_name,
            kind: DefinitionKind::UnitEnum,
            source,
            position,
        });
    }

    pub fn add_string_backed_enum(
        &mut self,
        fq_name: String,
        unqualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            fq_name,
            unqualified_name,
            kind: DefinitionKind::StringBackedEnum,
            source,
            position,
        });
    }

    pub fn add_int_backed_enum(
        &mut self,
        fq_name: String,
        unqualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            fq_name,
            unqualified_name,
            kind: DefinitionKind::IntBackedEnum,
            source,
            position,
        });
    }

    pub fn get_constant(&self, fq_name: &str) -> Option<&DefinitionReference> {
        self.map.iter().find_map(|(_, definitions)| {
            definitions.iter().find(|definition| {
                if let DefinitionKind::Constant = definition.kind {
                    definition.fq_name == fq_name
                } else {
                    false
                }
            })
        })
    }

    pub fn get_function(&self, fq_name: &str) -> Option<&DefinitionReference> {
        self.map.iter().find_map(|(_, definitions)| {
            definitions.iter().find(|definition| {
                if let DefinitionKind::Function = definition.kind {
                    definition.fq_name == fq_name
                } else {
                    false
                }
            })
        })
    }

    pub fn get_classish(&self, fq_name: &str) -> Option<&DefinitionReference> {
        self.map.iter().find_map(|(_, definitions)| {
            definitions.iter().find(|definition| {
                if let DefinitionKind::Class
                | DefinitionKind::Interface
                | DefinitionKind::UnitEnum
                | DefinitionKind::IntBackedEnum
                | DefinitionKind::StringBackedEnum
                | DefinitionKind::TypeAlias = definition.kind
                {
                    definition.fq_name == fq_name
                } else {
                    false
                }
            })
        })
    }

    pub fn get_definitions_in_source(&self, source: &str) -> Vec<&DefinitionReference> {
        self.map
            .get(source)
            .map(|definitions| definitions.iter().collect())
            .unwrap_or_default()
    }
}
