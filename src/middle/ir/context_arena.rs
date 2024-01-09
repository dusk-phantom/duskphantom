use super::*;
pub struct ContextArena {
    functions_arena: Arena<Function>,
    basic_blocks_arena: Arena<BasicBlock>,
    instructions_arena: Arena<Box<dyn Instruction>>,
}

impl ContextArena {
    /// ContextArena的构造函数
    pub fn new() -> Self {
        Self {
            functions_arena: Arena::new(),
            basic_blocks_arena: Arena::new(),
            instructions_arena: Arena::new(),
        }
    }

    /// 构造一个新的函数
    pub fn new_function(&mut self, name: &String) -> FunPtr {
        self.functions_arena
            .insert(Function::new(name.clone(), self))
    }

    /// 构造一个新的基本块
    pub fn new_basic_block(&mut self, name: String) -> BBPtr {
        self.basic_blocks_arena.insert(BasicBlock::new(name, self))
    }

    /// 构造一条新的指令
    pub fn new_instruction(&mut self) -> InstPtr {
        todo!()
    }
}
