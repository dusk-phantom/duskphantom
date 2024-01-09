pub mod basic_block;
mod context_arena;
pub mod function;
pub mod instruction;
pub mod module;

pub use self::basic_block::BasicBlock;
pub use self::function::Function;
pub use self::instruction::Instruction;
pub use self::module::Module;

use generational_arena::{Arena, Index};
use std::{collections::HashMap, pin::Pin};

/// 表示函数指针
type FunPtr = Index;
/// 表示基本块指针
type BBPtr = Index;
/// 表示指令指针
type InstPtr = Index;

use std::{
    ops::{Deref, DerefMut},
    ptr::NonNull,
};
