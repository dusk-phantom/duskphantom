pub mod block;
pub mod func;
pub mod gen;
pub mod inst;
pub mod var;
use crate::{errors::BackendError, middle};
// use ARC and Mutex

pub struct Program {
    // global var ,including primtype var and arr var
    pub global: Vec<var::Var>,
    // all funcs
    pub funcs: Vec<func::Func>,
    // optional entry func
    pub entry: Option<String>,
}
impl Program {
    pub fn has_entry(&self) -> bool {
        self.entry.is_some()
    }
    pub fn gen_asm(&self) -> String {
        // TODO
        String::new()
    }
}

#[allow(unused)]
pub fn gen(program: &middle::Program) -> Result<Program, BackendError> {
    // TODO
    Ok(Program {
        global: Vec::new(),
        funcs: Vec::new(),
        entry: None,
    })
}

#[allow(unused)]
pub fn optimize(program: &mut Program) {
    //TODO
}

#[allow(unused)]
pub fn asm2bin(asm: String) -> String {
    panic!("not implemented")
}
