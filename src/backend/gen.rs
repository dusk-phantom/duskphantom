use super::*;

pub fn gen_with_generator(generator: &dyn Generator) -> Result<Program, BackendError> {
    // TODO
    Ok(Program {
        global: Vec::new(),
        funcs: Vec::new(),
        entry: None,
    })
}
// a trait for backend generator
pub trait Generator {}
