use anyhow::Result;

use crate::middle::Program;

use super::{
    block_fuse, dead_code_elim, func_inline, inst_combine,
    load_store_elim, /* loop_optimization, */
    mem2reg, redundance_elim, sink_code,
};

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    mem2reg::optimize_program(program)?;
    loop {
        let mut changed = false;

        // Inline functions
        changed |= func_inline::optimize_program(program)?;

        // Simplify code
        loop {
            let mut c = false;
            c |= inst_combine::optimize_program(program)?;
            c |= load_store_elim::optimize_program(program)?;
            c |= dead_code_elim::optimize_program(program)?;
            changed |= c;
            if !c {
                break;
            }
        }

        // Optimize loop
        // TODO add changed and timing info for this pass
        // TODO remove inst_combine in loop_optimization
        // loop_optimization::optimize_program(program)?;

        // Remove redundancy for load_store_elim
        changed |= redundance_elim::optimize_program(program)?;

        // Fuse blocks
        changed |= block_fuse::optimize_program(program)?;

        // Break if unchanged
        if !changed {
            break;
        }
    }
    sink_code::optimize_program(program)?;
    Ok(true)
}
