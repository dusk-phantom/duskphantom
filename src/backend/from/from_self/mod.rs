pub use anyhow::Result;

mod builder;
mod instruction;
mod operand;
mod utils;
mod vars;

#[macro_use]
mod macros;

pub use builder::*;
#[allow(unused)]
pub use instruction::*;
#[allow(unused)]
pub use operand::*;

pub use super::irs::*;

/// 中端层面，地址是唯一的
/// 因此我可以将地址作为 id
/// 用在 parameter 和 instruction 上
type Address = usize;

#[allow(unused)]
pub fn gen_from_self(program: &middle::Program) -> Result<Program> {
    builder::IRBuilder::gen_from_self(program)
}
