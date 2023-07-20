use koopa::ir::{Program, FunctionData, entities::ValueData, layout::BasicBlockNode, ValueKind};

trait GenerateAsm {
    fn generate(&self);
}

impl GenerateAsm for Program {
    fn generate(&self) {
        // 遍历所有的指向函数的指针
        for &func in self.func_layout() {
            // 从指向函数的指针来获得函数本身
            let func_data = self.func(func);
            func_data.generate();
        }
    }
}

impl GenerateAsm for FunctionData {
    fn generate(&self) {
        // 遍历函数，查看函数内部的基本块
        for (&bb, node) in self.layout().bbs() {
            // 生成基本块的信息
            node.generate();
            // 遍历基本块里的指令(value)的指针
            for &inst in node.insts().keys() {
                // 获取指令
                let value_data = self.dfg().value(inst);
                // 处理指令
                value_data.generate();
            }
            
        }
    }
}

impl GenerateAsm for BasicBlockNode {
    fn generate(&self) {
        
    }
}

impl GenerateAsm for ValueData {
    fn generate(&self) {
        
        match self.kind() {
            ValueKind::Integer(int) => {
                // 处理 integer 指令

            }
            ValueKind::Return(ret) => {
                // 处理 ret 指令

            }
            // 其他
            _ => unreachable!()
        }
    }
}