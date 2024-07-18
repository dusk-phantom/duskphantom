pub use anyhow::Result;

mod builder;
mod instruction;
mod operand;

#[macro_use]
mod macros;

pub use builder::*;
#[allow(unused)]
pub use instruction::*;
#[allow(unused)]
pub use operand::*;

use super::super::prog;
use crate::errors::*;
use crate::middle;

/// 中端层面，地址是唯一的
/// 因此我可以将地址作为 id
/// 用在 parameter 和 instruction 上
type Address = usize;

#[allow(unused)]
pub fn gen(program: &middle::Program) -> Result<prog::Program, BackendError> {
    // TODO
    Ok(prog::Program {
        entry: None,
        modules: vec![],
    })
}
