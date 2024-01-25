use crate::backend::prog::Program;

use super::*;

pub struct ProgramMemPool {
    /// 函数内存池
    fun_pool: ObjPool<Function>,
    /// 基本块内存池
    bb_pool: ObjPool<BasicBlock>,
    /// 指令内存池
    inst_pool: ObjPool<Box<dyn Instruction>>,
}

//
impl ProgramMemPool {
    pub fn new() -> Self {
        Self {
            fun_pool: ObjPool::new(),
            bb_pool: ObjPool::new(),
            inst_pool: ObjPool::new(),
        }
    }
    pub fn alloc_function(&mut self, func: Function) -> FunPtr {
        self.fun_pool.alloc(func)
    }
    pub fn alloc_basic_block(&mut self, bb: BasicBlock) -> BBPtr {
        let bb = self.bb_pool.alloc(bb);
        BasicBlock::init_bb(bb);
        bb
    }
    pub fn alloc_instruction(&mut self, inst: Box<dyn Instruction>) -> InstPtr {
        self.inst_pool.alloc(inst)
    }
}

impl ProgramMemPool {
    pub fn clear(&mut self) {
        self.fun_pool.clear();
        self.bb_pool.clear();
        self.inst_pool.clear();
    }
}
