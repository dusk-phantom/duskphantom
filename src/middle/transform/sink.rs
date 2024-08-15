use std::collections::HashSet;

use anyhow::Result;

use crate::middle::analysis::effect_analysis::EffectAnalysis;
use crate::middle::ir::instruction::InstType;
use crate::middle::ir::{InstPtr, Operand};
use crate::middle::Program;

use super::Transform;

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    Sink::new(program, &effect_analysis).run_and_log()
}

pub struct Sink<'a> {
    program: &'a mut Program,
    effect_analysis: &'a EffectAnalysis,
    visited: HashSet<InstPtr>,
}

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

impl<'a> Sink<'a> {
    pub fn new(program: &'a mut Program, effect_analysis: &'a EffectAnalysis) -> Self {
        Self {
            program,
            effect_analysis,
            visited: HashSet::new(),
        }
    }

    fn sink_inst(&mut self, inst: InstPtr) -> Result<bool> {
        Ok(true)
    }

    fn has_side_effect(&mut self, inst: InstPtr) -> bool {
        matches!(
            inst.get_type(),
            InstType::Store | InstType::Ret | InstType::Br
        ) || self.effect_analysis.has_effect(inst)
    }
}
