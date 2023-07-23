use lalrpop_util::lalrpop_mod;
use std::env::args;
use std::fs::{read_to_string, write};
use std::io::Result;
mod ast;
mod generate_asm;
use generate_asm::{GenerateAsm, ProgramInfo};

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
    let output = args.next().unwrap();

    // 读取输入文件
    let input = read_to_string(input)?;

    // 调用 lalrpop 生成的 parser 解析输入文件
    let ast = sysy::CompUnitParser::new().parse(&input).unwrap();

    // println!("{:#?}", ast);

    let mut buf = Vec::new();
    ast.generate(&mut buf);
    let koopa_ir = String::from_utf8(buf).unwrap();

    // 调用库将koopa ir转换成koopa ir对应的AST
    let driver = koopa::front::Driver::from(koopa_ir.clone());
    let program = driver.generate_program().unwrap();

    if mode == "-koopa" {
        write(&output, koopa_ir.clone()).unwrap();
        println!("{}", koopa_ir);
    } else {
        write(&output, program.generate(ProgramInfo::new(&program, None))).unwrap();
        println!("{}", program.generate(ProgramInfo::new(&program, None)));
    }
    Ok(())
}
