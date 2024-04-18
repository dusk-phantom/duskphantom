use std::collections::HashMap;
use crate::errors::MiddelError;
use crate::{frontend, middle};
use crate::frontend::{Decl, Type};
use crate::middle::ir::FunPtr;
use crate::middle::irgen::{constant, value_type};
use crate::middle::irgen::function_kit::FunctionKit;
use crate::middle::irgen::value::Value;

/// Kit for translating a program to middle IR
pub struct ProgramKit<'a> {
    pub env: HashMap<String, Value>,
    pub fun_env: HashMap<String, FunPtr>,
    pub program: &'a mut middle::Program,
}


/// A program kit (top level) can generate declarations
impl<'a> ProgramKit<'a> {
    pub fn gen(mut self, program: &frontend::Program) -> Result<(), MiddelError> {
        for decl in program.module.iter() {
            self.gen_decl(decl)?;
        }
        for decl in program.module.iter() {
            self.gen_impl(decl)?;
        }
        Ok(())
    }

    /// Generate a declaration into the program
    /// Fails when declaration does not have a name
    pub fn gen_decl(&mut self, decl: &Decl) -> Result<(), MiddelError> {
        match decl {
            Decl::Var(ty, id, val) | Decl::Const(ty, id, val) => {
                // Get if value is global variable or constant
                let is_global_variable: bool = match decl {
                    Decl::Var(_, _, _) => true,
                    Decl::Const(_, _, _) => false,
                    _ => false,
                };

                // Get initializer
                let initializer = match val {
                    Some(v) => constant::expr_to_const(v)?,
                    None => constant::type_to_const(ty)?,
                };

                // Get global variable
                let global_val = self.program.mem_pool.new_global_variable(
                    id.clone(),
                    value_type::translate_type(ty),
                    is_global_variable,
                    initializer,
                );

                // Add global variable (pointer) to environment
                self.env.insert(id.clone(), Value::Pointer(global_val.into()));

                // Add global variable to program
                self.program.module.global_variables.push(global_val);
                Ok(())
            }
            Decl::Func(Type::Function(return_ty, _), id, _) => {
                    // Get function type
                    let fty = value_type::translate_type(return_ty);

                    // Create function
                    let fun_ptr = self.program.mem_pool.new_function(id.clone(), fty.clone());

                    // Add function to environment
                    self.fun_env.insert(id.clone(), fun_ptr);

                    // Add function to program
                    self.program.module.functions.push(fun_ptr);
                    Ok(())
            }
            _ => Err(MiddelError::GenError),
        }
    }

    /// Generate an implementation into the program
    pub fn gen_impl(&mut self, decl: &Decl) -> Result<(), MiddelError> {
        match decl {
            Decl::Func(Type::Function(_, params), id, Some(stmt)) => {
                // Clone function env before mutating it
                let cloned_fun_env = self.fun_env.clone();

                // Get function and its type
                let fun_ptr = self.fun_env.get_mut(id).ok_or(MiddelError::GenError)?;
                let fty = fun_ptr.return_type.clone();

                // Create basic block
                let entry_name = "entry".to_string();
                let mut entry = self.program.mem_pool.new_basicblock(entry_name);

                // Fill parameters
                for param in params.iter() {
                    let param = self.program.mem_pool.new_parameter(
                        param.id.clone().ok_or(MiddelError::GenError)?,
                        value_type::translate_type(&param.ty),
                    );

                    // Add parameter to function
                    fun_ptr.params.push(param);

                    // Add parameter to entry
                    let alloc = self.program.mem_pool.get_alloca(param.value_type.clone(), 1);
                    let store = self.program.mem_pool.get_store(param.into(), alloc.into());
                    entry.push_back(alloc);
                    entry.push_back(store);

                    // Add parameter to env
                    self.env.insert(param.name.clone(), Value::Pointer(alloc.into()));
                }

                // Build function
                let mut counter: usize = 0;
                let mut kit = FunctionKit {
                    program: self.program,
                    env: self.env.clone(),
                    fun_env: cloned_fun_env,
                    entry,
                    exit: entry,
                    break_to: None,
                    continue_to: None,
                    return_type: fty,
                    counter: &mut counter,
                };
                kit.gen_stmt(stmt)?;
                fun_ptr.entry = Some(kit.entry);
                fun_ptr.exit = Some(kit.exit);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}