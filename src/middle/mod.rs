use crate::{errors::MiddleError, frontend, utils::mem::ObjPtr};
use ir::ir_builder::IRBuilder;

mod analysis;
pub mod ir;
mod irgen;
mod transform;

use std::pin::Pin;
pub struct Program {
    pub module: ir::Module,
    pub mem_pool: Pin<Box<IRBuilder>>,
}

pub fn gen(program: &frontend::Program) -> Result<Program, MiddleError> {
    irgen::gen(program)
}

pub fn optimize(_program: &mut Program) {
    todo!()
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl Program {
    pub fn new() -> Self {
        let program_mem_pool = Box::pin(IRBuilder::new());
        let mem_pool: ObjPtr<IRBuilder> = ObjPtr::new(&program_mem_pool);
        Self {
            mem_pool: program_mem_pool,
            module: ir::Module::new(mem_pool),
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        self.mem_pool.clear();
    }
}
