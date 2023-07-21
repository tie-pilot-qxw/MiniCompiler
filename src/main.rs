use koopa::ir::ValueKind;
use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::{read_to_string, write};
use std::io::Result;
mod ast;
mod generate_asm;
use generate_asm::{ProgramInfo, GenerateAsm};

// 引用 lalrpop 生成的解析器
// 因为我们刚刚创建了 sysy.lalrpop, 所以模块名是 sysy
lalrpop_mod!(sysy);

fn main() -> Result<()> {
    // 解析命令行参数
    let mut args = args();
    args.next();
    let mode = args.next().unwrap();
    let input = args.next().unwrap();
    args.next();
    let mut output = args.next().unwrap();

    // 读取输入文件
    let input = read_to_string(input)?;

    // 调用 lalrpop 生成的 parser 解析输入文件
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();

    // 输出解析得到的AST，转换成koopa ir
    let koopa_ir = ast.to_string();
    // 调用库将koopa ir转换成koopa ir对应的AST
    let driver = koopa::front::Driver::from(ast.to_string());
    let program = driver.generate_program().unwrap();

    /*
    // 遍历所有的指向函数的指针
    for &func in program.func_layout() {
        // 从指向函数的指针来获得函数本身
        let func_data = program.func(func);

        // 遍历函数，查看函数内部的基本块
        for (&bb, node) in func_data.layout().bbs() {


            // 遍历基本块里的指令(value)的指针
            for &inst in node.insts().keys() {
                // 获取指令
                let value_data = func_data.dfg().value(inst);
                
                // 处理指令
                match value_data.kind() {
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
    }
    */
    println!("{}",mode);
    if mode == "-koopa" {
        write(&mut output, ast.to_string()).unwrap();
        println!("{}", ast);
    } else {
        write(&mut output, program.generate(ProgramInfo::new(&program, None))).unwrap();
        println!("{}", program.generate(ProgramInfo::new(&program, None)));
    }
    Ok(())
}
