use super::*;

/// The full program.
/// A executable program is a set of modules with an entry module.
/// For now, only one module is supported, so the only module is entry.
#[derive(Clone, PartialEq, Debug)]
pub struct Program {
    /// The module of the program.
    /// Currently only one module is supported.
    pub module: Module,
}

impl Program {
    pub fn new(decls: Vec<Decl>) -> Self {
        Self { module: decls }
    }
}

/// A module is a single file.
/// Only declaration can appear at top level.
pub type Module = Vec<Decl>;