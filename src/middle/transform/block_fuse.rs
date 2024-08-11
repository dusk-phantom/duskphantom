use anyhow::{Ok, Result};

use crate::middle::{
    ir::{instruction::InstType, BBPtr, FunPtr},
    Program,
};

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    BlockFuse::new(program).run()
}

struct BlockFuse<'a> {
    program: &'a mut Program,
}

impl<'a> BlockFuse<'a> {
    fn new(program: &'a mut Program) -> Self {
        Self { program }
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

    /// If block has only one predecessor, and that predecessor has only one successor,
    /// these two blocks can be fused as one.
    fn fuse_block(&mut self, mut bb: BBPtr, func: FunPtr) -> Result<bool> {
        let Some(mut pred) = bb.get_pred_bb().first().cloned() else {
            return Ok(false);
        };
        if func.entry == Some(pred) {
            return Ok(false);
        }
        if pred.get_succ_bb().len() == 1 {
            if bb.get_pred_bb().len() == 1 {
                // Last instruction is "br", move the rest to successor block
                for inst in pred.iter_rev().skip(1) {
                    bb.push_front(inst);
                }

                // Replace `pred -> bb` with `bb`
                pred.replace_entry(bb, func);

                // Remove `pred`
                pred.remove_self();
                return Ok(true);
            } else if pred.get_first_inst().get_type() == InstType::Br {
                // Replace `pred -> bb` with `bb`
                pred.replace_entry(bb, func);

                // Remove `pred`
                pred.remove_self();
                return Ok(true);
            }
        }
        Ok(false)
    }
}
