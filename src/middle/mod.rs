use crate::{/* errors::MiddleError, */ frontend, utils::mem::ObjPtr};
use anyhow::Context;
use ir::ir_builder::IRBuilder;
use transform::{loop_optimization, ultimate_pass};

pub mod analysis;
pub mod ir;
pub mod irgen;
pub mod transform;

use std::pin::Pin;

pub struct Program {
    pub module: ir::Module,
    pub mem_pool: Pin<Box<IRBuilder>>,
}

use crate::context;
use anyhow::Result;

pub fn gen(program: &frontend::Program) -> Result<Program> {
    irgen::gen(program).with_context(|| context!())
    // match irgen::gen(program) {
    //     Ok(program) => Ok(program),
    //     Err(_) => Err(MiddleError::GenError),
    // }
}

pub fn optimize(program: &mut Program) {
    ultimate_pass::optimize_program(program).unwrap();
    // Loop optimization
    loop_optimization::optimize_program(program).unwrap();
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
