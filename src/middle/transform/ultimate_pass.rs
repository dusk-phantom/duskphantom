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

        // Weaken instructions
        changed |= inst_combine::optimize_program(program)?;

        // Optimize loop
        // TODO add changed and timing info for this pass
        // TODO remove inst_combine in loop_optimization
        // TODO fix uni backedge preds
        // loop_optimization::optimize_program(program)?;

        // Remove redundancy for load_store_elim
        // TODO this is O(n^2) and consumes 1s+ in long_line case,
        // can we risk checking equality with hash code only?
        changed |= redundance_elim::optimize_program(program)?;

        // Remove unused code
        // TODO time complexity will increase if there's consecutive load that requires symbolic eval:
        //
        // ```
        // %1 = load (gep %ptr, 0)
        // %2 = load (gep %ptr, (%1 + 0))
        // %3 = load (gep %ptr, (%2 + 0))
        // ```
        //
        // If we loop load_elim, we can only handle the case without `+ 0`,
        // we should put load elim inside symbolic eval, and utilize GVN for alias analysis
        changed |= load_store_elim::optimize_program(program)?;
        changed |= dead_code_elim::optimize_program(program)?;

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
