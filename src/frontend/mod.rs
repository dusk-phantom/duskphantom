pub mod ir;
pub mod parse;
pub mod preprocess;
pub mod transform;

pub use ir::*;
use transform::constant_fold;

use crate::errors::FrontendError;

#[allow(unused)]
pub fn parse(src: &str) -> Result<Program, FrontendError> {
    let preprocessed = preprocess::timing::process(src);
    let mut program = parse::program::parse(&preprocessed)?;
    match constant_fold::optimize_program(&mut program) {
        Ok(_) => Ok(program),
        Err(e) => Err(FrontendError::OptimizeError),
    }
}

#[allow(unused)]
pub fn optimize(program: &mut Program) {}
