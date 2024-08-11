use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    backend::from_self::downcast_ref,
    middle::{
        ir::{
            instruction::{
                misc_inst::{Call, FCmp, ICmp},
                InstType,
            },
            BBPtr, InstPtr, Operand, ValueType,
        },
        Program,
    },
    utils::frame_map::FrameMap,
};

use super::{dominator_tree::DominatorTree, effect_analysis::EffectAnalysis};

pub struct SimpleGVN<'a> {
    ctx: Context<'a>,
    pub inst_expr: HashMap<InstPtr, Expr<'a>>,
    pub inst_leader: HashMap<InstPtr, InstPtr>,
}

#[derive(Clone, Copy)]
struct Context<'a> {
    effect_analysis: &'a EffectAnalysis,
}

impl<'a> SimpleGVN<'a> {
    pub fn new(program: &Program, effect_analysis: &'a EffectAnalysis) -> Self {
        let ctx = Context { effect_analysis };
        let mut simple_gvn = Self {
            ctx,
            inst_expr: HashMap::new(),
            inst_leader: HashMap::new(),
        };
        for func in program.module.functions.clone() {
            if func.is_lib() {
                continue;
            }
            let mut dom_tree = DominatorTree::new(func);
            simple_gvn.dfs(func.entry.unwrap(), FrameMap::new(), &mut dom_tree);
        }
        simple_gvn
    }

    fn dfs<'b>(
        &'b mut self,
        bb: BBPtr,
        mut expr_leader: FrameMap<'_, Expr<'a>, InstPtr>,
        dom_tree: &'b mut DominatorTree,
    ) {
        bb.iter().for_each(|inst| {
            // Refuse to replace instruction that returns void
            if inst.get_value_type() == ValueType::Void {
                return;
            }
            let expr = Expr::new(self, Operand::Instruction(inst));
            match expr_leader.get(&expr) {
                // Expression appeared before, set instruction leader
                Some(&leader) => {
                    self.inst_leader.insert(inst, leader);
                }
                // Expression not appeared before, set as leader
                None => {
                    expr_leader.insert(expr, inst);
                }
            }
        });
        for succ in dom_tree.get_dominatee(bb) {
            self.dfs(succ, expr_leader.branch(), dom_tree);
        }
    }
}

#[derive(Clone)]
pub struct Expr<'a> {
    ctx: Context<'a>,
    op: Operand,
    num: u64,
}

impl<'a> Expr<'a> {
    /// Create a value-numbered expression from operand
    pub fn new(simple_gvn: &mut SimpleGVN<'a>, op: Operand) -> Self {
        // If operand is not inst, construct expression directly
        let Operand::Instruction(inst) = op else {
            let mut hasher = DefaultHasher::new();
            op.hash(&mut hasher);
            return Self {
                ctx: simple_gvn.ctx,
                op,
                num: hasher.finish(),
            };
        };

        // If inst is not touched, construct expression
        let Some(expr) = simple_gvn.inst_expr.get(&inst) else {
            let num = Self::get_num(simple_gvn, inst);
            let expr = Self {
                ctx: simple_gvn.ctx,
                op,
                num,
            };
            simple_gvn.inst_expr.insert(inst, expr.clone());
            return expr;
        };

        // If inst is touched, return cached expression
        expr.clone()
    }

    /// Get value number for instruction
    fn get_num(simple_gvn: &mut SimpleGVN<'a>, inst: InstPtr) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Some instructions equal only when they are the same instance
        let ty = inst.get_type();
        if let InstType::Alloca | InstType::Load | InstType::Phi = ty {
            inst.hash(&mut hasher);
            return hasher.finish();
        }

        // Impure function equal only when they are the same instance
        if ty == InstType::Call && simple_gvn.ctx.effect_analysis.has_effect(inst) {
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
            .map(|op| Self::new(simple_gvn, op.clone()).num)
            .collect::<Vec<_>>();
        numbers.sort_unstable();
        for num in numbers {
            num.hash(&mut hasher);
        }
        hasher.finish()
    }
}

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
                if let InstType::Alloca | InstType::Load | InstType::Phi = ty {
                    return inst1 == inst2;
                }

                // Impure function equal only when they are the same instance
                if ty == InstType::Call
                    && self.ctx.effect_analysis.has_effect(*inst1)
                    && self.ctx.effect_analysis.has_effect(*inst2)
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
