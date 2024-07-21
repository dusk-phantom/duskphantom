use crate::{errors::MiddleError, frontend, utils::mem::ObjPtr};
use ir::ir_builder::IRBuilder;
use transform::mem2reg;

mod analysis;
pub mod ir;
pub mod irgen;
pub mod transform;

use std::pin::Pin;

pub struct Program {
    pub module: ir::Module,
    pub mem_pool: Pin<Box<IRBuilder>>,
}

pub fn gen(program: &frontend::Program) -> Result<Program, MiddleError> {
    match irgen::gen(program) {
        Ok(program) => Ok(program),
        Err(_) => Err(MiddleError::GenError),
    }
}

pub fn optimize(_program: &mut Program) {
    // deadcode_elimination(&mut _program.module);
    mem2reg::optimize_program(_program).unwrap();
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
