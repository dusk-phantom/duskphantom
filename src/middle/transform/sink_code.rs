use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Context, Result};

use crate::backend::from_self::downcast_ref;
use crate::context;
use crate::middle::analysis::dominator_tree::DominatorTree;
use crate::middle::analysis::effect_analysis::EffectAnalysis;
use crate::middle::ir::instruction::misc_inst::Phi;
use crate::middle::ir::instruction::{downcast_mut, InstType};
use crate::middle::ir::{BBPtr, InstPtr, Operand};
use crate::middle::Program;

use super::Transform;

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    SinkCode::new(program, &effect_analysis).run_and_log()
}

#[allow(unused)]
pub struct SinkCode<'a> {
    program: &'a mut Program,
    effect_analysis: &'a EffectAnalysis,
}

#[allow(unused)]
impl<'a> Transform for SinkCode<'a> {
    fn get_program_mut(&mut self) -> &mut Program {
        self.program
    }

    fn name() -> String {
        "sink_code".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for func in self.program.module.functions.clone() {
            if func.is_lib() {
                continue;
            }
            let mut dom_tree = DominatorTree::new(func);
            for bb in func.po_iter() {
                for inst in bb.iter_rev() {
                    changed |= self.sink_inst(inst, &mut dom_tree)?;
                }
            }
        }
        Ok(true)
    }
}

#[allow(unused)]
impl<'a> SinkCode<'a> {
    pub fn new(program: &'a mut Program, effect_analysis: &'a EffectAnalysis) -> Self {
        Self {
            program,
            effect_analysis,
        }
    }

    fn sink_inst(&mut self, mut inst: InstPtr, dom_tree: &mut DominatorTree) -> Result<bool> {
        let mut changed = false;

        // Refuse to sink instruction with side effect
        if self.is_fixed(inst) {
            return Ok(changed);
        }

        // If any user is in the same block, do not sink
        // TODO even in same BB it can sink as low as possible
        let root = inst
            .get_parent_bb()
            .ok_or_else(|| anyhow!("Instruction {} has no parent BB", inst))
            .with_context(|| context!())?;
        for user in FakeInst::from_inst_users(inst)? {
            let parent_bb = user.get_parent_bb()?;
            if root == parent_bb {
                return Ok(changed);
            }
        }

        // If there are two successors, sink into both and create necessary phi.
        //
        // Suppose bb dominates the two successors (A, B), and other blocks (C, D),
        // if there are users in (A OR B) branch, we can only sink it into (A AND B).
        // To remove partial redundancy we insert phi for (C, D), and phi can't be sunk.
        //
        // Otherwise we can sink it into (C, D), and sink them recursively.
        // Time complexity is O(n * log(n)) because each time users are partitioned.
        //
        // TODO below is a temporary implementation, it refuses to sink if there are users in (C OR D).
        if root.get_succ_bb().len() == 2 {
            let mut block_to_user: HashMap<BBPtr, HashSet<FakeInst>> = HashMap::new();
            for user in FakeInst::from_inst_users(inst)? {
                let user_bb = user.get_parent_bb()?;
                block_to_user.entry(user_bb).or_default().insert(user);
            }

            // Get mapping from dominatee to users
            let mut dominatee_to_user: HashMap<BBPtr, HashSet<FakeInst>> = HashMap::new();
            for (bb, _) in block_to_user.iter() {
                let mut cursor = *bb;
                let dominatee = loop {
                    let idom = dom_tree
                        .get_idom(cursor)
                        .ok_or_else(|| {
                            anyhow!("{} has no immediate dominator ({})", cursor.name, bb.name)
                        })
                        .with_context(|| context!())?;
                    if idom == root {
                        break cursor;
                    }
                    cursor = idom;
                };
                dominatee_to_user
                    .entry(dominatee)
                    .or_default()
                    .extend(block_to_user[bb].clone());
            }

            // Check if there are users in (C OR D) branch
            let mut exist_in_other = false;
            for (k, v) in dominatee_to_user.iter() {
                if !root.get_succ_bb().contains(k) && !v.is_empty() {
                    exist_in_other = true;
                    break;
                }
            }

            // If not, sink into each successor
            if !exist_in_other {
                changed = true;
                for succ in root.get_succ_bb() {
                    let user = dominatee_to_user
                        .get(succ)
                        .cloned()
                        .unwrap_or(HashSet::new());
                    if !user.is_empty() {
                        let mut new_inst = self
                            .program
                            .mem_pool
                            .copy_instruction(inst.as_ref().as_ref());
                        for op in inst.get_operand() {
                            new_inst.add_operand(op.clone());
                        }

                        // Get insert position (as low as possible)
                        let user_in_succ =
                            block_to_user.get(succ).cloned().unwrap_or(HashSet::new());
                        let mut frontier = None;
                        for inst in succ.iter() {
                            if user_in_succ.contains(&FakeInst::Normal(inst)) {
                                frontier = Some(inst);
                                break;
                            }
                        }

                        // Insert new instruction and maintain use-def chain
                        if let Some(mut frontier) = frontier {
                            frontier.insert_before(new_inst);
                        } else {
                            succ.get_last_inst().insert_before(new_inst);
                        }
                        for mut user in user {
                            user.replace_operand(&inst.into(), &new_inst.into());
                        }

                        // Sink recursively
                        self.sink_inst(new_inst, dom_tree)?;
                    }
                }

                // Remove the original instruction
                inst.remove_self();
            }
        }

        // If there is only one successor, and it's dominated
        // (the successor has only one predecessor), sink into it
        if root.get_succ_bb().len() == 1 {
            let succ = root.get_succ_bb().first().unwrap();
            if succ.get_pred_bb().len() == 1 {
                changed = true;
                succ.clone().push_front(inst);
                self.sink_inst(inst, dom_tree)?;
            }
        }

        Ok(changed)
    }

