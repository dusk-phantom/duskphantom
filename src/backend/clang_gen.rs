// 实现clang前端解析结果到backend的转换
use super::*;
use crate::clang_frontend;
use clang::*;
pub fn clang_gen(pro: &clang_frontend::Program) -> Result<backend::Program, CompilerError> {
    let mut program = frontend::parse(&pro.src)?;
    if pro.opt {
        frontend::optimize(&mut program);
    }
    let mut program = middle::gen(&mut program)?;
    if pro.opt {
        middle::optimize(&mut program);
    }
    let mut program = backend::gen(&mut program)?;
    if pro.opt {
        backend::optimize(&mut program);
    }
    Ok(program)
}
