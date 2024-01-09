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
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// 表示函数指针
type FunPtr = Index;
/// 表示基本块指针
type BBPtr = Index;
/// 表示指令指针
type InstPtr = Index;

pub fn context_init() {
    context_arena::CONTEXT_INSTRUCTION.get_or_init(|| Mutex::new(HashMap::new()));
    context_arena::CONTEXT_BASIC_BLOCK.get_or_init(|| Mutex::new(HashMap::new()));
    context_arena::CONTEXT_INSTRUCTION.get_or_init(|| Mutex::new(HashMap::new()));
}

pub fn context_drop() {
    context_arena::CONTEXT_FUNCTION
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .clear();
    context_arena::CONTEXT_BASIC_BLOCK
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .clear();
    context_arena::CONTEXT_INSTRUCTION
        .get()
        .unwrap()
        .lock()
        .unwrap()
        .clear();
}
