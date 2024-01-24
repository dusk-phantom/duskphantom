pub mod basic_block;
pub mod function;
pub mod instruction;
pub mod module;
pub mod program_mem_pool;
pub mod value_type;

pub use self::basic_block::{BBPtr, BasicBlock};
pub use self::function::{FunPtr, Function};
pub use self::instruction::{InstPtr, Instruction};
pub use self::module::Module;

use crate::utils::mem::{ObjPool, ObjPtr};
use std::sync::OnceLock;
use value_type::ValueType;
