use crate::context;
use crate::frontend::Expr;
use crate::middle::ir::{Constant, ValueType};
use crate::middle::irgen::function_kit::FunctionKit;
use crate::middle::irgen::value::Value;
use anyhow::{anyhow, Context};

use super::gen_library_function::is_argument_const;

impl<'a> FunctionKit<'a> {
    /// Generate an expression as a statement into the program
    pub fn gen_expr(&mut self, expr: &Expr) -> anyhow::Result<Value> {
        let Some(mut exit) = self.exit else {
            return Err(anyhow!("exit block can't be appended")).with_context(|| context!());
        };
        match expr {
            Expr::Var(x) => {
                // Ensure variable is defined
                let Some(operand) = self.get_env(x) else {
                    return Err(anyhow!("variable not defined")).with_context(|| context!());
                };

                // Clone the operand and return, this clones the underlying value or InstPtr
                Ok(operand.clone())
            }
            Expr::Array(ls) => Ok(Value::Array(
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
                    .getelementptr(self, vec![Constant::Int(0).into(), ix])
            }
            Expr::Field(_, _) => Err(anyhow!("field not supported")).with_context(|| context!()),
            Expr::Select(_, _) => Err(anyhow!("select not supported")).with_context(|| context!()),
            Expr::Int(x) => Ok(Constant::Int(*x).into()),
            Expr::Float(x) => Ok(Constant::Float(*x).into()),
            Expr::String(_) => Err(anyhow!("string not supported")).with_context(|| context!()),
            Expr::Char(_) => Err(anyhow!("char not supported")).with_context(|| context!()),
            Expr::Bool(_) => Err(anyhow!("bool not supported")).with_context(|| context!()),
            Expr::Call(func, args) => {
                // Ensure function is a defined variable
                let Expr::Var(func_name) = *func.clone() else {
                    return Err(anyhow!("function is not variable")).with_context(|| context!());
                };
                let Some(func_ptr) = self.get_fun_env(&func_name) else {
                    return Err(anyhow!("function not defined")).with_context(|| context!());
                };

                // Generate arguments
                let mut operands = Vec::new();
                if func_ptr.params.len() == args.len() {
                    for (param, arg) in func_ptr.params.iter().zip(args.iter()) {
                        let arg = self.gen_expr(arg)?.load(param.value_type.clone(), self)?;
                        operands.push(arg);
                    }
                } else {
                    for (i, arg) in args.iter().enumerate() {
                        // Support constant argument only for dynamic library functions like `putf`
                        let arg = if is_argument_const(&func_name, i) {
                            let constant = self.gen_program_kit().gen_const_expr(arg)?;
                            let name = self.unique_name("format");
                            let gvar = self.program.mem_pool.new_global_variable(
                                name,
                                constant.get_type(),
                                false,
                                constant,
                            );
                            self.program.module.global_variables.push(gvar);
                            Value::ReadWrite(gvar.into()).load_uncast(self)?.0
                        } else {
                            self.gen_expr(arg)?.load_uncast(self)?.0
                        };
                        operands.push(arg);
                    }
                }

                // Call the function
                let inst = self.program.mem_pool.get_call(func_ptr, operands);
                exit.push_back(inst);
                Ok(Value::ReadOnly(inst.into()))
            }
            Expr::Unary(op, expr) => self.gen_unary(op, expr),
            Expr::Binary(head, tail) => self.gen_binary(head, tail),
            Expr::Conditional(_, _, _) => {
                Err(anyhow!("conditional not supported")).with_context(|| context!())
            }
        }
    }
}
