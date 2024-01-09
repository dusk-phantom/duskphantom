use super::*;

pub static CONTEXT_FUNCTION: OnceLock<Mutex<HashMap<FunPtr, Arena<Function>>>> = OnceLock::new();
pub static CONTEXT_BASIC_BLOCK: OnceLock<Mutex<HashMap<FunPtr, Arena<BasicBlock>>>> =
    OnceLock::new();
pub static CONTEXT_INSTRUCTION: OnceLock<Mutex<HashMap<FunPtr, Arena<Box<dyn Instruction>>>>> =
    OnceLock::new();
