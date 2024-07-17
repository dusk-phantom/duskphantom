use anyhow::{anyhow, Context, Result};

use crate::frontend::Type;
use crate::middle::ir::{Constant, FunPtr, ValueType};
use crate::middle::irgen::value::Value;
use crate::{context, frontend, middle};
use std::collections::HashMap;

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

    /// Get from environment
    pub fn get_env(&self, name: &str) -> Option<Value> {
        for frame in self.env.iter().rev() {
            if let Some(val) = frame.get(name) {
                return Some(val.clone());
            }
        }
        None
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

    /// Translate a frontend type to IR value type
    pub fn translate_type(&self, ty: &Type) -> Result<ValueType> {
        match ty {
            Type::Void => Ok(ValueType::Void),
            Type::Int32 => Ok(ValueType::Int),
            Type::Float32 => Ok(ValueType::Float),
            Type::String => Err(anyhow!("string not supported")).with_context(|| context!()),
            Type::Char => Err(anyhow!("char not supported")).with_context(|| context!()),
            Type::Boolean => Ok(ValueType::Bool),
            Type::Pointer(ty) => Ok(ValueType::Pointer(Box::new(self.translate_type(ty)?))),
            Type::Array(ty, index_expr) => {
                let index_constant = self.gen_const_expr(index_expr)?;
                let Constant::Int(index) = index_constant else {
                    // TODO need to support usize?
                    return Err(anyhow!("index is not an integer")).with_context(|| context!());
                };
                Ok(ValueType::Array(
                    Box::new(self.translate_type(ty)?),
                    index as usize,
                ))
            }
            Type::Function(_, _) => {
                Err(anyhow!("function not supported")).with_context(|| context!())
            }
            Type::Enum(_) => Err(anyhow!("enum not supported")).with_context(|| context!()),
            Type::Union(_) => Err(anyhow!("union not supported")).with_context(|| context!()),
            Type::Struct(_) => Err(anyhow!("struct not supported")).with_context(|| context!()),
        }
    }
}
