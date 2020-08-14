use crate::types::*;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum SymbolType {
    Variable,
    Function,
    FunctionParameter
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub symbol_type: SymbolType,
    pub primitive_type: PrimitiveType,
    pub parameter_types: Vec<PrimitiveType>,
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
        //TODO: add symbol type check
        self.symbols.get(name)
    }

    pub fn add(
        &mut self,
        name: &String,
        primitive_type: PrimitiveType,
        parameter_types: Vec<PrimitiveType>,
        symbol_type: SymbolType,
    ) -> Symbol {        
        self.last_offset += primitive_type.get_size() as i32 / 8;

        let symbol = Symbol {
            symbol_type,
            primitive_type,
            parameter_types,
            name: name.clone(),
            offset: self.last_offset,
        };
        self.symbols.insert(name.clone(), symbol.clone());

        symbol
    }

    pub fn add_with_offset(
        &mut self,
        name: &String,
        primitive_type: PrimitiveType,
        parameter_types: Vec<PrimitiveType>,
        symbol_type: SymbolType,
        offset: i32,
    ) -> Symbol {        
        self.last_offset += primitive_type.get_size() as i32 / 8;

        let symbol = Symbol {
            symbol_type,
            primitive_type,
            parameter_types,
            name: name.clone(),
            offset
        };
        self.symbols.insert(name.clone(), symbol.clone());

        symbol
    }
}
