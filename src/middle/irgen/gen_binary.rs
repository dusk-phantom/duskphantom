use crate::context;
use crate::frontend::{BinaryOp, Expr};
use crate::middle::ir::instruction::misc_inst::{FCmpOp, ICmpOp};
use crate::middle::ir::{Constant, ValueType};
use crate::middle::irgen::function_kit::FunctionKit;
use crate::middle::irgen::value::Value;
use anyhow::{anyhow, Context};

impl<'a> FunctionKit<'a> {
    /// Generate a binary expression
    pub fn gen_binary(&mut self, op: &BinaryOp, lhs: &Expr, rhs: &Expr) -> anyhow::Result<Value> {
        let Some(mut exit) = self.exit else {
            return Err(anyhow!("exit block can't be appended")).with_context(|| context!());
        };

        // Apply operation
        match op {
            BinaryOp::Add => {
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;
                let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;
                let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;
                let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;
                let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;

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
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;
                let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;
                let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;
                let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;
                let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;
                let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                // Generate arguments and get type to cast
                let lhs_val = self.gen_expr(lhs)?;
                let rhs_val = self.gen_expr(rhs)?;
                let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

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
                // Allocate basic blocks
                let alt_name: String = self.unique_name("alt");
                let alt_entry = self.program.mem_pool.new_basicblock(alt_name);
                let final_name = self.unique_name("final");
                let mut final_entry = self.program.mem_pool.new_basicblock(final_name);

                // Load left operand to primary block, jump to alt or final block
                let lop = self.gen_expr(lhs)?.load(ValueType::Bool, self)?;
                let mut primary_exit = self.exit.unwrap();
                primary_exit.push_back(self.program.mem_pool.get_br(Some(lop)));
                primary_exit.set_true_bb(alt_entry);
                primary_exit.set_false_bb(final_entry);

                // Load right operand to alt block, jump to final block
                self.exit = Some(alt_entry);
                let rop = self.gen_expr(rhs)?.load(ValueType::Bool, self)?;
                let mut alt_exit: crate::utils::mem::ObjPtr<crate::middle::ir::BasicBlock> =
                    self.exit.unwrap();
                alt_exit.push_back(self.program.mem_pool.get_br(None));
                alt_exit.set_true_bb(final_entry);

                // Get `&&` result with "phi" instruction in final block
                self.exit = Some(final_entry);
                let inst = self.program.mem_pool.get_phi(
                    ValueType::Bool,
                    vec![
                        (Constant::Bool(false).into(), primary_exit),
                        (rop, alt_exit),
                    ],
                );
                final_entry.push_back(inst);
                Ok(Value::ReadOnly(inst.into()))
            }
            BinaryOp::Or => {
                // Allocate basic blocks
                let alt_name: String = self.unique_name("alt");
                let alt_entry = self.program.mem_pool.new_basicblock(alt_name);
                let final_name = self.unique_name("final");
                let mut final_entry = self.program.mem_pool.new_basicblock(final_name);

                // Load left operand to primary block, jump to final or alt block
                let lop = self.gen_expr(lhs)?.load(ValueType::Bool, self)?;
                let mut primary_exit = self.exit.unwrap();
                primary_exit.push_back(self.program.mem_pool.get_br(Some(lop)));
                primary_exit.set_true_bb(final_entry);
                primary_exit.set_false_bb(alt_entry);

                // Load right operand to alt block, jump to final block
                self.exit = Some(alt_entry);
                let rop = self.gen_expr(rhs)?.load(ValueType::Bool, self)?;
                let mut alt_exit = self.exit.unwrap();
                alt_exit.push_back(self.program.mem_pool.get_br(None));
                alt_exit.set_true_bb(final_entry);

                // Get `||` result with "phi" instruction in final block
                self.exit = Some(final_entry);
                let inst = self.program.mem_pool.get_phi(
                    ValueType::Bool,
                    vec![(Constant::Bool(true).into(), primary_exit), (rop, alt_exit)],
                );
                final_entry.push_back(inst);
                Ok(Value::ReadOnly(inst.into()))
            }
        }
    }
}
