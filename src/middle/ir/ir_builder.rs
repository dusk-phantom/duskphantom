use super::*;

pub struct IRBuilder {
    fun_pool: ObjPool<Function>,
    bb_pool: ObjPool<BasicBlock>,
    inst_pool: ObjPool<Box<dyn Instruction>>,
    inst_id: usize,
}

impl IRBuilder {
    pub fn new() -> Self {
        Self {
            fun_pool: ObjPool::new(),
            bb_pool: ObjPool::new(),
            inst_pool: ObjPool::new(),
            inst_id: 0,
        }
    }

    /// Allocate a space for func, return a pointer to this space.
    pub fn new_function(&mut self, name: String, return_type: ValueType) -> FunPtr {
        let func = Function {
            mem_pool: self.into(),
            name: name.clone(),
            entry: None,
            exit: None,
            return_type,
            params: self.new_basicblock(format!("p_{}", name)),
        };
        self.fun_pool.alloc(func)
    }

    /// Allocate a space for basicblock, return a pointer to this space.
    pub fn new_basicblock(&mut self, name: String) -> BBPtr {
        let ptr = self.into();
        let bb = self.bb_pool.alloc(BasicBlock::new(name, ptr));
        unsafe {
            BasicBlock::init_bb(bb);
        }
        bb
    }

    /// Gets a new id for instruction.
    #[inline]
    pub fn new_inst_id(&mut self) -> usize {
        let old = self.inst_id;
        self.inst_id += 1;
        old
    }

    /// Allocate a space for instruction, return a pointer to this space.
    pub fn new_instruction(&mut self, inst: Box<dyn Instruction>) -> InstPtr {
        self.inst_pool.alloc(inst)
    }
}

impl IRBuilder {
    pub fn clear(&mut self) {
        self.fun_pool.clear();
        self.bb_pool.clear();
        self.inst_pool.clear();
    }
}
