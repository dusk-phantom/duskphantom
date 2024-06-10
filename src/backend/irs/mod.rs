pub mod block;
pub mod func;
// pub mod inst;
#[macro_use]
mod macros;

pub mod module;
pub mod operand;
pub mod prog;
pub mod stack_slot;
pub mod var;
mod insts;

pub use super::*;
pub use block::*;
pub use func::*;
pub use module::*;
pub use operand::*;
pub use prog::*;
pub use stack_slot::*;
pub use insts::*;
