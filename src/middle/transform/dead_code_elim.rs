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

use crate::middle::analysis::effect_analysis::EffectAnalysis;
use crate::middle::ir::instruction::InstType;
use crate::middle::ir::{InstPtr, Operand};
use crate::middle::Program;

use super::Transform;

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    DeadCodeElim::new(program, &effect_analysis).run_and_log()
}

pub struct DeadCodeElim<'a> {
    program: &'a mut Program,
    effect_analysis: &'a EffectAnalysis,
}

impl<'a> Transform for DeadCodeElim<'a> {
    fn get_program_mut(&mut self) -> &mut Program {
        self.program
    }

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
                    changed |= self.dead_code_elim_inst(inst)?;
                }
            }
        }

        // Global variable does not require revisit, remove unused variables at the end
        let len0 = self.program.module.global_variables.len();
        self.program
            .module
            .global_variables
            .retain(|var| !var.get_user().is_empty());
        let len1 = self.program.module.global_variables.len();
        changed |= len0 != len1;
        Ok(changed)
    }
}

impl<'a> DeadCodeElim<'a> {
    pub fn new(program: &'a mut Program, effect_analysis: &'a EffectAnalysis) -> Self {
        Self {
            program,
            effect_analysis,
        }
    }

    fn dead_code_elim_inst(&mut self, mut inst: InstPtr) -> Result<bool> {
        if !inst.get_user().is_empty() || self.has_side_effect(inst) {
            return Ok(false);
        }
        let operands: Vec<_> = inst.get_operand().into();
        inst.remove_self();
        for op in operands {
            if let Operand::Instruction(inst) = op {
                self.dead_code_elim_inst(inst)?;
            }
        }
        Ok(true)
    }

    fn has_side_effect(&mut self, inst: InstPtr) -> bool {
        matches!(
            inst.get_type(),
            InstType::Store | InstType::Ret | InstType::Br
        ) || self.effect_analysis.has_effect(inst)
    }
}
