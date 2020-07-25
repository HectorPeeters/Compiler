use crate::types::*;
use std::collections::HashMap;

#[derive(Debug)]
pub enum SymbolType {
    Variable,
    //    Function,
}

#[derive(Debug)]
pub struct Symbol {
    symbol_type: SymbolType,
    pub primitive_type: PrimitiveType,
    name: String,
    pub offset: i32,
}

#[derive(Debug)]
pub struct Scope {
    symbols: HashMap<String, Symbol>,
    last_offset: i32,
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

    pub fn add(&mut self, name: String, symbol_type: SymbolType, primitive_type: PrimitiveType) {
        self.last_offset += primitive_type.get_size() as i32 / 8;

        self.symbols.insert(
            name.clone(),
            Symbol {
                symbol_type,
                primitive_type,
                name,
                offset: self.last_offset,
            },
        );
    }
}
