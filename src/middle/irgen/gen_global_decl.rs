use crate::context;
use crate::frontend::{Decl, Type};
use crate::middle::irgen::program_kit::ProgramKit;
use crate::middle::irgen::value::Value;
use crate::middle::irgen::{constant, value_type};
use anyhow::{anyhow, Context};

impl<'a> ProgramKit<'a> {
    /// Generate a global declaration into the program
    /// Fails when declaration does not have a name
    pub fn gen_global_decl(&mut self, decl: &Decl) -> anyhow::Result<()> {
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
                    Some(v) => self.gen_const_expr(v)?,
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
                self.env
                    .insert(id.clone(), Value::Pointer(global_val.into()));

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
            _ => Err(anyhow!("invalid declaration")).with_context(|| context!()),
        }
    }
}
