use std::collections::VecDeque;

use crate::frontend::Decl;
use crate::middle::ir::Operand;
use crate::middle::irgen::function_kit::FunctionKit;
use crate::{context, middle::ir::Constant};
use anyhow::{anyhow, Context};

use super::gen_const::gen_const;
use super::gen_type::gen_type;
use super::value::{alloc, Value};
use super::{constant, value};

impl<'a> FunctionKit<'a> {
    /// Generate a declaration as a statement into the program
    pub fn gen_inner_decl(&mut self, decl: &Decl) -> anyhow::Result<()> {
        match decl {
            Decl::Const(raw_ty, id, op) => {
                // Make sure constant has an initializer
                let Some(expr) = op else {
                    return Err(anyhow!("const declaration must have an initializer"))
                        .with_context(|| context!());
                };

                // Translate type
                let ty = gen_type(raw_ty)?;

                // Generate constant value
                let val = gen_const(expr)?;

                // If constant is an array, collapse it and store into global variable
                let val = match val {
                    Constant::Array(arr) => {
                        let arr = constant::collapse_array(&mut VecDeque::from(arr), &ty)?;
                        let name = self.unique_name(id);
                        let gvar = self.program.mem_pool.new_global_variable(
                            name,
                            arr.get_type(),
                            false,
                            arr,
                        );
                        self.program.module.global_variables.push(gvar);
                        Value::ReadWrite(gvar.into())
                    }
                    _ => Value::ReadOnly(val.into()),
                };

                // Add value to environment
                self.insert_env(id.clone(), val);
                Ok(())
            }
            Decl::Var(raw_ty, id, op) => {
                // Allocate space for variable, add to environment
                let ty = gen_type(raw_ty)?;
                let lhs = alloc(ty.clone(), self);
                self.insert_env(id.clone(), lhs.clone());

                // Assign to the variable if it is defined
                if let Some(expr) = op {
                    // Generate expression as variable type
                    let mut rhs = self.gen_expr(expr)?;

                    // Collapse and memset 0 if `rhs` is array
                    if let Value::Array(arr) = rhs {
                        let ptr = lhs.clone().load_uncast(self)?.0;
                        let memset_func = self.get_fun_env("llvm.memset.p0.i32").unwrap();
                        let memset_call = self.program.mem_pool.get_call(
                            memset_func,
                            vec![
                                ptr,
                                Operand::Constant(Constant::SignedChar(0)),
                                Operand::Constant(Constant::Int(ty.size() as i32 * 4)),
                                Operand::Constant(Constant::Bool(false)),
                            ],
                        );
                        self.exit.unwrap().push_back(memset_call);
                        rhs = value::collapse_array(&mut VecDeque::from(arr), &ty)?;
                    }

                    // Assign operand to value
                    lhs.assign(self, rhs)?;
                };
                Ok(())
            }
            Decl::Stack(decls) => {
                // Generate each declaration
                for decl in decls.iter() {
                    self.gen_inner_decl(decl)?;
                }
                Ok(())
            }
            _ => Err(anyhow!("unrecognized declaration")).with_context(|| context!()),
        }
    }
}
