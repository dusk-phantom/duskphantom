pub mod ir;
pub mod parse;

pub use ir::*;

use crate::errors::FrontendError;

#[allow(unused)]
pub fn parse(src: &str) -> Result<Program, FrontendError> {
    parse::program::parse(src)
}

#[allow(unused)]
pub fn optimize(program: &mut Program) {}
