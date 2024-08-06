use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use anyhow::Result;

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        analysis::dominator_tree::DominatorTree,
        ir::{
            instruction::{
                misc_inst::{FCmp, ICmp},
                InstType,
            },
            BBPtr, FunPtr, InstPtr, Operand, ValueType,
        },
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
pub struct Expr {
    op: Operand,
    num: u64,
}

impl Expr {
    /// Create a value-numbered expression from operand
    pub fn new(ctx: &mut SimpleGVN, op: Operand) -> Self {
        // If operand is not inst, construct expression directly
        let Operand::Instruction(inst) = op else {
            let mut hasher = DefaultHasher::new();
            op.hash(&mut hasher);
            return Self {
                op,
                num: hasher.finish(),
            };
        };

        // If inst is not touched, construct expression
        let Some(expr) = ctx.exprs.get(&inst) else {
            let num = Self::get_num(ctx, inst);
            let expr = Self { op, num };
            ctx.exprs.insert(inst, expr.clone());
            return expr;
        };

        // If inst is touched, return cached expression
        expr.clone()
    }

    /// Get value number for instruction
    fn get_num(ctx: &mut SimpleGVN, inst: InstPtr) -> u64 {
        let mut hasher = DefaultHasher::new();
        // Some instructions equal only when they are the same instance
        // TODO pure function analysis
        let ty = inst.get_type();
        if let InstType::Alloca | InstType::Call | InstType::Load | InstType::Phi = ty {
            inst.hash(&mut hasher);
            return hasher.finish();
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
            .map(|op| Self::new(ctx, op.clone()).num)
            .collect::<Vec<_>>();
        numbers.sort_unstable();
        for num in numbers {
            num.hash(&mut hasher);
        }
        hasher.finish()
    }
}

impl Hash for Expr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.num.hash(state);
    }
}

impl PartialEq for Expr {
    fn eq(&self, other: &Self) -> bool {
        match (&self.op, &other.op) {
            (Operand::Instruction(inst1), Operand::Instruction(inst2)) => {
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
            _ => self.op == other.op,
        }
    }
}

impl Eq for Expr {}

impl From<Operand> for Expr {
    fn from(op: Operand) -> Self {
        Self { op, num: 0 }
    }
}

#[allow(unused)]
pub struct SimpleGVN {
    exprs: HashMap<InstPtr, Expr>,
    fun: FunPtr,
    dom_tree: DominatorTree,
}

#[allow(unused)]
impl SimpleGVN {
    pub fn new(fun: FunPtr) -> Self {
        Self {
            fun,
            exprs: HashMap::new(),
            dom_tree: DominatorTree::new(fun),
        }
    }

    fn run(&mut self) {
        self.dfs(self.fun.entry.unwrap(), FrameMap::new());
    }

    fn dfs(&mut self, bb: BBPtr, mut expr_leader: FrameMap<'_, Expr, InstPtr>) {
        bb.iter().for_each(|mut inst| {
            // Refuse to replace instruction that returns void
            if inst.get_value_type() == ValueType::Void {
                return;
            }
            let expr = Expr::new(self, Operand::Instruction(inst));
            match expr_leader.get(&expr) {
                // Expression appeared before, replace instruction with leader
                Some(&leader) => {
                    inst.replace_self(&leader.into());
                }
                // Expression not appeared before, set as leader
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
