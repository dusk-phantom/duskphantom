use std::{
    collections::{HashMap, HashSet},
    hash::{Hash, Hasher},
};

use crate::middle::ir::{instruction::InstType, FunPtr, InstPtr, Operand};

#[derive(Clone)]
pub enum Expr {
    Inst(InstPtr),
    Operand(Operand),
}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Expr::Inst(inst) => {
                // Some instructions will not equal even if they have the same type and operands
                // TODO pure function analysis
                let ty = inst.get_type();
                if let InstType::Alloca | InstType::Call | InstType::Load | InstType::Phi = ty {
                    inst.hash(state);
                    return;
                }

                // Hash instruction type
                inst.get_type().hash(state);

                // Hash each operand
                inst.get_operand().iter().for_each(|op| {
                    let expr: Expr = op.clone().into();
                    expr.hash(state);
                });
            }
            Expr::Operand(op) => op.hash(state),
        }
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Expr::Inst(inst1), Expr::Inst(inst2)) => {
                // If instruction type is not the same, their value is not the same
                let ty = inst1.get_type();
                if ty != inst2.get_type() {
                    return false;
                }

                // Some instructions will not equal even if they have the same type and operands
                // TODO pure function analysis
                if let InstType::Alloca | InstType::Call | InstType::Load | InstType::Phi = ty {
                    return false;
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
                        let expr1: Expr = op1.clone().into();
                        let expr2: Expr = op2.clone().into();
                        expr1 == expr2
                    });

                // Check if instruction is commutative
                let commutative = matches!(
                    ty,
                    InstType::Add | InstType::Mul | InstType::And | InstType::Or | InstType::Xor
                );

                // If instruction is commutative, compare all operands in reverse order
                let all_the_same_rev = commutative
                    && inst1
                        .get_operand()
                        .iter()
                        .rev()
                        .zip(inst2.get_operand().iter())
                        .all(|(op1, op2)| {
                            let expr1: Expr = op1.clone().into();
                            let expr2: Expr = op2.clone().into();
                            expr1 == expr2
                        });

                // Return result
                all_the_same || all_the_same_rev
            }
            (Expr::Operand(op1), Expr::Operand(op2)) => op1 == op2,
            _ => false,
        }
    }
}

impl Eq for Expr {}

impl From<Operand> for Expr {
    fn from(op: Operand) -> Self {
        match op {
            Operand::Instruction(inst) => Self::Inst(inst),
            _ => Self::Operand(op),
        }
    }
}

impl From<InstPtr> for Expr {
    fn from(inst: InstPtr) -> Self {
        Self::Inst(inst)
    }
}

#[allow(unused)]
pub struct SimpleGVN {
    inst_to_expr: HashMap<InstPtr, Expr>,
    expr_to_inst: HashMap<Expr, HashSet<InstPtr>>,
}

#[allow(unused)]
impl SimpleGVN {
    pub fn new(fun: FunPtr) -> Self {
        let mut inst_to_expr = HashMap::new();
        let mut expr_to_inst = HashMap::new();
        fun.rpo_iter().for_each(|bb| {
            bb.iter().for_each(|inst| {
                let expr: Expr = inst.into();
                inst_to_expr.insert(inst, expr.clone());
                expr_to_inst
                    .entry(expr)
                    .or_insert_with(HashSet::new)
                    .insert(inst);
            })
        });
        Self {
            inst_to_expr,
            expr_to_inst,
        }
    }
}
