use crate::context;
use crate::frontend::{BinaryOp, Expr};
use crate::middle::ir::instruction::misc_inst::{FCmpOp, ICmpOp};
use crate::middle::ir::ValueType;
use crate::middle::irgen::function_kit::FunctionKit;
use crate::middle::irgen::value::Value;
use anyhow::{anyhow, Context};

impl<'a> FunctionKit<'a> {
    /// Generate a binary expression
    pub fn gen_binary(&mut self, op: &BinaryOp, lhs: &Expr, rhs: &Expr) -> anyhow::Result<Value> {
        let Some(mut exit) = self.exit else {
            return Err(anyhow!("exit block can't be appended")).with_context(|| context!());
        };

        // Generate arguments
        let lhs_val = self.gen_expr(lhs)?;
        let rhs_val = self.gen_expr(rhs)?;

        // Calculate maximum type for operator polymorphism
        let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

        // Apply operation
        match op {
            BinaryOp::Add => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add "add" instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_add(lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self.program.mem_pool.get_fadd(lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    _ => Err(anyhow!("`+` for NaN")).with_context(|| context!()),
                }
            }
            BinaryOp::Sub => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add "sub" instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_sub(lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self.program.mem_pool.get_fsub(lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    _ => Err(anyhow!("`-` for NaN")).with_context(|| context!()),
                }
            }
            BinaryOp::Mul => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add "mul" instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_mul(lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self.program.mem_pool.get_fmul(lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    _ => Err(anyhow!("`*` for NaN")).with_context(|| context!()),
                }
            }
            BinaryOp::Div => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add "div" instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_sdiv(lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self.program.mem_pool.get_fdiv(lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    _ => Err(anyhow!("`/` for NaN")).with_context(|| context!()),
                }
            }
            BinaryOp::Mod => {
                // Load operand as integers
                let lop = lhs_val.load(ValueType::Int, self)?;
                let rop = rhs_val.load(ValueType::Int, self)?;

                // Add "signed rem" instruction, operand is the result of the instruction
                let inst = self.program.mem_pool.get_srem(lop, rop);
                exit.push_back(inst);
                Ok(Value::ReadOnly(inst.into()))
            }
            // Bitwise operation on int is not required
            BinaryOp::Shr => Err(anyhow!("`>>` not supported")).with_context(|| context!()),
            BinaryOp::Shl => Err(anyhow!("`<<` not supported")).with_context(|| context!()),
            BinaryOp::BitAnd => Err(anyhow!("`&` not supported")).with_context(|| context!()),
            BinaryOp::BitOr => Err(anyhow!("`|` not supported")).with_context(|| context!()),
            BinaryOp::BitXor => Err(anyhow!("`^` not supported")).with_context(|| context!()),
            BinaryOp::Gt => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_icmp(ICmpOp::Sgt, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Ugt, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    _ => Err(anyhow!("`>` for NaN")).with_context(|| context!()),
                }
            }
            BinaryOp::Lt => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_icmp(ICmpOp::Slt, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Ult, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    _ => Err(anyhow!("`<` for NaN")).with_context(|| context!()),
                }
            }
            BinaryOp::Ge => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_icmp(ICmpOp::Sge, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Uge, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    _ => Err(anyhow!("`>=` for NaN")).with_context(|| context!()),
                }
            }
            BinaryOp::Le => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_icmp(ICmpOp::Sle, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Ule, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    _ => Err(anyhow!("`<=` for NaN")).with_context(|| context!()),
                }
            }
            BinaryOp::Eq => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_icmp(ICmpOp::Eq, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Ueq, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    _ => Err(anyhow!("`==` for NaN")).with_context(|| context!()),
                }
            }
            BinaryOp::Ne => {
                // Load operand as maximum type
                let lop = lhs_val.load(max_ty.clone(), self)?;
                let rop = rhs_val.load(max_ty.clone(), self)?;

                // Add compare instruction, operand is the result of the instruction
                match max_ty {
                    ValueType::Int => {
                        let inst = self.program.mem_pool.get_icmp(ICmpOp::Ne, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fcmp(FCmpOp::Une, max_ty, lop, rop);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    _ => Err(anyhow!("`!=` for NaN")).with_context(|| context!()),
                }
            }
            BinaryOp::And => {
                // Load operands as bool
                let lop = lhs_val.load(ValueType::Bool, self)?;
                let rop = rhs_val.load(ValueType::Bool, self)?;

                // Add "and" instruction, operand is the result of the instruction
                let inst = self.program.mem_pool.get_and(lop, rop);
                exit.push_back(inst);
                Ok(Value::ReadOnly(inst.into()))
            }
            BinaryOp::Or => {
                // Load operands as bool
                let lop = lhs_val.load(ValueType::Bool, self)?;
                let rop = rhs_val.load(ValueType::Bool, self)?;

                // Add "or" instruction, operand is the result of the instruction
                let inst = self.program.mem_pool.get_or(lop, rop);
                exit.push_back(inst);
                Ok(Value::ReadOnly(inst.into()))
            }
        }
    }
}
