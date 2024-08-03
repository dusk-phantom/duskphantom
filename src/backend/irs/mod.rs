pub mod block;
#[allow(clippy::legacy_numeric_constants)]
pub mod func;
// pub mod inst;
#[macro_use]
mod macros;

pub mod instruction;
pub mod module;
pub mod operand;
pub mod prog;
pub mod reg_set;
pub mod stack_slot;
pub mod var;

pub use super::*;
pub use block::*;
pub use func::*;
pub use instruction::*;
pub use module::*;
pub use operand::*;
pub use prog::*;
pub use stack_slot::*;
pub use var::*;
