use anyhow::Result;

use crate::middle::ir::FunPtr;
use crate::middle::irgen::value::Value;
use crate::utils::frame_map::FrameMap;
use crate::{frontend, middle};

/// Kit for translating a program to middle IR
pub struct ProgramKit<'a> {
    pub env: FrameMap<'a, String, Value>,
    pub fun_env: FrameMap<'a, String, FunPtr>,
    pub program: &'a mut middle::Program,
}

/// A program kit (top level) can generate declarations
impl<'a> ProgramKit<'a> {
    pub fn gen(mut self, program: &frontend::Program) -> Result<()> {
        self.gen_library_function();
        for decl in program.module.iter() {
            self.gen_global_decl(decl)?;
        }
        for decl in program.module.iter() {
            self.gen_impl(decl)?;
        }
        Ok(())
    }
}
