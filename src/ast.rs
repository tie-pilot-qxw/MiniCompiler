use std::{io::Write, sync::atomic::AtomicUsize};

static VAR_NAME: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
pub struct CompUnit {
    pub func_def: FuncDef,
}

impl CompUnit {
    pub fn generate(&self, f: &mut Vec<u8>) {
        self.func_def.generate(f);
    }
}
#[derive(Debug)]
pub struct FuncDef {
    pub func_type: FuncType,
    pub ident: String,
    pub block: Block,
}

impl FuncDef {
    fn generate(&self, f: &mut Vec<u8>) {
        write!(f, "fun @{}(): ", self.ident).unwrap();
        self.func_type.generate(f);
        writeln!(f, "{{").unwrap();
        self.block.generate(f);
        writeln!(f, "}}").unwrap();
    }
}

#[derive(Debug)]
pub enum FuncType {
    Int,
}

impl FuncType {
    fn generate(&self, f: &mut Vec<u8>) {
        write!(f, "i32 ").unwrap();
    }
}

#[derive(Debug)]
pub struct Block {
    pub stmt: Stmt,
}

impl Block {
    fn generate(&self, f: &mut Vec<u8>) {
        writeln!(f, "%entry:").unwrap();
        self.stmt.generate(f);
    }
}

#[derive(Debug)]
pub struct Stmt {
    pub exp: Exp,
}

impl Stmt {
    fn generate(&self, f: &mut Vec<u8>) {
        let var_name = self.exp.generate(f);
        writeln!(f, "    ret {}", var_name).unwrap();
    }
}

#[derive(Debug)]
pub struct Exp {
    pub unary_exp: UnaryExp,
}

impl Exp {
    fn generate(&self, f: &mut Vec<u8>) -> String {
        self.unary_exp.generate(f)
    }
}

#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    Number(Number),
}

impl PrimaryExp {
    fn generate(&self, f: &mut Vec<u8>) -> String {
        match self {
            PrimaryExp::Exp(exp) => exp.generate(f),
            PrimaryExp::Number(num) => num.generate().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Number {
    pub num: i32,
}
impl Number {
    fn generate(&self) -> i32 {
        self.num
    }
}

#[derive(Debug)]
pub enum UnaryExp {
    PrimaryExp(PrimaryExp),
    Unary(UnaryOp, Box<UnaryExp>),
}

impl UnaryExp {
    fn generate(&self, f: &mut Vec<u8>) -> String {
        match self {
            Self::PrimaryExp(p_exp) => p_exp.generate(f),

            Self::Unary(op, u_exp) => {
                let input_name = u_exp.generate(f);
                let id = VAR_NAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let output_name = format!("%{}", id.to_string());
                match op {
                    UnaryOp::Bang => {
                        writeln!(f, "    {} = eq {}, 0", output_name, input_name).unwrap();
                    }

                    UnaryOp::Negative => {
                        writeln!(f, "    {} = sub 0, {}", output_name, input_name).unwrap();
                    }
                }
                output_name
            }
        }
    }
}

#[derive(Debug)]
pub enum UnaryOp {
    Negative,
    Bang,
}
