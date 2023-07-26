use std::{io::Write, sync::atomic::AtomicUsize};

static VAR_NAME: AtomicUsize = AtomicUsize::new(0);

fn gen_var_name() -> String {
    let id = VAR_NAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("%{}", id.to_string())
}

fn to_logic(var: String, f: &mut Vec<u8>) -> String {
    let output_name = gen_var_name();
    writeln!(f, "    {} = ne {}, {}", output_name, var, "0").unwrap();
    output_name
}

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
    pub l_or_exp: LOrExp,
}

impl Exp {
    fn generate(&self, f: &mut Vec<u8>) -> String {
        self.l_or_exp.generate(f)
    }
}

#[derive(Debug)]
pub enum LOrExp {
    LAndExp(LAndExp),
    Or(Box<LOrExp>, LAndExp),
}

impl LOrExp {
    pub fn generate(&self, f: &mut Vec<u8>) -> String {
        match self {
            Self::LAndExp(l_and_exp) => l_and_exp.generate(f),

            Self::Or(l_or_exp, l_and_exp) => {
                let v1 = to_logic(l_or_exp.generate(f), f);
                let v2 = to_logic(l_and_exp.generate(f), f);
                let output_name = gen_var_name();
                writeln!(f, "    {} = or {}, {}", output_name, v1, v2).unwrap();
                output_name
            }
        }
    }
}

#[derive(Debug)]
pub enum LAndExp {
    EqExp(EqExp),
    And(Box<LAndExp>, EqExp),
}

impl LAndExp {
    fn generate(&self, f: &mut Vec<u8>) -> String {
        match self {
            Self::And(l_and_exp, eq_exp) => {
                let v1 = to_logic(l_and_exp.generate(f), f);
                let v2 = to_logic(eq_exp.generate(f), f);
                let output_name = gen_var_name();
                writeln!(f, "    {} = and {}, {}", output_name, v1, v2).unwrap();
                output_name
            }

            Self::EqExp(eq_exp) => eq_exp.generate(f),
        }
    }
}

#[derive(Debug)]
pub enum EqExp {
    RelExp(RelExp),
    Eq(Box<EqExp>, EqSign, RelExp),
}

impl EqExp {
    fn generate(&self, f: &mut Vec<u8>) -> String {
        match self {
            Self::Eq(eq_exp, sign, rel_exp) => {
                let v1 = eq_exp.generate(f);
                let v2 = rel_exp.generate(f);
                let output_name = gen_var_name();
                match sign {
                    EqSign::Eq => {
                        writeln!(f, "    {} = eq {}, {}", output_name, v1, v2).unwrap();
                    }
                    EqSign::Neq => {
                        writeln!(f, "    {} = ne {}, {}", output_name, v1, v2).unwrap();
                    }
                }
                output_name
            }

            Self::RelExp(rel_exp) => rel_exp.generate(f),
        }
    }
}

#[derive(Debug)]
pub enum EqSign {
    Eq,
    Neq,
}

#[derive(Debug)]
pub enum RelExp {
    AddExp(AddExp),
    Cmp(Box<RelExp>, CmpSign, AddExp),
}

impl RelExp {
    fn generate(&self, f: &mut Vec<u8>) -> String {
        match self {
            Self::AddExp(add_exp) => add_exp.generate(f),

            Self::Cmp(rel_exp, sign, add_exp) => {
                let v1 = rel_exp.generate(f);
                let v2 = add_exp.generate(f);
                let output_name = gen_var_name();
                match sign {
                    CmpSign::Leq => {
                        writeln!(f, "    {} = le {}, {}", output_name, v1, v2).unwrap();
                    }
                    CmpSign::Less => {
                        writeln!(f, "    {} = lt {}, {}", output_name, v1, v2).unwrap();
                    }
                    CmpSign::Meq => {
                        writeln!(f, "    {} = ge {}, {}", output_name, v1, v2).unwrap();
                    }
                    CmpSign::More => {
                        writeln!(f, "    {} = gt {}, {}", output_name, v1, v2).unwrap();
                    }
                }
                output_name
            }
        }
    }
}

#[derive(Debug)]
pub enum CmpSign {
    Less,
    More,
    Leq,
    Meq,
}

#[derive(Debug)]
pub enum AddExp {
    MulExp(MulExp),
    AddExp(Box<AddExp>, AddSign, MulExp),
}

impl AddExp {
    fn generate(&self, f: &mut Vec<u8>) -> String {
        match self {
            Self::MulExp(mul_exp) => mul_exp.generate(f),

            Self::AddExp(add_exp, sign, mul_exp) => {
                let v1 = add_exp.generate(f);
                let v2 = mul_exp.generate(f);
                let output_name = gen_var_name();

                match sign {
                    AddSign::Add => {
                        writeln!(f, "    {} = add {}, {}", output_name, v1, v2).unwrap();
                    }

                    AddSign::Sub => {
                        writeln!(f, "    {} = sub {}, {}", output_name, v1, v2).unwrap();
                    }
                }
                output_name
            }
        }
    }
}

#[derive(Debug)]
pub enum AddSign {
    Add,
    Sub,
}

#[derive(Debug)]
pub enum MulExp {
    UnaryExp(UnaryExp),
    MulExp(Box<MulExp>, MulSign, UnaryExp),
}

impl MulExp {
    fn generate(&self, f: &mut Vec<u8>) -> String {
        match self {
            Self::MulExp(mul_exp, sign, unary_exp) => {
                let v1 = mul_exp.generate(f);
                let v2 = unary_exp.generate(f);
                let output_name = gen_var_name();

                match sign {
                    MulSign::Div => {
                        writeln!(f, "    {} = div {}, {}", output_name, v1, v2).unwrap();
                    }
                    MulSign::Mod => {
                        writeln!(f, "    {} = mod {}, {}", output_name, v1, v2).unwrap();
                    }
                    MulSign::Mul => {
                        writeln!(f, "    {} = mul {}, {}", output_name, v1, v2).unwrap();
                    }
                }
                output_name
            }

            Self::UnaryExp(unary_exp) => unary_exp.generate(f),
        }
    }
}

#[derive(Debug)]
pub enum MulSign {
    Mul,
    Div,
    Mod,
}

#[derive(Debug)]
pub enum PrimaryExp {
    Exp(Box<Exp>),
    Number(Number),
}

impl PrimaryExp {
    fn generate(&self, f: &mut Vec<u8>) -> String {
        match self {
            Self::Exp(exp) => exp.generate(f),
            Self::Number(num) => num.generate().to_string(),
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
                let output_name = gen_var_name();
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
