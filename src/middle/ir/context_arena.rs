use std::sync::OnceLock;

use super::*;

static mut CONTEXT_FUNCTION: OnceLock<Arena<Function>> = OnceLock::new();
static mut CONTEXT_BASIC_BLOCK: OnceLock<Arena<BasicBlock>> = OnceLock::new();
static mut CONTEXT_INSTRUCTION: OnceLock<Arena<Box<dyn Instruction>>> = OnceLock::new();