    fn is_fixed(&mut self, inst: InstPtr) -> bool {
        matches!(
            inst.get_type(),
            InstType::Load | InstType::Store | InstType::Ret | InstType::Br | InstType::Phi
        ) || self.effect_analysis.has_effect(inst)
    }
}

/// We treat `phi` as multiple fake instructions, each corresponds to one incoming value.
/// This way it's unified with normal instruction.
#[derive(PartialEq, Eq, Hash, Clone)]
enum FakeInst {
    Normal(InstPtr),
    Phi(InstPtr, BBPtr),
}

impl FakeInst {
    fn from_inst_users(inst: InstPtr) -> Result<Vec<FakeInst>> {
        inst.get_user()
            .iter()
            .map(|user| FakeInst::from_inst_user(inst, *user))
            .collect::<Result<Vec<Vec<_>>>>()
            .map(|v| v.into_iter().flatten().collect())
    }

    fn from_inst_user(inst: InstPtr, user: InstPtr) -> Result<Vec<FakeInst>> {
        if user.get_type() == InstType::Phi {
            let phi = downcast_ref::<Phi>(user.as_ref().as_ref());
            let mut result = Vec::new();
            for (op, bb) in phi.get_incoming_values() {
                if op == &Operand::Instruction(inst) {
                    result.push(FakeInst::Phi(user, *bb));
                }
            }
            Ok(result)
        } else {
            Ok(vec![FakeInst::Normal(user)])
        }
    }

    fn get_parent_bb(&self) -> Result<BBPtr> {
        match self {
            FakeInst::Normal(inst) => inst
                .get_parent_bb()
                .ok_or_else(|| anyhow!("Instruction {} has no parent BB", inst))
                .with_context(|| context!()),
            // Phi FakeInst locates in one of data source block
            FakeInst::Phi(_, bb) => Ok(*bb),
        }
    }

    fn replace_operand(&mut self, old: &Operand, new: &Operand) {
        match self {
            FakeInst::Normal(inst) => {
                inst.replace_operand(old, new);
            }
            FakeInst::Phi(inst, bb) => {
                let phi = downcast_mut::<Phi>(inst.as_mut().as_mut());
                phi.replace_incoming_value_at(*bb, new.clone());
            }
        }
    }
}
