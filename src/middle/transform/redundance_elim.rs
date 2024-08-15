use std::collections::HashMap;

use anyhow::Result;

use crate::{
    middle::{
        analysis::{
            dominator_tree::DominatorTree,
            effect_analysis::EffectAnalysis,
            simple_gvn::{Expr, SimpleGVN},
        },
        ir::{BBPtr, InstPtr, Operand, ValueType},
        Program,
    },
    utils::frame_map::FrameMap,
};

use super::Transform;

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    let mut gvn = SimpleGVN::new(&effect_analysis);
    RedundanceElim::new(program, &mut gvn).run_and_log()
}

pub struct RedundanceElim<'a> {
    program: &'a mut Program,
    gvn: &'a mut SimpleGVN<'a>,
}

impl<'a> Transform for RedundanceElim<'a> {
    fn name() -> String {
        "redundance_elim".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        // Get instruction leader from GVN
        let mut inst_leader = HashMap::new();
        for func in self.program.module.functions.clone() {
            let Some(bb) = func.entry else {
                continue;
            };
            let mut dom_tree = DominatorTree::new(func);
            let expr_leader = FrameMap::new();
            dfs(self.gvn, bb, &mut inst_leader, expr_leader, &mut dom_tree);
        }

        // Replace instruction with its leader
        let mut changed = false;
        for (inst, leader) in inst_leader.iter() {
            let leader = (*leader).into();
            inst.clone().replace_self(&leader);
            changed = true;
        }
        Ok(changed)
    }
}

impl<'a> RedundanceElim<'a> {
    pub fn new(program: &'a mut Program, gvn: &'a mut SimpleGVN<'a>) -> Self {
        Self { program, gvn }
    }
}

/// Analyse instruction leader from GVN
fn dfs<'a>(
    gvn: &mut SimpleGVN<'a>,
    bb: BBPtr,
    inst_leader: &mut HashMap<InstPtr, InstPtr>,
    mut expr_leader: FrameMap<'_, Expr<'a>, InstPtr>,
    dom_tree: &mut DominatorTree,
) {
    bb.iter().for_each(|inst| {
        // Refuse to replace instruction that returns void
        if inst.get_value_type() == ValueType::Void {
            return;
        }
        let expr = gvn.get_expr(Operand::Instruction(inst));
        match expr_leader.get(&expr) {
            // Expression appeared before, set instruction leader
            Some(&leader) => {
                inst_leader.insert(inst, leader);
            }
            // Expression not appeared before, set as leader
            None => {
                expr_leader.insert(expr, inst);
            }
        }
    });
    for succ in dom_tree.get_dominatee(bb) {
        dfs(gvn, succ, inst_leader, expr_leader.branch(), dom_tree);
    }
}
