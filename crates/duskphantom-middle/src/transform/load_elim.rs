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

use anyhow::Result;

use crate::{
    analysis::memory_ssa::{MemorySSA, Node},
    ir::{instruction::InstType, FunPtr, InstPtr},
    Program,
};

use super::Transform;

pub fn optimize_program<'a>(
    program: &'a mut Program,
    memory_ssa: &'a mut MemorySSA,
) -> Result<bool> {
    LoadElim::new(program, memory_ssa).run_and_log()
}

pub struct LoadElim<'a, 'b> {
    program: &'a mut Program,
    memory_ssa: &'a mut MemorySSA<'b>,
}

impl<'a, 'b> Transform for LoadElim<'a, 'b> {
    fn get_program_mut(&mut self) -> &mut Program {
        self.program
    }

    fn name() -> String {
        "load_elim".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for func in self.program.module.functions.clone() {
            if func.is_lib() {
                continue;
            }
            for bb in func.rpo_iter() {
                for inst in bb.iter() {
                    changed |= self.process_inst(inst, func)?;
                }
            }
        }
        Ok(changed)
    }
}

impl<'a, 'b> LoadElim<'a, 'b> {
    pub fn new(program: &'a mut Program, memory_ssa: &'a mut MemorySSA<'b>) -> Self {
        Self {
            program,
            memory_ssa,
        }
    }

    fn process_inst(&mut self, mut inst: InstPtr, func: FunPtr) -> Result<bool> {
        // Instruction must be load (instead of function call), otherwise it can't be optimized
        if inst.get_type() != InstType::Load {
            return Ok(false);
        }

        // Get corresponding MemorySSA node
        let Some(load_node) = self.memory_ssa.get_inst_node(inst) else {
            return Ok(false);
        };

        // It should be a MemoryUse node (not entry or phi)
        let Node::Normal(_, Some(src), _, _) = load_node.as_ref() else {
            return Ok(false);
        };

        // Predict value in MemorySSA
        let predicted = self.memory_ssa.predict_read(*src, inst, func)?;

        // Replace if value can be predicted
        if let Some(predicted) = predicted {
            inst.replace_self(&predicted);
            self.memory_ssa.remove_node(load_node);
            return Ok(true);
        }
        Ok(false)
    }
}
