use crate::errors::MiddelError;
use crate::frontend;
use crate::frontend::{Decl, Type};
use crate::middle::irgen;
use crate::middle::irgen::{FunctionKit, ProgramKit, Value};

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
                let gval = match val {
                    Some(v) => self.program.mem_pool.new_global_variable(
                        id.clone(),
                        irgen::translate_type(ty),
                        // This global variable is mutable
                        true,
                        irgen::expr_to_const(v)?,
                    ),
                    None => self.program.mem_pool.new_global_variable(
                        id.clone(),
                        irgen::translate_type(ty),
                        true,
                        irgen::type_to_const(ty)?,
                    ),
                };

                // Add global variable (pointer) to environment
                self.env.insert(id.clone(), Value::Pointer(gval.into()));

                // Add global variable to program
                self.program.module.global_variables.push(gval);
                Ok(())
            }
            Decl::Func(ty, id, op) => {
                if let (Some(stmt), Type::Function(return_ty, params)) = (op, ty) {
                    // Get function type
                    let fty = irgen::translate_type(return_ty);

                    // Create function
                    let mut fptr = self.program.mem_pool.new_function(id.clone(), fty.clone());

                    // Fill parameters
                    for param in params.iter() {
                        let param = self.program.mem_pool.new_parameter(
                            param.id.clone().ok_or(MiddelError::GenError)?,
                            irgen::translate_type(&param.ty),
                        );
                        fptr.params.push(param);
                    }

                    // Build function
                    let fname = self.unique_debug("entry");
                    let bb = self.program.mem_pool.new_basicblock(fname);
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
                    };
                    kit.gen_stmt(stmt)?;
                    fptr.entry = Some(kit.entry);
                    fptr.exit = Some(kit.exit);

                    // Add function to environment
                    self.fun_env.insert(id.clone(), fptr);

                    // Add function to programs
                    self.program.module.functions.push(fptr);
                    Ok(())
                } else {
                    Ok(())
                }
            }
            Decl::Enum(_, _) => todo!(),
            Decl::Union(_, _) => todo!(),
            Decl::Struct(_, _) => todo!(),
        }
    }

    /// Generate a unique debug name for a basic block
    pub fn unique_debug(&self, base: &'static str) -> String {
        base.to_string()
    }
}
