use crate::{errors::MiddelError, frontend};

pub struct Program {
    pub content: String,
}

pub fn gen(program: &mut frontend::Program) -> Result<Program, MiddelError> {
    Ok(Program {
        content: program.content.clone(),
    })
}

pub fn optimize(program: &mut Program) {
    program.content = program.content.replace("1+1", "2");
}
