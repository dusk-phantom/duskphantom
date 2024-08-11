use anyhow::Result;

use crate::middle::analysis::effect_analysis::EffectAnalysis;
use crate::middle::ir::instruction::InstType;
use crate::middle::ir::{BBPtr, FunPtr, InstPtr, Operand};
use crate::middle::Program;

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<()> {
    let effect_analysis = EffectAnalysis::new(program);
    DeadCodeElim::new(program, &effect_analysis).dead_code_elim();
    Ok(())
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

    fn dead_code_elim(&mut self) {
        self.program
            .module
            .functions
            .clone()
            .iter()
            .filter(|f| !f.is_lib())
            .for_each(|f| self.dead_code_elim_func(f));

        // Global variable does not require revisit, remove unused variables at the end
        self.program
            .module
            .global_variables
            .retain(|x| !x.get_user().is_empty());
    }

    fn dead_code_elim_func(&mut self, func: &FunPtr) {
        // Use post order traversal to reduce revisits
        func.po_iter().for_each(|bb| self.dead_code_elim_block(bb));
    }

    fn dead_code_elim_block(&mut self, bb: BBPtr) {
        // Iterate forward so that next instruction is always valid
        bb.iter().for_each(|inst| self.dead_code_elim_inst(inst));
    }

    fn dead_code_elim_inst(&mut self, mut inst: InstPtr) {
        if !inst.get_user().is_empty() || self.has_side_effect(inst) {
            return;
        }
        let operands: Vec<_> = inst.get_operand().into();
        inst.remove_self();
        for op in operands {
            if let Operand::Instruction(inst) = op {
                self.dead_code_elim_inst(inst);
            }
        }
    }

    fn has_side_effect(&mut self, inst: InstPtr) -> bool {
        matches!(
            inst.get_type(),
            InstType::Store | InstType::Ret | InstType::Br
        ) || self.effect_analysis.has_effect(inst)
    }
}
