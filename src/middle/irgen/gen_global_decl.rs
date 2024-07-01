use std::collections::VecDeque;

use crate::context;
use crate::frontend::{Decl, Type};
use crate::middle::ir::Constant;
use crate::middle::irgen::program_kit::ProgramKit;
use crate::middle::irgen::value::Value;
use anyhow::{anyhow, Context};

use super::constant::{collapse_array, type_to_const};
use super::value_type::translate_type;

impl<'a> ProgramKit<'a> {
    /// Generate a global declaration into the program
    /// Fails when declaration does not have a name
    pub fn gen_global_decl(&mut self, decl: &Decl) -> anyhow::Result<()> {
        match decl {
            Decl::Var(ty, name, val) | Decl::Const(ty, name, val) => {
                // Get variable type
                let value_type = translate_type(ty);

                // Get if value is global variable or constant
                let is_global_variable: bool = match decl {
                    Decl::Var(_, _, _) => true,
                    Decl::Const(_, _, _) => false,
                    _ => false,
                };

                // Get initializer
                let mut initializer = match val {
                    Some(v) => self.gen_const_expr(v)?,
                    None => type_to_const(&value_type)?,
                };

                // Collapse initializer array
                if let Constant::Array(arr) = initializer {
                    initializer = collapse_array(&mut VecDeque::from(arr), &value_type)?;
                }

                // Cast initializer to required type
                initializer = initializer.cast(&value_type);

                // Get global variable
                let global_val = self.program.mem_pool.new_global_variable(
                    name.clone(),
                    value_type,
                    is_global_variable,
                    initializer,
                );

                // Add global variable (pointer) to environment
                self.env
                    .insert(name.clone(), Value::ReadWrite(global_val.into()));

                // Add global variable to program
                self.program.module.global_variables.push(global_val);
                Ok(())
            }
            Decl::Func(Type::Function(return_ty, params), id, _) => {
                // Get function type
                let fty = translate_type(return_ty);

                // Create function
                let mut fun_ptr = self.program.mem_pool.new_function(id.clone(), fty.clone());

                // Generate parameters
                for param in params.iter() {
                    let param = self.program.mem_pool.new_parameter(
                        param.id.clone().unwrap_or("_".to_string()),
                        translate_type(&param.ty),
                    );
                    fun_ptr.params.push(param);
                }

                // Add function to environment
                self.fun_env.insert(id.clone(), fun_ptr);

                // Add function to program
                self.program.module.functions.push(fun_ptr);
                Ok(())
            }
            _ => Err(anyhow!("invalid declaration")).with_context(|| context!()),
        }
    }
}
