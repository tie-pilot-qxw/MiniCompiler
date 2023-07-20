use std::fmt;

#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

impl fmt::Display for CompUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.func_def.to_string())
    }
}
#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

impl fmt::Display for FuncDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "fun @{}(): {} {{\n{}}}",
            self.ident,
            self.func_type.to_string(),
            self.block.to_string()
        )
    }
}

#[derive(Debug)]
pub enum FuncType {
    Int,
}

impl fmt::Display for FuncType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "i32")
    }
}

#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%entry:\n{}\n", self.stmt.to_string())
    }
}

#[derive(Debug)]
pub struct Stmt {
    pub num: i32,
}

impl fmt::Display for Stmt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ret {}", self.num)
    }
}

#[derive(Debug)]
pub struct Number {
    pub num: i32,
}
