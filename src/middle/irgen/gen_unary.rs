use crate::context;
use crate::frontend::{Expr, UnaryOp};
use crate::middle::ir::{Constant, ValueType};
use crate::middle::irgen::function_kit::FunctionKit;
use crate::middle::irgen::value::Value;
use anyhow::{anyhow, Context};

impl<'a> FunctionKit<'a> {
    /// Generate a unary expression
    pub fn gen_unary(&mut self, op: &UnaryOp, expr: &Expr) -> anyhow::Result<Value> {
        let Some(mut exit) = self.exit else {
            return Err(anyhow!("exit block can't be appended")).with_context(|| context!());
        };

        // Generate argument
        let val = self.gen_expr(expr)?;

        // Calculate type for operator polymorphism
        let ty = val.get_type();

        // Apply operation
        match op {
            UnaryOp::Neg => {
                // Return 0 - x
                let operand = val.load(ty.clone(), self)?;
                match ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_sub(Constant::Int(0).into(), operand);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fsub(Constant::Float(0.0).into(), operand);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Bool => {
                        // Convert to int and then make negative
                        let zext = self.program.mem_pool.get_zext(operand);
                        let sub = self
                            .program
                            .mem_pool
                            .get_sub(Constant::Int(0).into(), zext.into());
                        exit.push_back(zext);
                        exit.push_back(sub);
                        Ok(Value::ReadOnly(sub.into()))
                    }
                    _ => Err(anyhow!("`-` for NaN")).with_context(|| context!()),
                }
            }
            UnaryOp::Pos => {
                // Return operand directly
                let operand = val.load(ty.clone(), self)?;
                match ty {
                    ValueType::Int | ValueType::Float | ValueType::Bool => {
                        Ok(Value::ReadOnly(operand))
                    }
                    _ => Err(anyhow!("`+` for NaN")).with_context(|| context!()),
                }
            }
            UnaryOp::Not => {
                // Load as boolean
                let bool_op = val.load(ValueType::Bool, self)?;

                // Add "xor" instruction
                let inst = self
                    .program
                    .mem_pool
                    .get_xor(bool_op, Constant::Bool(true).into());
                exit.push_back(inst);
                Ok(Value::ReadOnly(inst.into()))
            }
            _ => Err(anyhow!("unary operator not supported")).with_context(|| context!()),
        }
    }
}
