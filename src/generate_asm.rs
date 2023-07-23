use koopa::ir::{
    entities::ValueData, layout::BasicBlockNode, Function, FunctionData, Program, Value, ValueKind,
};

#[derive(Clone, Copy)]
pub struct ProgramInfo<'p> {
    program: &'p Program,
    which_func: Option<Function>,
}

impl<'p> ProgramInfo<'p> {
    pub fn new(program: &'p Program, which_func: Option<Function>) -> Self {
        Self {
            program,
            which_func,
        }
    }

    // pub fn program(&self) -> &'p Program {
    //     self.program
    // }

    pub fn value(&self, value: Value) -> &ValueData {
        assert_ne!(self.which_func, None);
        self.program
            .func(self.which_func.unwrap())
            .dfg()
            .value(value)
    }
}
pub trait GenerateAsm {
    fn generate(&self, info: ProgramInfo) -> String;
}

impl GenerateAsm for Program {
    fn generate(&self, _info: ProgramInfo) -> String {
        let mut ans = String::new();
        ans += "    .text\n"; // 声明之后的数据需要被放入代码段中

        // 声明全局符号
        // 遍历所有的指向函数的指针
        for &func in self.func_layout() {
            // 从指向函数的指针来获得函数本身
            let func_data = self.func(func);
            ans += "    .globl ";
            ans += &func_data.name()[1..];
            ans += "\n";
        }

        for &func in self.func_layout() {
            let func_data = self.func(func);
            ans += &func_data.generate(ProgramInfo::new(&self, Some(func)));
        }
        ans
    }
}

impl GenerateAsm for FunctionData {
    fn generate(&self, info: ProgramInfo) -> String {
        let mut ans = String::new();
        ans += &self.name()[1..];
        ans += ":\n";

        // 遍历函数，查看函数内部的基本块
        for (&_bb, node) in self.layout().bbs() {
            // 生成基本块的信息
            ans += &node.generate(info);
        }
        ans
    }
}

impl GenerateAsm for BasicBlockNode {
    fn generate(&self, info: ProgramInfo) -> String {
        let mut ans = String::new();
        // 遍历基本块里的指令(value)的指针
        for &inst in self.insts().keys() {
            // 获取指令
            let value_data = info.value(inst);
            // 处理指令
            ans += &value_data.generate(info);
        }
        ans
    }
}

impl GenerateAsm for ValueData {
    fn generate(&self, info: ProgramInfo) -> String {
        match self.kind() {
            ValueKind::Integer(int) => {
                // 处理 integer 指令
                let mut ans = String::new();
                ans += &int.value().to_string();
                ans
            }
            ValueKind::Return(ret) => {
                // 处理 ret 指令
                let mut ans = String::new();
                if ret.value() != None {
                    ans += "    li a0, ";
                    ans += &info.value(ret.value().unwrap()).generate(info);
                    ans += "\n";
                }
                ans += "    ret";
                ans
            }
            // 其他
            _ => unreachable!(),
        }
    }
}
