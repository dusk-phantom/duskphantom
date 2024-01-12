use crate::{errors::MiddelError, frontend};

mod analysis;
pub mod ir;
mod transform;

pub struct Program {
    module: ir::Module,
}

pub fn gen(program: &mut frontend::Program) -> Result<Program, MiddelError> {
    todo!()
}

pub fn optimize(program: &mut Program) {
    todo!()
}

impl Program {
    pub fn new() -> Self {
        Self {
            module: ir::Module::new(),
        }
    }
}
