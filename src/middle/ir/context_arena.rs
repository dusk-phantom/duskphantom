use super::*;
pub struct ContextArena {
    functions_arena: Arena<Function>,
    basic_blocks_arena: Arena<BasicBlock>,
    instruction: Arena<Instruction>,
}

impl ContextArena {
    /// ContextArena的构造函数
    pub fn new() -> Self {
        Self {
            functions_arena: Arena::new(),
            basic_blocks_arena: Arena::new(),
            instruction: Arena::new(),
        }
    }

    /// 构造一个新的函数
    pub fn new_function(&mut self, name: &String) -> Index {
        self.functions_arena
            .insert(Function::new(name.clone(), self))
    }

    /// 构造一个新的基本块
    pub fn new_basic_block(&mut self, name: String) -> Index {
        self.basic_blocks_arena.insert(BasicBlock::new(name, self))
    }

    /// 构造一条新的指令
    pub fn new_instruction(&mut self, instruction: Instruction) -> Index {
        self.instruction.insert(instruction);
        todo!()
    }
}
