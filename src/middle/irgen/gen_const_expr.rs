use crate::context;
use crate::frontend::Expr;
use crate::middle::ir::{Constant, Operand};
use crate::middle::irgen::program_kit::ProgramKit;
use crate::middle::irgen::value::Value;
use anyhow::{anyhow, Context, Result};

impl<'a> ProgramKit<'a> {
    /// Generate constant expression
    pub fn gen_const_expr(&self, expr: &Expr) -> Result<Constant> {
        match expr {
            Expr::Var(x) => {
                // Ensure variable is defined
                let Some(val) = self.get_env(x) else {
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
            Expr::Array(ls) => Ok(Constant::Array(
                ls.iter()
                    .map(|x| self.gen_const_expr(x))
                    .collect::<anyhow::Result<_, _>>()?,
            )),
            Expr::Map(_) => Err(anyhow!("map is not implemented yet")).with_context(|| context!()),
            Expr::Index(arr, idx) => {
                let arr_const = self.gen_const_expr(arr)?;
                let idx_const = self.gen_const_expr(idx)?;
                let Constant::Array(arr) = arr_const else {
                    return Err(anyhow!("indexing non-array")).with_context(|| context!());
                };
                let Constant::Int(idx) = idx_const else {
                    return Err(anyhow!("indexing with non-integer")).with_context(|| context!());
                };
                if idx < 0 || idx as usize >= arr.len() {
                    return Err(anyhow!("index out of bounds")).with_context(|| context!());
                }
                Ok(arr[idx as usize].clone())
            }
            Expr::Field(_, _) => Err(anyhow!("field not implemented")).with_context(|| context!()),
            Expr::Select(_, _) => Err(anyhow!("select not supported")).with_context(|| context!()),
            Expr::Int(x) => Ok(Constant::Int(*x)),
            Expr::Float(x) => Ok(Constant::Float(*x)),
            Expr::String(str) => {
                let mut vec = vec![];

                // Add trailing zero to bytes, pad bytes to multiple of 4
                let mut bytes = str.as_bytes().to_vec();
                bytes.push(0);
                while bytes.len() % 4 != 0 {
                    bytes.push(0);
                }

                // Convert to little indian
                for i in 0..(bytes.len() / 4) {
                    let mut val: u32 = bytes[i * 4 + 3] as u32;
                    val = val * 256 + bytes[i * 4 + 2] as u32;
                    val = val * 256 + bytes[i * 4 + 1] as u32;
                    val = val * 256 + bytes[i * 4] as u32;
                    vec.push(Constant::Int(val as i32));
                }
                Ok(Constant::Array(vec))
            }
            Expr::Char(_) => Err(anyhow!("char not implemented")).with_context(|| context!()),
            Expr::Bool(_) => Err(anyhow!("bool not implemented")).with_context(|| context!()),
            Expr::Call(_, _) => Err(anyhow!("call not implemented")).with_context(|| context!()),
            Expr::Unary(op, expr) => self.gen_const_unary(op, expr),
            Expr::Binary(head, tail) => self.gen_const_binary(head, tail),
            Expr::Conditional(_, _, _) => {
                Err(anyhow!("conditional not implemented")).with_context(|| context!())
            }
        }
    }
}
