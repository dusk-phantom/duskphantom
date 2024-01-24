use crate::{
    errors::MiddelError,
    frontend,
    utils::mem::{ObjPool, ObjPtr},
};
use ir::program_mem_pool::ProgramMemPool;

mod analysis;
pub mod ir;
mod transform;

pub struct Program {
    /// 整个代码的中间表示集中在一个module中,后面可能扩展到多文件程序时候多个modules
    pub module: ir::Module,
    mem_pool: ObjPool<ProgramMemPool>, // 用与program同drop的内存池来分配program_mem_pool需要的内存,以获得固定实际内存地址的program_mem_pool
    pub program_mem_pool: ObjPtr<ProgramMemPool>,
}

pub fn gen(program: &mut frontend::Program) -> Result<Program, MiddelError> {
    todo!()
}

pub fn optimize(program: &mut Program) {
    todo!()
}

impl Program {
    pub fn new() -> Self {
        let mut mem_pool = ObjPool::new();
        let program_mem_pool = mem_pool.alloc(ProgramMemPool::new());
        Self {
            mem_pool,
            program_mem_pool,
            module: ir::Module::new(program_mem_pool),
        }
    }
}

// 为program实现drop方法
impl Drop for Program {
    fn drop(&mut self) {
        self.program_mem_pool.as_mut().clear();
        self.mem_pool.clear();
    }
}
