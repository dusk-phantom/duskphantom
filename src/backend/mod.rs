pub mod block;
pub mod func;
pub mod gen;
pub mod inst;
pub mod var;
use crate::{errors::BackendError, middle};

pub struct Program {
    content: String,
}

impl Program {
    pub fn gen_asm(&self) -> String {
        self.content.clone()
    }
}

pub fn gen(program: &middle::Program) -> Result<Program, BackendError> {
    Ok(Program {
        content: program.content.clone(),
    })
}

pub fn optimize(program: &mut Program) {
    program.content = program.content.replace("1+1", "2");
}

#[allow(unused)]
pub fn asm2bin(asm: String) -> String {
    panic!("not implemented")
}
