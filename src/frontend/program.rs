use super::*;

/// The full program.
/// A excutable program is a set of modules with an entry module.
/// For now, only one module is supported, so the only module is entry.
pub struct Program {
    /// The module of the program.
    /// Currently only one module is supported.
    pub module: Module,
}

/// A module is a single file.
/// Only declaration can appear at top level.
pub type Module = Vec<Decl>;

pub fn parse(_src: &str) -> Result<Program, FrontendError> {
    Err(FrontendError::ParseError)
}

#[allow(unused)]
pub fn optimize(program: &mut Program) {}
