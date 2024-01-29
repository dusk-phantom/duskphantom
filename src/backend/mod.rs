pub mod block;
pub mod func;
pub mod gen_asm;
pub mod inst;
pub mod module;
pub mod prog;
pub mod var;

use rayon::prelude::*;

use crate::{clang_frontend, errors::BackendError, middle};

#[allow(unused)]
pub fn gen(program: &middle::Program) -> Result<prog::Program, BackendError> {
    // TODO
    Ok(prog::Program {
        entry: None,
        modules: vec![],
    })
}

#[allow(unused)]
pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<prog::Program, BackendError> {
    // TODO
    Ok(prog::Program {
        entry: None,
        modules: vec![],
    })
}

#[allow(unused)]
pub fn optimize(program: &mut prog::Program) {
    // 乘除法优化
    // 块重排
    // 指令调度
    // 寄存器分配与合并
}

#[allow(unused)]
pub fn asm2bin(asm: String) -> String {
    panic!("not implemented")
}
