use crate::types::*;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub enum SymbolType {
    Variable,
    //    Function,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub symbol_type: SymbolType,
    pub primitive_type: PrimitiveType,
    pub name: String,
    pub offset: i32,
}

#[derive(Debug)]
pub struct Scope {
    pub symbols: HashMap<String, Symbol>,
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

    pub fn add(
        &mut self,
        name: &String,
        symbol_type: SymbolType,
        primitive_type: PrimitiveType,
    ) -> Symbol {
        self.last_offset += primitive_type.get_size() as i32 / 8;

        let symbol = Symbol {
            symbol_type,
            primitive_type,
            name: name.clone(),
            offset: self.last_offset,
        };
        self.symbols.insert(name.clone(), symbol.clone());

        symbol
    }
}
