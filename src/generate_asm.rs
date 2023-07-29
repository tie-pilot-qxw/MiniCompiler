use std::io::Write;
use std::{collections::HashMap, sync::atomic::AtomicUsize};

use koopa::ir::{
    entities::ValueData,
    layout::BasicBlockNode,
    values::{Binary, Integer, Return},
    BinaryOp, Function, FunctionData, Program, Value, ValueKind,
};

static RIG_NAME: AtomicUsize = AtomicUsize::new(0);

fn gen_rig_name() -> String {
    let id = RIG_NAME.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    format!("t{}", id.to_string())
}

#[derive(Clone)]
pub struct ProgramInfo<'p> {
    program: &'p Program,
    which_func: Option<Function>,
    values: HashMap<Value, String>,
    cur_value: Option<Value>,
}

impl<'p> ProgramInfo<'p> {
    pub fn new(program: &'p Program, which_func: Option<Function>) -> Self {
        Self {
            program,
            which_func,
            values: HashMap::new(),
            cur_value: None,
        }
    }

    fn get_key(&self) -> Value {
        self.cur_value.unwrap()
    }

    fn set_key(&mut self, key: Value) {
        self.cur_value = Some(key);
    }

    fn get_data(&self, value: Value) -> &ValueData {
        assert_ne!(self.which_func, None);
        self.program
            .func(self.which_func.unwrap())
            .dfg()
            .value(value)
    }

    fn set_func(&mut self, func: Function) {
        self.which_func = Some(func);
    }

    fn add_value(&mut self, key: Value, name: String) {
        self.values.insert(key, name);
    }

    fn query_value(&self, key: Value) -> Option<&String> {
        self.values.get(&key)
    }
}
pub trait GenerateAsm {
    fn generate(&self, info: &mut ProgramInfo, f: &mut Vec<u8>) -> Option<String>;
}

impl GenerateAsm for Program {
    fn generate(&self, info: &mut ProgramInfo, f: &mut Vec<u8>) -> Option<String> {
        writeln!(f, "    .text").unwrap(); // 声明之后的数据需要被放入代码段中

        // 声明全局符号
        // 遍历所有的指向函数的指针
        for &func in self.func_layout() {
            // 从指向函数的指针来获得函数本身
            let func_data = self.func(func);
            writeln!(f, "    .globl {}", &func_data.name()[1..]).unwrap();
        }

        for &func in self.func_layout() {
            let func_data = self.func(func);
            info.set_func(func);
            func_data.generate(info, f);
        }
        None
    }
}

impl GenerateAsm for FunctionData {
    fn generate(&self, info: &mut ProgramInfo, f: &mut Vec<u8>) -> Option<String> {
        writeln!(f, "{}:", &self.name()[1..]).unwrap();

        // 遍历函数，查看函数内部的基本块
        for (&_bb, node) in self.layout().bbs() {
            // 生成基本块的信息
            node.generate(info, f);
        }
        None
    }
}

impl GenerateAsm for BasicBlockNode {
    fn generate(&self, info: &mut ProgramInfo, f: &mut Vec<u8>) -> Option<String> {
        // 遍历基本块里的指令(value)的指针
        for &inst in self.insts().keys() {
            // 获取指令
            let value_data = info.get_data(inst).clone();
            // 处理指令
            info.set_key(inst);
            value_data.generate(info, f);
        }
        None
    }
}

impl GenerateAsm for ValueData {
    fn generate(&self, info: &mut ProgramInfo, f: &mut Vec<u8>) -> Option<String> {
        match self.kind() {
            ValueKind::Integer(int) => int.generate(info, f),
            ValueKind::Return(ret) => ret.generate(info, f),
            ValueKind::Binary(bin) => bin.generate(info, f),
            // 其他
            _ => unreachable!(),
        }
    }
}

impl GenerateAsm for Integer {
    fn generate(&self, _info: &mut ProgramInfo, f: &mut Vec<u8>) -> Option<String> {
        // 处理 integer 指令
        let val = self.value();
        if val == 0 {
            return Some("x0".to_owned());
        }
        let output = gen_rig_name();
        writeln!(f, "    li {output}, {val}").unwrap();
        Some(output)
    }
}

impl GenerateAsm for Return {
    fn generate(&self, info: &mut ProgramInfo, f: &mut Vec<u8>) -> Option<String> {
        // 处理 ret 指令
        if self.value() != None {
            let tmp = info.get_key();
            info.set_key(self.value().unwrap());
            let ret = info
                .get_data(self.value().unwrap())
                .clone()
                .generate(info, f);
            writeln!(f, "    mv a0, {}", ret.unwrap()).unwrap();
            info.set_key(tmp);
        }
        writeln!(f, "    ret").unwrap();
        None
    }
}

impl GenerateAsm for Binary {
    fn generate(&self, info: &mut ProgramInfo, f: &mut Vec<u8>) -> Option<String> {
        let find = info.query_value(info.get_key());
        if find != None {
            return Some(find.unwrap().clone());
        }
        let tmp = info.get_key();
        match self.op() {
            BinaryOp::Eq => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    xor {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                writeln!(f, "    seqz {output}, {output}").unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::Sub => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    sub {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::Add => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    add {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::Div => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    div {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::Mul => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    mul {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::Mod => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    rem {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::Le => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    sgt {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                writeln!(f, "    seqz {output}, {output}").unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::Ge => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    slt {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                writeln!(f, "    seqz {output}, {output}").unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::And => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    and {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::Gt => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    sgt {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::Lt => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    slt {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::Or => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    or {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            BinaryOp::NotEq => {
                info.set_key(self.lhs());
                let lhs = info.get_data(self.lhs()).clone().generate(info, f);
                info.set_key(self.rhs());
                let rhs = info.get_data(self.rhs()).clone().generate(info, f);
                let output = gen_rig_name();
                writeln!(f, "    xor {output}, {}, {}", lhs.unwrap(), rhs.unwrap()).unwrap();
                writeln!(f, "    snez {output}, {output}").unwrap();
                // 表明当前value已经处理过了
                info.set_key(tmp);
                info.add_value(info.get_key(), output.clone());
                Some(output)
            }

            _ => unreachable!(),
        }
    }
}
