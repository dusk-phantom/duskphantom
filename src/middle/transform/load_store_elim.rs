use anyhow::Result;

use crate::middle::{
    analysis::{effect_analysis::EffectAnalysis, memory_ssa::MemorySSA},
    Program,
};

use super::{load_elim, store_elim};

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    let mut memory_ssa = MemorySSA::new(program, &effect_analysis);
    let mut changed = false;

    // Eliminate predictable load first, and then eliminate unused store
    changed |= load_elim::optimize_program(program, &mut memory_ssa)?;
    changed |= store_elim::optimize_program(program, &mut memory_ssa)?;
    Ok(changed)
}
