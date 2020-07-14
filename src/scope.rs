use std::collections::HashMap;

#[derive(Debug)]
pub enum SymbolType {
    Variable,
    Function,
}

#[derive(Debug)]
pub struct Symbol {
    symbol_type: SymbolType,
    name: String,
}

#[derive(Debug)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            symbols: HashMap::new(),
        }
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn add(&mut self, name: String, symbol_type: SymbolType) {
        self.symbols.insert(
            name.clone(),
            Symbol {
                symbol_type,
                name: name,
            },
        );
    }
}
