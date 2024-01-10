use super::*;

pub static CONTEXT_FUNCTION: OnceLock<Mutex<Arena<Mutex<Function>>>> = OnceLock::new();
pub static CONTEXT_BASIC_BLOCK: OnceLock<Mutex<HashMap<Index, Arena<Mutex<BasicBlock>>>>> =
    OnceLock::new();
pub static CONTEXT_INSTRUCTION: OnceLock<
    Mutex<HashMap<Index, Arena<Mutex<Box<dyn Instruction>>>>>,
> = OnceLock::new();
