use crate::errors::FrontEndError;

pub struct Program {
    pub content: String,
}

pub fn parse(src: &str) -> Result<Program, FrontEndError> {
    Ok(Program {
        content: src.to_string(),
    })
}

pub fn optimize(program: &mut Program) {}
