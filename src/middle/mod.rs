use crate::{
    errors::MiddelError,
    frontend,
    utils::mem::{ObjPool, ObjPtr},
};
use ir::program_mem_pool::ProgramMemPool;

mod analysis;
pub mod ir;
mod transform;
use std::pin::Pin;

pub struct Program {
    /// 整个代码的中间表示集中在一个module中,后面可能扩展到多文件程序时候多个modules
    pub module: ir::Module,
    pub program_mem_pool: Pin<Box<ProgramMemPool>>,
}

pub fn gen(program: &mut frontend::Program) -> Result<Program, MiddelError> {
    todo!()
}

pub fn optimize(program: &mut Program) {
    todo!()
}

impl Program {
    pub fn new() -> Self {
        let program_mem_pool = Box::pin(ProgramMemPool::new());
        let mem_pool: ObjPtr<ProgramMemPool> = ObjPtr::new(&program_mem_pool);
        Self {
            program_mem_pool,
            module: ir::Module::new(mem_pool),
        }
    }
}

// 为program实现drop方法
impl Drop for Program {
    fn drop(&mut self) {
        self.program_mem_pool.as_mut().clear();
    }
}
