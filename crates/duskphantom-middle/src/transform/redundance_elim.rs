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

use std::collections::HashMap;

use anyhow::Result;

use crate::{
    analysis::{
        dominator_tree::DominatorTree,
        effect_analysis::EffectAnalysis,
        memory_ssa::MemorySSA,
        simple_gvn::{Expr, SimpleGVN},
    },
    ir::{InstPtr, Operand, ValueType},
    Program,
};

use super::Transform;

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    let memory_ssa = MemorySSA::new(program, &effect_analysis);
    let mut gvn = SimpleGVN::new(&memory_ssa);
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

            // Implementation of Expr::Hash does not use it's mutable content,
            // so it's false positive according to:
            // https://rust-lang.github.io/rust-clippy/master/index.html#/mutable_key_type
            #[allow(clippy::mutable_key_type)]
            let mut expr_leader: HashMap<Expr, InstPtr> = HashMap::new();

            // Iterate all instructions
            for bb in func.rpo_iter() {
                for inst in bb.iter() {
                    // Refuse to replace instruction that returns void
                    if inst.get_value_type() == ValueType::Void {
                        continue;
                    }
                    let expr = self.gvn.get_expr(Operand::Instruction(inst));
                    match expr_leader.get(&expr) {
                        // Expression appeared before, move leader and inst to lowest common ancestor
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
