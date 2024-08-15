use std::collections::HashMap;

use anyhow::Result;

use crate::middle::{
    analysis::{
        dominator_tree::DominatorTree,
        effect_analysis::EffectAnalysis,
        simple_gvn::{Expr, SimpleGVN},
    },
    ir::{InstPtr, Operand, ValueType},
    Program,
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
    fn get_program_mut(&mut self) -> &mut Program {
        self.program
    }

    fn name() -> String {
        "redundance_elim".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        // Get instruction leader from GVN
        let mut changed = false;
        for func in self.program.module.functions.clone() {
            if func.is_lib() {
                continue;
            }
            let mut dom_tree = DominatorTree::new(func);
            let mut expr_leader: HashMap<Expr, InstPtr> = HashMap::new();
            for bb in func.rpo_iter() {
                for inst in bb.iter() {
                    // Refuse to replace instruction that returns void
                    if inst.get_value_type() == ValueType::Void {
                        continue;
                    }
                    let expr = self.gvn.get_expr(Operand::Instruction(inst));
                    match expr_leader.get(&expr) {
                        // Expression appeared before, remove redundancy
                        Some(&leader) => {
                            let inst_bb = inst.get_parent_bb().unwrap();
                            let leader_bb = leader.get_parent_bb().unwrap();
                            let lca = dom_tree.get_lca(inst_bb, leader_bb);
                            if lca != leader_bb {
                                // Remove partial redundancy and hoist at the same time
                                lca.get_last_inst().insert_before(leader);
                            }
                            inst.clone().replace_self(&leader.into());
                            changed = true;
                        }
                        // Expression not appeared before, set as leader
                        None => {
                            expr_leader.insert(expr, inst);
                        }
                    }
                }
            }
        }
        Ok(changed)
    }
}

impl<'a> RedundanceElim<'a> {
    pub fn new(program: &'a mut Program, gvn: &'a mut SimpleGVN<'a>) -> Self {
        Self { program, gvn }
    }
}
