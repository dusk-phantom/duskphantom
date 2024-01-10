pub mod basic_block;
mod context_arena;
pub mod function;
pub mod instruction;
pub mod module;
pub mod value_type;

pub use self::basic_block::BasicBlock;
pub use self::function::Function;
pub use self::instruction::Instruction;
pub use self::module::Module;
use context_arena::CONTEXT_BASIC_BLOCK;
use context_arena::CONTEXT_FUNCTION;
use context_arena::CONTEXT_INSTRUCTION;

use generational_arena::{Arena, Index};
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use value_type::ValueType;

/// 表示函数指针
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct FunPtr(Index);

/// 表示基本块指针
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct BBPtr(Index);

/// 表示指令指针
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct InstPtr(Index);

/// 初始化context
pub fn context_init() {
    CONTEXT_INSTRUCTION.get_or_init(|| Mutex::new(HashMap::new()));
    CONTEXT_BASIC_BLOCK.get_or_init(|| Mutex::new(HashMap::new()));
    CONTEXT_INSTRUCTION.get_or_init(|| Mutex::new(HashMap::new()));
}

/// 释放context,在生成后端代码后由后端调用
pub fn context_drop() {
    CONTEXT_FUNCTION.get().unwrap().lock().unwrap().clear();
    CONTEXT_BASIC_BLOCK.get().unwrap().lock().unwrap().clear();
    CONTEXT_INSTRUCTION.get().unwrap().lock().unwrap().clear();
}
