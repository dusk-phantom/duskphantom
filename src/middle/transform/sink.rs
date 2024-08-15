use std::collections::{HashMap, HashSet};

use anyhow::{anyhow, Context, Result};

use crate::context;
use crate::middle::analysis::dominator_tree::DominatorTree;
use crate::middle::analysis::effect_analysis::EffectAnalysis;
use crate::middle::ir::instruction::InstType;
use crate::middle::ir::{BBPtr, InstPtr};
use crate::middle::Program;

use super::Transform;

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    Sink::new(program, &effect_analysis).run_and_log()
}

#[allow(unused)]
pub struct Sink<'a> {
    program: &'a mut Program,
    effect_analysis: &'a EffectAnalysis,
    visited: HashSet<InstPtr>,
}

#[allow(unused)]
impl<'a> Transform for Sink<'a> {
    fn name() -> String {
        "dead_code_elim".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for func in self.program.module.functions.clone().iter() {
            if func.is_lib() {
                continue;
            }
            for bb in func.po_iter() {
                for inst in bb.iter() {
                    if self.visited.contains(&inst) {
                        continue;
                    }
                }
            }
        }
        Ok(true)
    }
}

#[allow(unused)]
impl<'a> Sink<'a> {
    pub fn new(program: &'a mut Program, effect_analysis: &'a EffectAnalysis) -> Self {
        Self {
            program,
            effect_analysis,
            visited: HashSet::new(),
        }
    }

    fn sink_inst(&mut self, inst: InstPtr, dom_tree: &mut DominatorTree) -> Result<bool> {
        // Refuse to sink instruction with side effect, or visited instruction
        if self.has_side_effect(inst) || self.visited.contains(&inst) {
            return Ok(false);
        }

        // Sink users first
        let mut changed = false;
        self.visited.insert(inst);
        for user in inst.get_user() {
            changed |= self.sink_inst(*user, dom_tree)?;
        }

        // If any user is in the same block, do not sink
        let root = inst
            .get_parent_bb()
            .ok_or_else(|| anyhow!("Instruction {} has no parent BB", inst))
            .with_context(|| context!())?;
        for user in inst.get_user() {
            if Some(root) == user.get_parent_bb() {
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
        if root.get_succ_bb().len() == 2 {
            let mut block_to_inst: HashMap<BBPtr, HashSet<InstPtr>> = HashMap::new();
            for user in inst.get_user() {
                let user_bb = user
                    .get_parent_bb()
                    .ok_or_else(|| anyhow!("Instruction {} has no parent BB", user))
                    .with_context(|| context!())?;
                block_to_inst.entry(user_bb).or_default().insert(*user);
            }

            // Get mapping from dominatee to block
            let mut dominatee_to_inst: HashMap<BBPtr, HashSet<InstPtr>> = HashMap::new();
            for (bb, _) in block_to_inst.iter() {
                let mut cursor = *bb;
                let dominatee = loop {
                    let idom = dom_tree.get_idom(cursor).unwrap();
                    if idom == root {
                        break cursor;
                    }
                    cursor = idom;
                };
                dominatee_to_inst
                    .entry(dominatee)
                    .or_default()
                    .extend(&block_to_inst[bb]);
            }

            // Check if there are users in (A OR B) branch
            let mut exist_in_succ = false;
            for succ in root.get_succ_bb() {
                if dominatee_to_inst.contains_key(succ) {
                    exist_in_succ = true;
                }
            }

            // If there are users in (A OR B) branch
            if exist_in_succ {
                let mem_pool = &mut self.program.mem_pool;
                let mut new_insts = [
                    mem_pool.copy_instruction(inst.as_ref().as_ref()),
                    mem_pool.copy_instruction(inst.as_ref().as_ref()),
                ];

                // Sink into (A AND B)
                for (i, new_inst) in new_insts.iter_mut().enumerate() {
                    for op in inst.get_operand() {
                        new_inst.add_operand(op.clone());
                    }
                    // TODO put as below as possible
                    root.get_succ_bb()[i].clone().push_front(*new_inst);
                }

                // TODO insert phi, replace operand
            }

            // TODO other case
        }

        Ok(changed)
    }

    fn has_side_effect(&mut self, inst: InstPtr) -> bool {
        matches!(
            inst.get_type(),
            InstType::Store | InstType::Ret | InstType::Br
        ) || self.effect_analysis.has_effect(inst)
    }
}
