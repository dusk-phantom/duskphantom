use anyhow::Result;

use crate::middle::{
    analysis::{effect_analysis::EffectAnalysis, simple_gvn::SimpleGVN},
    Program,
};

use super::Transform;

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    RedundanceElim::new(program).run_and_log()
}

pub struct RedundanceElim<'a> {
    program: &'a mut Program,
}

impl<'a> Transform for RedundanceElim<'a> {
    fn name() -> String {
        "redundance_elim".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let effect_analysis = EffectAnalysis::new(self.program);
        let gvn = SimpleGVN::new(self.program, &effect_analysis);
        let mut changed = false;
        for (inst, leader) in gvn.inst_leader.iter() {
            let leader = (*leader).into();
            inst.clone().replace_self(&leader);
            changed = true;
        }
        Ok(changed)
    }
}

impl<'a> RedundanceElim<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self { program }
    }
}
