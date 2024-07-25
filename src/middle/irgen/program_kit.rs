use std::collections::HashMap;

use anyhow::Result;

use crate::middle::ir::FunPtr;
use crate::middle::irgen::value::Value;
use crate::{frontend, middle};

/// Kit for translating a program to middle IR
pub struct ProgramKit<'a> {
    pub env: &'a mut Vec<HashMap<String, Value>>,
    pub fun_env: &'a mut Vec<HashMap<String, FunPtr>>,
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

    /// Insert to environment
    pub fn insert_env(&mut self, name: String, value: Value) {
        self.env.last_mut().unwrap().insert(name, value);
    }

    /// Get from func environment
    pub fn get_fun_env(&self, name: &str) -> Option<FunPtr> {
        for frame in self.fun_env.iter().rev() {
            if let Some(val) = frame.get(name) {
                return Some(*val);
            }
        }
        None
    }

    /// Insert to func environment
    pub fn insert_fun_env(&mut self, name: String, value: FunPtr) {
        self.fun_env.last_mut().unwrap().insert(name, value);
    }
}
