use std::collections::HashMap;

use crate::{frontend, middle};
use anyhow::Result;
use program_kit::ProgramKit;

mod constant;
mod function_kit;
mod gen_binary;
mod gen_const_binary;
mod gen_const_expr;
mod gen_const_unary;
mod gen_expr;
mod gen_global_decl;
mod gen_impl;
mod gen_inner_decl;
mod gen_library_function;
mod gen_stmt;
mod gen_unary;
mod program_kit;
mod value;
mod value_type;

/// Generate middle IR from a frontend AST
pub fn gen(program: &frontend::Program) -> Result<middle::Program> {
    let mut result = middle::Program::new();
    ProgramKit {
        program: &mut result,
        env: &mut vec![HashMap::new()],
        fun_env: &mut vec![HashMap::new()],
    }
    .gen(program)?;
    Ok(result)
}
