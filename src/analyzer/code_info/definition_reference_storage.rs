use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum DefinitionKind {
    // Namespace
    Namespace,
    // Imports
    Use(String),
    UseFunction(String),
    UseConstant(String),
    // Items
    Constant,
    Function,
    TypeAlias,
    Interface,
    Class,
    UnitEnum,
    StringBackedEnum,
    IntBackedEnum,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct DefinitionReference {
    pub name: String,
    pub qualified_name: Option<String>,
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

    pub fn add_namespace(&mut self, name: String, source: String, position: (usize, usize)) {
        self.add(DefinitionReference {
            name,
            qualified_name: None,
            kind: DefinitionKind::Namespace,
            source,
            position,
        });
    }

    pub fn add_use(
        &mut self,
        name: String,
        symbol: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: None,
            kind: DefinitionKind::Use(symbol),
            source,
            position,
        });
    }

    pub fn add_use_function(
        &mut self,
        name: String,
        symbol: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: None,
            kind: DefinitionKind::UseFunction(symbol),
            source,
            position,
        });
    }

    pub fn add_use_constant(
        &mut self,
        name: String,
        symbol: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: None,
            kind: DefinitionKind::UseConstant(symbol),
            source,
            position,
        });
    }

    pub fn add_constant(
        &mut self,
        name: String,
        qualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: Some(qualified_name),
            kind: DefinitionKind::Constant,
            source,
            position,
        });
    }

    pub fn add_type_alias(
        &mut self,
        name: String,
        qualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: Some(qualified_name),
            kind: DefinitionKind::TypeAlias,
            source,
            position,
        });
    }

    pub fn add_function(
        &mut self,
        name: String,
        qualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: Some(qualified_name),
            kind: DefinitionKind::Function,
            source,
            position,
        });
    }

    pub fn add_interface(
        &mut self,
        name: String,
        qualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: Some(qualified_name),
            kind: DefinitionKind::Interface,
            source,
            position,
        });
    }

    pub fn add_class(
        &mut self,
        name: String,
        qualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: Some(qualified_name),
            kind: DefinitionKind::Class,
            source,
            position,
        });
    }

    pub fn add_unit_enum(
        &mut self,
        name: String,
        qualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: Some(qualified_name),
            kind: DefinitionKind::UnitEnum,
            source,
            position,
        });
    }

    pub fn add_string_backed_enum(
        &mut self,
        name: String,
        qualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: Some(qualified_name),
            kind: DefinitionKind::StringBackedEnum,
            source,
            position,
        });
    }

    pub fn add_int_backed_enum(
        &mut self,
        name: String,
        qualified_name: String,
        source: String,
        position: (usize, usize),
    ) {
        self.add(DefinitionReference {
            name,
            qualified_name: Some(qualified_name),
            kind: DefinitionKind::IntBackedEnum,
            source,
            position,
        });
    }

    pub fn get_constant(&self, fq_name: &str) -> Option<&DefinitionReference> {
        self.map.iter().find_map(|(_, definitions)| {
            definitions.iter().find(|definition| {
                if let DefinitionKind::Constant = definition.kind {
                    definition
                        .qualified_name
                        .as_ref()
                        .map(|name| name.to_lowercase() == fq_name.to_lowercase())
                        .unwrap_or(false)
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
                    definition
                        .qualified_name
                        .as_ref()
                        .map(|name| name.to_lowercase() == fq_name.to_lowercase())
                        .unwrap_or(false)
                } else {
                    false
                }
            })
        })
    }

    /// Return a type definition by its fully qualified name.
    ///
    /// This will return the first definition found in any source file.
    ///
    /// A type definition can be either:
    ///
    /// - a class
    /// - an interface
    /// - an enum
    /// - a type alias
    pub fn get_type(&self, fq_name: &str) -> Option<&DefinitionReference> {
        self.map.iter().find_map(|(_, definitions)| {
            definitions.iter().find(|definition| {
                if let DefinitionKind::Class
                | DefinitionKind::Interface
                | DefinitionKind::UnitEnum
                | DefinitionKind::IntBackedEnum
                | DefinitionKind::StringBackedEnum
                | DefinitionKind::TypeAlias = definition.kind
                {
                    definition
                        .qualified_name
                        .as_ref()
                        .map(|name| name.to_lowercase() == fq_name.to_lowercase())
                        .unwrap_or(false)
                } else {
                    false
                }
            })
        })
    }

    pub fn get_constant_name_in_source(
        &self,
        source: &str,
        name: &str,
    ) -> Option<&DefinitionReference> {
        self.map.get(source).and_then(|definitions| {
            definitions
                .iter()
                .find(|definition| match &definition.kind {
                    DefinitionKind::UseConstant(_) | DefinitionKind::Constant => {
                        definition.name.to_lowercase() == name.to_lowercase()
                    }
                    _ => false,
                })
        })
    }

    pub fn get_used_constant_in_source(
        &self,
        source: &str,
        name: &str,
    ) -> Option<&DefinitionReference> {
        self.map.get(source).and_then(|definitions| {
            definitions
                .iter()
                .find(|definition| match &definition.kind {
                    DefinitionKind::UseConstant(constant_name) => {
                        constant_name.to_lowercase() == name.to_lowercase()
                    }
                    _ => false,
                })
        })
    }

    pub fn get_function_name_in_source(
        &self,
        source: &str,
        name: &str,
    ) -> Option<&DefinitionReference> {
        self.map.get(source).and_then(|definitions| {
            definitions
                .iter()
                .find(|definition| match &definition.kind {
                    DefinitionKind::UseFunction(_) | DefinitionKind::Function => {
                        definition.name.to_lowercase() == name.to_lowercase()
                    }
                    _ => false,
                })
        })
    }

    pub fn get_used_function_in_source(
        &self,
        source: &str,
        name: &str,
    ) -> Option<&DefinitionReference> {
        self.map.get(source).and_then(|definitions| {
            definitions
                .iter()
                .find(|definition| match &definition.kind {
                    DefinitionKind::UseFunction(function_name) => {
                        function_name.to_lowercase() == name.to_lowercase()
                    }
                    _ => false,
                })
        })
    }

    pub fn get_type_name_in_source(
        &self,
        source: &str,
        name: &str,
    ) -> Option<&DefinitionReference> {
        self.map.get(source).and_then(|definitions| {
            definitions
                .iter()
                .find(|definition| match &definition.kind {
                    DefinitionKind::Use(_)
                    | DefinitionKind::TypeAlias
                    | DefinitionKind::Interface
                    | DefinitionKind::Class
                    | DefinitionKind::UnitEnum
                    | DefinitionKind::StringBackedEnum
                    | DefinitionKind::IntBackedEnum => {
                        definition.name.to_lowercase() == name.to_lowercase()
                    }
                    _ => false,
                })
        })
    }

    pub fn get_all_types_in_source(&self, source: &str) -> Vec<&DefinitionReference> {
        self.map
            .get(source)
            .map(|definitions| {
                definitions
                    .iter()
                    .filter(|definition| match &definition.kind {
                        DefinitionKind::Use(_)
                        | DefinitionKind::TypeAlias
                        | DefinitionKind::Interface
                        | DefinitionKind::Class
                        | DefinitionKind::UnitEnum
                        | DefinitionKind::StringBackedEnum
                        | DefinitionKind::IntBackedEnum => true,
                        _ => false,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn get_used_type_in_source(
        &self,
        source: &str,
        name: &str,
    ) -> Option<&DefinitionReference> {
        self.map.get(source).and_then(|definitions| {
            definitions
                .iter()
                .find(|definition| match &definition.kind {
                    DefinitionKind::Use(type_name) => {
                        type_name.to_lowercase() == name.to_lowercase()
                    }
                    _ => false,
                })
        })
    }

    pub fn get_all_in_source(&self, source: &str) -> Vec<&DefinitionReference> {
        self.map
            .get(source)
            .map(|definitions| definitions.iter().collect())
            .unwrap_or_default()
    }
}
