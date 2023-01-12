use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum SymbolKind {
    Interface,
    Class,
    Enum,
    TypeAlias,
    Function,
    Constant,
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub source: String,
    pub position: (usize, usize),
}

#[derive(Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct SourceSymbols {
    pub source: String,
    pub symbols: Vec<Symbol>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct CodeBaseSymbols {
    pub symbols: Vec<Symbol>,
}

impl CodeBaseSymbols {
    pub fn new() -> CodeBaseSymbols {
        CodeBaseSymbols {
            symbols: Vec::default(),
        }
    }

    pub fn add(&mut self, symbol: Symbol) {
        self.symbols.push(symbol);
    }

    pub fn add_class(&mut self, fq_name: String, source: String, position: (usize, usize)) {
        self.add(Symbol {
            name: fq_name,
            kind: SymbolKind::Class,
            source,
            position,
        });
    }

    pub fn add_interface(&mut self, fq_name: String, source: String, position: (usize, usize)) {
        self.add(Symbol {
            name: fq_name,
            kind: SymbolKind::Interface,
            source,
            position,
        });
    }

    pub fn add_enum(&mut self, fq_name: String, source: String, position: (usize, usize)) {
        self.add(Symbol {
            name: fq_name,
            kind: SymbolKind::Enum,
            source,
            position,
        });
    }

    pub fn add_type_alias(&mut self, fq_name: String, source: String, position: (usize, usize)) {
        self.add(Symbol {
            name: fq_name,
            kind: SymbolKind::TypeAlias,
            source,
            position,
        });
    }

    pub fn add_function(&mut self, fq_name: String, source: String, position: (usize, usize)) {
        self.add(Symbol {
            name: fq_name,
            kind: SymbolKind::Function,
            source,
            position,
        });
    }

    pub fn add_constant(&mut self, fq_name: String, source: String, position: (usize, usize)) {
        self.add(Symbol {
            name: fq_name,
            kind: SymbolKind::Constant,
            source,
            position,
        });
    }

    pub fn get_classish(&self, fq_name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|symbol| {
            if let SymbolKind::Class
            | SymbolKind::Interface
            | SymbolKind::Enum
            | SymbolKind::TypeAlias = symbol.kind
            {
                symbol.name == fq_name
            } else {
                false
            }
        })
    }

    pub fn get_function(&self, fq_name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|symbol| {
            if let SymbolKind::Function = symbol.kind {
                symbol.name == fq_name
            } else {
                false
            }
        })
    }

    pub fn get_constant(&self, fq_name: &str) -> Option<&Symbol> {
        self.symbols.iter().find(|symbol| {
            if let SymbolKind::Constant = symbol.kind {
                symbol.name == fq_name
            } else {
                false
            }
        })
    }

    pub fn get_symbols_in_source(&self, source: &str) -> Vec<&Symbol> {
        self.symbols
            .iter()
            .filter(|symbol| symbol.source == source)
            .collect()
    }
}
