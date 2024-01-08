use crate::errors::FrontendError;

pub struct Program {
    pub content: String,
}

pub fn parse(src: &str) -> Result<Program, FrontendError> {
    Ok(Program {
        content: src.to_string(),
    })
}
#[allow(unused)]
pub fn optimize(program: &mut Program) {}
