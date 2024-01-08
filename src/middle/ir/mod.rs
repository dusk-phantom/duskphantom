pub mod basic_block;
mod context_arena;
pub mod function;
pub mod instruction;
pub mod module;

pub use self::basic_block::BasicBlock;
pub use self::function::Function;
pub use self::instruction::Instruction;
pub use self::module::Module;

use self::context_arena::ContextArena;
use crate::middle::tool::ObjPtr;
use generational_arena::{Arena, Index};
use std::{collections::HashMap, pin::Pin};
