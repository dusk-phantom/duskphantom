pub mod basic_block;
pub mod function;
pub mod instruction;
#[macro_use]
mod macros;
pub mod ir_builder;
pub mod module;
pub mod value_type;

pub use self::basic_block::{BBPtr, BasicBlock};
pub use self::function::{FunPtr, Function};
pub use self::instruction::{InstPtr, Instruction};
pub use self::module::Module;

use crate::utils::mem::{ObjPool, ObjPtr};
use ir_builder::IRBuilder;
use std::collections::{HashSet, VecDeque};
use value_type::ValueType;
