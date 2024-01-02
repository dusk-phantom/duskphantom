pub mod block;
pub mod func;
pub mod gen;
pub mod inst;
pub mod prog;
pub mod var;
use rayon::prelude::*;

use crate::{errors::BackendError, middle};
// use ARC and Mutex

#[allow(unused)]
pub fn gen(program: &middle::Program) -> Result<prog::Program, BackendError> {
    // TODO
    Ok(prog::Program {
        global: Vec::new(),
        funcs: Vec::new(),
        entry: None,
    })
}

#[allow(unused)]
pub fn optimize(program: &mut prog::Program) {
    //TODO
}

#[allow(unused)]
pub fn asm2bin(asm: String) -> String {
    panic!("not implemented")
}
