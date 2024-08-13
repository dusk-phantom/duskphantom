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
        changed |= redundance_elim::optimize_program(program)?;

        // Remove unused code
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
