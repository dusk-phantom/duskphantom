use crate::{errors::MiddleError, frontend, utils::mem::ObjPtr};
use ir::ir_builder::IRBuilder;
use transform::{mem2reg, simple_gvn};

mod analysis;
pub mod ir;
pub mod irgen;
pub mod transform;

use std::pin::Pin;

use self::transform::{constant_fold, deadcode_elimination};

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

pub fn optimize(program: &mut Program) {
    mem2reg::optimize_program(program).unwrap();
    deadcode_elimination::optimize_program(program).unwrap();
    constant_fold::optimize_program(program).unwrap();
    deadcode_elimination::optimize_program(program).unwrap();
    simple_gvn::optimize_program(program).unwrap();
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
