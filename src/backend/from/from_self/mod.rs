pub use anyhow::Result;

mod builder;
mod instruction;
mod operand;

pub use builder::*;
#[allow(unused)]
pub use instruction::*;
#[allow(unused)]
pub use operand::*;

use super::super::prog;
use crate::errors::*;
use crate::middle;

#[allow(unused)]
pub fn gen(program: &middle::Program) -> Result<prog::Program, BackendError> {
    // TODO
    Ok(prog::Program {
        entry: None,
        modules: vec![],
    })
}
