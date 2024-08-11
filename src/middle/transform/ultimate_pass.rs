use anyhow::Result;

use crate::middle::Program;

use super::{
    block_fuse, dead_code_elim, func_inline, inst_combine, load_store_elim, mem2reg,
    redundance_elim, unreachable_block_elim,
};

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    mem2reg::optimize_program(program)?;
    loop {
        // Inline functions
        let mut changed = false;
        changed |= func_inline::optimize_program(program)?;

        // Weaken instructions
        changed |= inst_combine::optimize_program(program)?;
        changed |= redundance_elim::optimize_program(program)?;

        // Remove unused code
        changed |= load_store_elim::optimize_program(program)?;
        changed |= dead_code_elim::optimize_program(program)?;

        // Remove unreachable block and instruction
        changed |= unreachable_block_elim::optimize_program(program)?;
        changed |= block_fuse::optimize_program(program)?;

        // Break if unchanged
        if !changed {
            break;
        }
    }
    Ok(true)
}
