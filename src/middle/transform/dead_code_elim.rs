use anyhow::Result;

use crate::middle::analysis::effect_analysis::EffectAnalysis;
use crate::middle::ir::instruction::InstType;
use crate::middle::ir::{InstPtr, Operand};
use crate::middle::Program;

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    DeadCodeElim::new(program, &effect_analysis).run()
}

struct DeadCodeElim<'a> {
    program: &'a mut Program,
    effect_analysis: &'a EffectAnalysis,
}

impl<'a> DeadCodeElim<'a> {
    fn new(program: &'a mut Program, effect_analysis: &'a EffectAnalysis) -> Self {
        Self {
            program,
            effect_analysis,
        }
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
