use anyhow::Result;

use crate::middle::{
    analysis::{effect_analysis::EffectAnalysis, memory_ssa::MemorySSA},
    Program,
};

use super::{load_elim, store_elim};

pub fn optimize_program(program: &mut Program) -> Result<()> {
    let effect_analysis = EffectAnalysis::new(program);
    let mut memory_ssa = MemorySSA::new(program, &effect_analysis);
    load_elim::optimize_program(program, &mut memory_ssa)?;
    store_elim::optimize_program(program, &mut memory_ssa)?;
    Ok(())
}
