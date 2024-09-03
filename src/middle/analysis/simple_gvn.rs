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

use crate::middle::ir::instruction::downcast_ref;
use crate::middle::ir::{
    instruction::{
        misc_inst::{Call, FCmp, ICmp},
        InstType,
    },
    InstPtr, Operand,
};
use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use super::memory_ssa::MemorySSA;

pub struct SimpleGVN<'a> {
    ctx: Context<'a>,
    inst_expr: HashMap<InstPtr, Expr<'a>>,
}

#[derive(Clone, Copy)]
struct Context<'a> {
    memory_ssa: &'a MemorySSA<'a>,
}

impl<'a> SimpleGVN<'a> {
    pub fn new(memory_ssa: &'a MemorySSA<'a>) -> Self {
        let ctx = Context { memory_ssa };
        Self {
            ctx,
            inst_expr: HashMap::new(),
        }
    }

    /// Create a value-numbered expression from operand
    pub fn get_expr(&mut self, op: Operand) -> Expr<'a> {
        // If operand is not inst, construct expression directly
        let Operand::Instruction(inst) = op else {
            let mut hasher = DefaultHasher::new();
            op.hash(&mut hasher);
            return Expr {
                ctx: self.ctx,
                op,
                num: hasher.finish(),
            };
        };

        // If inst is not touched, construct expression
        let Some(expr) = self.inst_expr.get(&inst) else {
            let num = self.get_num(inst);
            let expr = Expr {
                ctx: self.ctx,
                op,
                num,
            };
            self.inst_expr.insert(inst, expr.clone());
            return expr;
        };

        // If inst is touched, return cached expression
        expr.clone()
    }

    /// Get value number for instruction
    fn get_num(&mut self, inst: InstPtr) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Some instructions equal only when they are the same instance
        let ty = inst.get_type();
        if let InstType::Alloca | InstType::Phi = ty {
            inst.hash(&mut hasher);
            return hasher.finish();
        }

        // Hash corresponding MemoryDef node for loads
        if let InstType::Load = ty {
            let node = self.ctx.memory_ssa.get_inst_node(inst).unwrap();
            let use_node = node.get_use_node();
            use_node.hash(&mut hasher);
        }

        // Impure function equal only when they are the same instance
        if ty == InstType::Call && self.ctx.memory_ssa.effect_analysis.has_effect(inst) {
            inst.hash(&mut hasher);
            return hasher.finish();
        }

        // Hash called function for pure function call
        if ty == InstType::Call {
            let call = downcast_ref::<Call>(inst.as_ref().as_ref());
            call.func.hash(&mut hasher);
        }

        // Hash condition for compare instruction
        if matches!(ty, InstType::ICmp) {
            let cmp = downcast_ref::<ICmp>(inst.as_ref().as_ref());
            cmp.op.hash(&mut hasher);
        } else if matches!(ty, InstType::FCmp) {
            let cmp = downcast_ref::<FCmp>(inst.as_ref().as_ref());
            cmp.op.hash(&mut hasher);
        }

        // Hash instruction type
        inst.get_type().hash(&mut hasher);

        // Hash number of operands in canonical order
        let mut numbers = inst
            .get_operand()
            .iter()
            .map(|op| self.get_expr(op.clone()).num)
            .collect::<Vec<_>>();
        numbers.sort_unstable();
        for num in numbers {
            num.hash(&mut hasher);
        }
        hasher.finish()
    }
}

#[derive(Clone)]
pub struct Expr<'a> {
    ctx: Context<'a>,
    op: Operand,
    num: u64,
}

impl<'a> Expr<'a> {}

impl<'a> Hash for Expr<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.num.hash(state);
    }
}

impl<'a> PartialEq for Expr<'a> {
    fn eq(&self, other: &Self) -> bool {
        match (&self.op, &other.op) {
            (Operand::Instruction(inst1), Operand::Instruction(inst2)) => {
                // If instruction type is not the same, their value is not the same
                let ty = inst1.get_type();
                if ty != inst2.get_type() {
                    return false;
                }

                // Some instructions equal only when they are the same instance
                if let InstType::Alloca | InstType::Phi = ty {
                    return inst1 == inst2;
                }

                // For loads, they should use the same MemoryDef node
                if let InstType::Load = ty {
                    let node1 = self.ctx.memory_ssa.get_inst_node(*inst1).unwrap();
                    let node2 = self.ctx.memory_ssa.get_inst_node(*inst2).unwrap();
                    let use1 = node1.get_use_node();
                    let use2 = node2.get_use_node();
                    if use1 != use2 {
                        return false;
                    }
                }

                // Impure function equal only when they are the same instance
                if ty == InstType::Call
                    && self.ctx.memory_ssa.effect_analysis.has_effect(*inst1)
                    && self.ctx.memory_ssa.effect_analysis.has_effect(*inst2)
                {
                    return inst1 == inst2;
                }

                // Compare called function for pure function call
                if ty == InstType::Call {
                    let call1 = downcast_ref::<Call>(inst1.as_ref().as_ref());
                    let call2 = downcast_ref::<Call>(inst2.as_ref().as_ref());
                    if call1.func != call2.func {
                        return false;
                    }
                }

                // Compare condition for compare instruction
                if matches!(ty, InstType::ICmp) {
                    let cmp1 = downcast_ref::<ICmp>(inst1.as_ref().as_ref());
                    let cmp2 = downcast_ref::<ICmp>(inst2.as_ref().as_ref());
                    if cmp1.op != cmp2.op {
                        return false;
                    }
                } else if matches!(ty, InstType::FCmp) {
                    let cmp1 = downcast_ref::<FCmp>(inst1.as_ref().as_ref());
                    let cmp2 = downcast_ref::<FCmp>(inst2.as_ref().as_ref());
                    if cmp1.op != cmp2.op {
                        return false;
                    }
                }

                // If number of operands is not the same, their value is not the same
                if inst1.get_operand().len() != inst2.get_operand().len() {
                    return false;
                }

                // Compare all operands in order
                let all_the_same = inst1
                    .get_operand()
                    .iter()
                    .zip(inst2.get_operand().iter())
                    .all(|(op1, op2)| {
                        let expr1: Expr = Expr {
                            ctx: self.ctx,
                            op: op1.clone(),
                            num: 0,
                        };
                        let expr2: Expr = Expr {
                            ctx: self.ctx,
                            op: op2.clone(),
                            num: 0,
                        };
                        expr1 == expr2
                    });

                // Check if instruction is commutative
                let commutative = matches!(
                    ty,
                    InstType::Add
                        | InstType::Mul
                        | InstType::FAdd
                        | InstType::FMul
                        | InstType::And
                        | InstType::Or
                        | InstType::Xor
                );

                // If instruction is commutative, compare all operands in reverse order
                let all_the_same_rev = commutative
                    && inst1
                        .get_operand()
                        .iter()
                        .rev()
                        .zip(inst2.get_operand().iter())
                        .all(|(op1, op2)| {
                            let expr1: Expr = Expr {
                                ctx: self.ctx,
                                op: op1.clone(),
                                num: 0,
                            };
                            let expr2: Expr = Expr {
                                ctx: self.ctx,
                                op: op2.clone(),
                                num: 0,
                            };
                            expr1 == expr2
                        });

                // Return result
                all_the_same || all_the_same_rev
            }
            _ => self.op == other.op,
        }
    }
}

impl<'a> Eq for Expr<'a> {}
