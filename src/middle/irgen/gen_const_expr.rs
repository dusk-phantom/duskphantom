use crate::context;
use crate::frontend::Expr;
use crate::middle::ir::{Constant, Operand};
use crate::middle::irgen::program_kit::ProgramKit;
use crate::middle::irgen::value::Value;
use anyhow::{anyhow, Context};

impl<'a> ProgramKit<'a> {
    /// Generate constant expression
    pub fn gen_const_expr(&mut self, expr: &Expr) -> anyhow::Result<Constant> {
        match expr {
            Expr::Var(x) => {
                // Ensure variable is defined
                let Some(val) = self.env.get(x) else {
                    return Err(anyhow!("variable not defined")).with_context(|| context!());
                };

                // Make sure returned value is a constant
                // If operand is a global variable, convert it to constant
                // because the global variable's value is not mutated yet
                match val.clone() {
                    Value::ReadWrite(Operand::Global(gvar)) => Ok(gvar.initializer.clone()),
                    Value::ReadOnly(Operand::Constant(val)) => Ok(val),
                    _ => Err(anyhow!("variable is not a constant")).with_context(|| context!()),
                }
            }
            Expr::Pack(ls) => Ok(Constant::Array(
                ls.iter()
                    .map(|x| self.gen_const_expr(x))
                    .collect::<anyhow::Result<_, _>>()?,
            )),
            Expr::Map(_) => Err(anyhow!("map is not implemented yet")).with_context(|| context!()),
            Expr::Index(_, _) => Err(anyhow!("index not implemented")).with_context(|| context!()),
            Expr::Field(_, _) => Err(anyhow!("field not implemented")).with_context(|| context!()),
            Expr::Select(_, _) => Err(anyhow!("select not supported")).with_context(|| context!()),
            Expr::Int32(x) => Ok(Constant::Int(*x)),
            Expr::Float32(x) => Ok(Constant::Float(*x)),
            Expr::String(str) => Ok(Constant::Array(
                str.chars()
                    .map(|x| Constant::Int(x as i32))
                    .collect::<Vec<_>>(),
            )),
            Expr::Char(_) => Err(anyhow!("char not implemented")).with_context(|| context!()),
            Expr::Bool(_) => Err(anyhow!("bool not implemented")).with_context(|| context!()),
            Expr::Call(_, _) => Err(anyhow!("call not implemented")).with_context(|| context!()),
            Expr::Unary(op, expr) => self.gen_const_unary(op, expr),
            Expr::Binary(op, lhs, rhs) => self.gen_const_binary(op, lhs, rhs),
            Expr::Conditional(_, _, _) => {
                Err(anyhow!("conditional not implemented")).with_context(|| context!())
            }
        }
    }
}
