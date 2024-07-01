use std::collections::VecDeque;

use crate::context;
use crate::frontend::Decl;
use crate::middle::irgen::function_kit::FunctionKit;
use anyhow::{anyhow, Context};

use super::value::{alloc, collapse_array, Value};
use super::value_type::translate_type;

impl<'a> FunctionKit<'a> {
    /// Generate a declaration as a statement into the program
    pub fn gen_inner_decl(&mut self, decl: &Decl) -> anyhow::Result<()> {
        match decl {
            Decl::Var(raw_ty, id, op) => {
                // Allocate space for variable, add to environment
                let ty = translate_type(raw_ty);
                let lhs = alloc(ty.clone(), self);
                self.env.insert(id.clone(), lhs.clone());

                // Assign to the variable if it is defined
                if let Some(expr) = op {
                    // Generate expression as variable type
                    let mut rhs = self.gen_expr(expr)?;

                    // Collapse if `rhs` is array
                    if let Value::Array(arr) = rhs {
                        rhs = collapse_array(&mut VecDeque::from(arr), &ty)?;
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
