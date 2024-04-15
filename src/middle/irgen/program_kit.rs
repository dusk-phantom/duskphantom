use std::collections::HashMap;
use crate::errors::MiddelError;
use crate::{frontend, middle};
use crate::frontend::{Decl, Type};
use crate::middle::ir::{FunPtr, ValueType};
use crate::middle::irgen::{constant, value_type};
use crate::middle::irgen::function_kit::FunctionKit;
use crate::middle::irgen::value::Value;

/// Kit for translating a program to middle IR
pub struct ProgramKit<'a> {
    pub env: HashMap<String, Value>,
    pub fun_env: HashMap<String, FunPtr>,
    pub ctx: HashMap<String, ValueType>,
    pub program: &'a mut middle::Program,
}


/// A program kit (top level) can generate declarations
impl<'a> ProgramKit<'a> {
    pub fn gen(mut self, program: &frontend::Program) -> Result<(), MiddelError> {
        for decl in program.module.iter() {
            self.gen_decl(decl)?;
        }
        Ok(())
    }

    /// Generate a declaration into the program
    /// Fails when declaration does not have a name
    pub fn gen_decl(&mut self, decl: &Decl) -> Result<(), MiddelError> {
        match decl {
            Decl::Var(ty, id, val) => {
                // Get global variable
                let global_val = match val {
                    Some(v) => self.program.mem_pool.new_global_variable(
                        id.clone(),
                        value_type::translate_type(ty),
                        // This global variable is mutable
                        true,
                        constant::expr_to_const(v)?,
                    ),
                    None => self.program.mem_pool.new_global_variable(
                        id.clone(),
                        value_type::translate_type(ty),
                        true,
                        constant::type_to_const(ty)?,
                    ),
                };

                // Add global variable (pointer) to environment
                self.env.insert(id.clone(), Value::Pointer(global_val.into()));

                // Add global variable to program
                self.program.module.global_variables.push(global_val);
                Ok(())
            }
            Decl::Func(ty, id, op) => {
                if let (Some(stmt), Type::Function(return_ty, params)) = (op, ty) {
                    // Get function type
                    let fty = value_type::translate_type(return_ty);

                    // Create function
                    let mut fun_ptr = self.program.mem_pool.new_function(id.clone(), fty.clone());

                    // Fill parameters
                    for param in params.iter() {
                        let param = self.program.mem_pool.new_parameter(
                            param.id.clone().ok_or(MiddelError::GenError)?,
                            value_type::translate_type(&param.ty),
                        );
                        fun_ptr.params.push(param);
                    }

                    // Build function
                    let fun_name = "entry".to_string();
                    let bb = self.program.mem_pool.new_basicblock(fun_name);
                    let mut counter: usize = 0;
                    let mut kit = FunctionKit {
                        program: self.program,
                        env: self.env.clone(),
                        fun_env: self.fun_env.clone(),
                        ctx: self.ctx.clone(),
                        entry: bb,
                        exit: bb,
                        break_to: None,
                        continue_to: None,
                        return_type: fty,
                        counter: &mut counter,
                    };
                    kit.gen_stmt(stmt)?;
                    fun_ptr.entry = Some(kit.entry);
                    fun_ptr.exit = Some(kit.exit);

                    // Add function to environment
                    self.fun_env.insert(id.clone(), fun_ptr);

                    // Add function to programs
                    self.program.module.functions.push(fun_ptr);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            Decl::Enum(_, _) => Err(MiddelError::GenError),
            Decl::Union(_, _) => Err(MiddelError::GenError),
            Decl::Struct(_, _) => Err(MiddelError::GenError),
        }
    }
}