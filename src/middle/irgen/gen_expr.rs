use crate::context;
use crate::frontend::Expr;
use crate::middle::ir::{Constant, ValueType};
use crate::middle::irgen::function_kit::FunctionKit;
use crate::middle::irgen::value::Value;
use anyhow::{anyhow, Context};

impl<'a> FunctionKit<'a> {
    /// Generate an expression as a statement into the program
    pub fn gen_expr(&mut self, expr: &Expr) -> anyhow::Result<Value> {
        let Some(mut exit) = self.exit else {
            return Err(anyhow!("exit block can't be appended")).with_context(|| context!());
        };
        match expr {
            Expr::Var(x) => {
                // Ensure variable is defined
                let Some(operand) = self.env.get(x) else {
                    return Err(anyhow!("variable not defined")).with_context(|| context!());
                };

                // Clone the operand and return, this clones the underlying value or InstPtr
                Ok(operand.clone())
            }
            Expr::Pack(ls) => Ok(Value::Array(
                ls.iter()
                    .map(|x| self.gen_expr(x))
                    .collect::<anyhow::Result<_, _>>()?,
            )),
            Expr::Map(_) => Err(anyhow!("map is not supported")).with_context(|| context!()),
            Expr::Index(x, v) => {
                // Load index as integer
                let ix = self.gen_expr(v)?.load(ValueType::Int, self)?;

                // Generate GEP
                self.gen_expr(x)?
                    .get_element_ptr(self, vec![Constant::Int(0).into(), ix])
            }
            Expr::Field(_, _) => Err(anyhow!("field not supported")).with_context(|| context!()),
            Expr::Select(_, _) => Err(anyhow!("select not supported")).with_context(|| context!()),
            Expr::Int32(x) => Ok(Constant::Int(*x).into()),
            Expr::Float32(x) => Ok(Constant::Float(*x).into()),
            Expr::String(_) => Err(anyhow!("string not supported")).with_context(|| context!()),
            Expr::Char(_) => Err(anyhow!("char not supported")).with_context(|| context!()),
            Expr::Bool(_) => Err(anyhow!("bool not supported")).with_context(|| context!()),
            Expr::Call(func, args) => {
                // Generate arguments
                let mut operands = Vec::new();
                for arg in args.iter() {
                    operands.push(self.gen_expr(arg)?.load(ValueType::Int, self)?);
                }

                // Ensure function is a defined variable
                let Expr::Var(func) = *func.clone() else {
                    return Err(anyhow!("function is not variable")).with_context(|| context!());
                };
                let Some(fun) = self.fun_env.get(&func) else {
                    return Err(anyhow!("function not defined")).with_context(|| context!());
                };

                // Call the function
                let inst = self.program.mem_pool.get_call(*fun, operands);
                exit.push_back(inst);
                Ok(Value::Operand(inst.into()))
            }
            Expr::Unary(op, expr) => self.gen_unary(op, expr),
            Expr::Binary(op, lhs, rhs) => self.gen_binary(op, lhs, rhs),
            Expr::Conditional(_, _, _) => {
                Err(anyhow!("conditional not supported")).with_context(|| context!())
            }
        }
    }
}