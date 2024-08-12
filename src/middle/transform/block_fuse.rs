use anyhow::{Ok, Result};

use crate::middle::{
    ir::{BBPtr, FunPtr},
    Program,
};

use super::Transform;

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    BlockFuse::new(program).run_and_log()
}

pub struct BlockFuse<'a> {
    program: &'a mut Program,
}

impl<'a> Transform for BlockFuse<'a> {
    fn name() -> String {
        "block_fuse".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for fun in self
            .program
            .module
            .functions
            .clone()
            .iter()
            .filter(|f| !f.is_lib())
        {
            for bb in fun.rpo_iter() {
                changed |= self.fuse_block(bb, *fun)?;
            }
        }
        Ok(changed)
    }
}

impl<'a> BlockFuse<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    /// If block has only one predecessor, and that predecessor has only one successor,
    /// these two blocks can be fused as one.
    fn fuse_block(&mut self, mut bb: BBPtr, func: FunPtr) -> Result<bool> {
        let Some(mut pred) = bb.get_pred_bb().first().cloned() else {
            return Ok(false);
        };
        if func.entry == Some(pred) {
            return Ok(false);
        }
        if pred.get_succ_bb().len() == 1 && bb.get_pred_bb().len() == 1 {
            // Last instruction is "br", move the rest to successor block
            for inst in pred.iter_rev().skip(1) {
                bb.push_front(inst);
            }

            // Replace `pred -> bb` with `bb`
            pred.replace_entry(bb, func);

            // Remove `pred`
            pred.remove_self();
            return Ok(true);
        }
        Ok(false)
    }
}
