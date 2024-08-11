use anyhow::Result;

use crate::middle::{
    analysis::{effect_analysis::EffectAnalysis, simple_gvn::SimpleGVN},
    Program,
};

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    let gvn = SimpleGVN::new(program, &effect_analysis);
    let mut changed = false;
    for (inst, leader) in gvn.inst_leader.iter() {
        let leader = (*leader).into();
        inst.clone().replace_self(&leader);
        changed = true;
    }
    Ok(changed)
}
