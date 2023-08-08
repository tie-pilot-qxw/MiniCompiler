use std::collections::HashMap;

pub(super) enum DataType {
    ConstInt(i32),
    Int,
}

pub(super) struct SymbolTable {
    pub var: HashMap<String, DataType>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { var: HashMap::new() }
    }
}