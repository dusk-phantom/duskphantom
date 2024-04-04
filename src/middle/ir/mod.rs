pub mod basic_block;
pub mod function;
pub mod instruction;
#[macro_use]
mod macros;
pub mod constant;
pub mod global_variable;
pub mod ir_builder;
pub mod module;
pub mod operand;
pub mod value_type;

pub use self::basic_block::{BBPtr, BasicBlock};
pub use self::function::{FunPtr, Function, ParaPtr, Parameter};
pub use self::instruction::{InstPtr, Instruction};
pub use self::module::Module;
pub use constant::Constant;
pub use global_variable::{GlobalPtr, GlobalVariable};
pub use ir_builder::IRBuilder;
pub use operand::Operand;
pub use value_type::ValueType;

use crate::utils::mem::{ObjPool, ObjPtr};
use std::collections::{HashSet, VecDeque};
use std::fmt::Display;
