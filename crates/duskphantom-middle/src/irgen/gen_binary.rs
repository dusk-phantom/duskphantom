// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use crate::frontend::{BinaryOp, Expr};
use crate::ir::instruction::misc_inst::{FCmpOp, ICmpOp};
use crate::ir::{Constant, ValueType};
use crate::irgen::function_kit::FunctionKit;
use crate::irgen::value::Value;
use anyhow::{anyhow, Context};
use duskphantom_utils::context;

impl<'a> FunctionKit<'a> {
    /// Generate a binary expression
    pub fn gen_binary(&mut self, head: &Expr, tail: &[(BinaryOp, Expr)]) -> anyhow::Result<Value> {
        let Some(mut exit) = self.exit else {
            return Err(anyhow!("exit block can't be appended")).with_context(|| context!());
        };

        // Apply operation by iteration
        let mut lhs_val = self.gen_expr(head)?;
        for (op, rhs) in tail {
            lhs_val = match op {
                BinaryOp::Add => {
                    // Generate arguments and get type to cast
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
                    let rhs_val = self.gen_expr(rhs)?;
                    let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

                    // Load operand as maximum type
                    let lop = lhs_val.load(max_ty.clone(), self)?;
                    let rop = rhs_val.load(max_ty.clone(), self)?;

                    // Add compare instruction, operand is the result of the instruction
                    match max_ty {
                        ValueType::Int => {
                            let inst =
                                self.program
                                    .mem_pool
                                    .get_icmp(ICmpOp::Sgt, max_ty, lop, rop);
                            exit.push_back(inst);
                            Ok(Value::ReadOnly(inst.into()))
                        }
                        ValueType::Float => {
                            let inst =
                                self.program
                                    .mem_pool
                                    .get_fcmp(FCmpOp::Ogt, max_ty, lop, rop);
                            exit.push_back(inst);
                            Ok(Value::ReadOnly(inst.into()))
                        }
                        _ => Err(anyhow!("`>` for NaN")).with_context(|| context!()),
                    }
                }
                BinaryOp::Lt => {
                    // Generate arguments and get type to cast
                    let rhs_val = self.gen_expr(rhs)?;
                    let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

                    // Load operand as maximum type
                    let lop = lhs_val.load(max_ty.clone(), self)?;
                    let rop = rhs_val.load(max_ty.clone(), self)?;

                    // Add compare instruction, operand is the result of the instruction
                    match max_ty {
                        ValueType::Int => {
                            let inst =
                                self.program
                                    .mem_pool
                                    .get_icmp(ICmpOp::Slt, max_ty, lop, rop);
                            exit.push_back(inst);
                            Ok(Value::ReadOnly(inst.into()))
                        }
                        ValueType::Float => {
                            let inst =
                                self.program
                                    .mem_pool
                                    .get_fcmp(FCmpOp::Olt, max_ty, lop, rop);
                            exit.push_back(inst);
                            Ok(Value::ReadOnly(inst.into()))
                        }
                        _ => Err(anyhow!("`<` for NaN")).with_context(|| context!()),
                    }
                }
                BinaryOp::Ge => {
                    // Generate arguments and get type to cast
                    let rhs_val = self.gen_expr(rhs)?;
                    let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

                    // Load operand as maximum type
                    let lop = lhs_val.load(max_ty.clone(), self)?;
                    let rop = rhs_val.load(max_ty.clone(), self)?;

                    // Add compare instruction, operand is the result of the instruction
                    match max_ty {
                        ValueType::Int => {
                            let inst =
                                self.program
                                    .mem_pool
                                    .get_icmp(ICmpOp::Sge, max_ty, lop, rop);
                            exit.push_back(inst);
                            Ok(Value::ReadOnly(inst.into()))
                        }
                        ValueType::Float => {
                            let inst =
                                self.program
                                    .mem_pool
                                    .get_fcmp(FCmpOp::Oge, max_ty, lop, rop);
                            exit.push_back(inst);
                            Ok(Value::ReadOnly(inst.into()))
                        }
                        _ => Err(anyhow!("`>=` for NaN")).with_context(|| context!()),
                    }
                }
                BinaryOp::Le => {
                    // Generate arguments and get type to cast
                    let rhs_val = self.gen_expr(rhs)?;
                    let max_ty = lhs_val.get_type().max_with(&rhs_val.get_type());

                    // Load operand as maximum type
                    let lop = lhs_val.load(max_ty.clone(), self)?;
                    let rop = rhs_val.load(max_ty.clone(), self)?;

                    // Add compare instruction, operand is the result of the instruction
                    match max_ty {
                        ValueType::Int => {
                            let inst =
                                self.program
                                    .mem_pool
                                    .get_icmp(ICmpOp::Sle, max_ty, lop, rop);
                            exit.push_back(inst);
                            Ok(Value::ReadOnly(inst.into()))
                        }
                        ValueType::Float => {
                            let inst =
                                self.program
                                    .mem_pool
                                    .get_fcmp(FCmpOp::Ole, max_ty, lop, rop);
                            exit.push_back(inst);
                            Ok(Value::ReadOnly(inst.into()))
                        }
                        _ => Err(anyhow!("`<=` for NaN")).with_context(|| context!()),
                    }
                }
                BinaryOp::Eq => {
                    // Generate arguments and get type to cast
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
                            let inst =
                                self.program
                                    .mem_pool
                                    .get_fcmp(FCmpOp::Oeq, max_ty, lop, rop);
                            exit.push_back(inst);
                            Ok(Value::ReadOnly(inst.into()))
                        }
                        _ => Err(anyhow!("`==` for NaN")).with_context(|| context!()),
                    }
                }
                BinaryOp::Ne => {
                    // Generate arguments and get type to cast
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
                            let inst =
                                self.program
                                    .mem_pool
                                    .get_fcmp(FCmpOp::One, max_ty, lop, rop);
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
                    let lop = lhs_val.load(ValueType::Bool, self)?;
                    let mut primary_exit = self.exit.unwrap();
                    primary_exit.push_back(self.program.mem_pool.get_br(Some(lop)));
                    primary_exit.set_true_bb(alt_entry);
                    primary_exit.set_false_bb(final_entry);

                    // Load right operand to alt block, jump to final block
                    self.exit = Some(alt_entry);
                    let rop = self.gen_expr(rhs)?.load(ValueType::Bool, self)?;
                    let mut alt_exit: duskphantom_utils::mem::ObjPtr<
                        crate::ir::BasicBlock,
                    > = self.exit.unwrap();
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
                    let lop = lhs_val.load(ValueType::Bool, self)?;
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
            }?;
        }
        Ok(lhs_val)
    }
}
