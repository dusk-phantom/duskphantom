pub mod ir;
pub mod parse;
pub mod transform;

pub use ir::*;
use transform::constant_fold;

use crate::errors::FrontendError;

#[allow(unused)]
pub fn parse(src: &str) -> Result<Program, FrontendError> {
    let mut program = parse::program::parse(src)?;
    constant_fold::optimize_program(&mut program);
    Ok(program)
}

#[allow(unused)]
pub fn optimize(program: &mut Program) {}
