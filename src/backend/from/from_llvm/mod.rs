mod builder;
mod operand;
mod instruction;

pub use super::irs::*;
pub use crate::context;
pub use crate::clang_frontend;
pub use anyhow::{anyhow,Result,Context};


#[cfg(feature = "clang_enabled")]
#[allow(unused)]
pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<Program> {
    builder::IRBuilder::gen_from_clang(program)
}
