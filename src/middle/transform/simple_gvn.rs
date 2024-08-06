use std::hash::{Hash, Hasher};

use anyhow::Result;

use crate::{
    middle::{
        analysis::dominator_tree::DominatorTree,
        ir::{instruction::InstType, BBPtr, FunPtr, InstPtr, Operand},
        Program,
    },
    utils::frame_map::FrameMap,
};

pub fn optimize_program(program: &mut Program) -> Result<()> {
    for fun in program.module.functions.iter().filter(|f| !f.is_lib()) {
        SimpleGVN::new(*fun).run();
    }
    Ok(())
}

#[derive(Clone)]
pub enum Expr {
    Inst(InstPtr),
    Operand(Operand),
}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Expr::Inst(inst) => {
                // Some instructions equal only when they are the same instance
                // TODO pure function analysis
                let ty = inst.get_type();
                if let InstType::Alloca | InstType::Call | InstType::Load | InstType::Phi = ty {
                    inst.hash(state);
                    return;
                }

                // Hash instruction type
                // TODO we can hash operands when they're in canonical order
                inst.get_type().hash(state);
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

                // Some instructions equal only when they are the same instance
                // TODO pure function analysis
                if let InstType::Alloca | InstType::Call | InstType::Load | InstType::Phi = ty {
                    return inst1 == inst2;
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
    fun: FunPtr,
    dom_tree: DominatorTree,
}

#[allow(unused)]
impl SimpleGVN {
    pub fn new(fun: FunPtr) -> Self {
        Self {
            fun,
            dom_tree: DominatorTree::new(fun),
        }
    }

    pub fn run(&mut self) {
        self.dfs(self.fun.entry.unwrap(), FrameMap::new());
    }

    fn dfs(&mut self, bb: BBPtr, mut expr_leader: FrameMap<'_, Expr, InstPtr>) {
        bb.iter().for_each(|mut inst| {
            let expr: Expr = inst.into();
            match expr_leader.get(&expr) {
                Some(&leader) => {
                    inst.replace_self(&leader.into());
                }
                None => {
                    expr_leader.insert(expr, inst);
                }
            }
        });
        for succ in self.dom_tree.get_dominatee(bb) {
            self.dfs(succ, expr_leader.branch());
        }
    }
}
