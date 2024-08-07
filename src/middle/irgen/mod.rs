use crate::{frontend, middle, utils::frame_map::FrameMap};
use anyhow::Result;
use program_kit::ProgramKit;

mod function_kit;
mod gen_binary;
mod gen_const;
mod gen_expr;
mod gen_global_decl;
mod gen_impl;
mod gen_inner_decl;
mod gen_library_function;
mod gen_stmt;
mod gen_type;
mod gen_unary;
mod program_kit;
mod value;

/// Generate middle IR from a frontend AST
pub fn gen(program: &frontend::Program) -> Result<middle::Program> {
    let mut result = middle::Program::new();
    ProgramKit {
        program: &mut result,
        env: FrameMap::new(),
        fun_env: FrameMap::new(),
    }
    .gen(program)?;
    Ok(result)
}
