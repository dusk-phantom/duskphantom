mod builder;
mod instruction;
#[macro_use]
mod macros;
mod operand;

pub use super::irs::*;
pub use crate::clang_frontend;
pub use crate::context;
pub use anyhow::{anyhow, Context, Result};

#[cfg(feature = "clang_enabled")]
#[allow(unused)]
pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<Program> {
    builder::IRBuilder::gen_from_clang(program)
}
