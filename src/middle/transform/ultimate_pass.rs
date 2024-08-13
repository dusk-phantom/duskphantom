use anyhow::Result;

use crate::middle::Program;

use super::{
    block_fuse, dead_code_elim, func_inline, load_store_elim, mem2reg, redundance_elim,
    symbolic_eval,
};

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    mem2reg::optimize_program(program)?;
    loop {
        let mut changed = false;

        // Inline functions
        changed |= func_inline::optimize_program(program)?;

        // Weaken instructions
        changed |= symbolic_eval::optimize_program(program)?;

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
    Ok(true)
}