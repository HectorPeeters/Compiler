use std::collections::HashMap;

#[derive(Debug)]
pub enum SymbolType {
    Variable,
//    Function,
}

#[derive(Debug)]
pub struct Symbol {
    symbol_type: SymbolType,
    name: String,
    pub offset: i32,
}

//TODO: replace by getter of function
#[derive(Debug)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
    pub last_offset: i32,
}

impl Scope {
    pub fn new() -> Self {
        Scope {
            symbols: HashMap::new(),
            last_offset: 0,
        }
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn add(&mut self, name: String, symbol_type: SymbolType, offset: i32) {
        self.symbols.insert(
            name.clone(),
            Symbol {
                symbol_type,
                name,
                offset,
            },
        );
    }
}
