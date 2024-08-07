use anyhow::Result;

use crate::middle::{
    ir::{BBPtr, FunPtr},
    Program,
};

#[allow(unused)]
pub fn optimize_program(program: &mut Program) -> Result<()> {
    BlockFuse::new(program).run();
    Ok(())
}

struct BlockFuse<'a> {
    program: &'a mut Program,
}

impl<'a> BlockFuse<'a> {
    fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    fn run(&mut self) {
        for fun in self
            .program
            .module
            .functions
            .clone()
            .iter()
            .filter(|f| !f.is_lib())
        {
            self.fuse_function(*fun);
        }
    }

    fn fuse_function(&mut self, fun: FunPtr) {
        for bb in fun.dfs_iter() {
            self.fuse_block(bb, fun);
        }
    }

    /// If block has only one predecessor, and that predecessor has only one successor,
    /// these two blocks can be fused as one.
    fn fuse_block(&mut self, mut bb: BBPtr, mut fun: FunPtr) {
        if bb.get_pred_bb().len() == 1 {
            let pred = bb.get_pred_bb()[0];
            if pred.get_succ_bb().len() == 1 {
                // Last instruction is "br", move the rest to successor block
                for inst in pred.iter_rev().skip(1) {
                    bb.push_front(inst);
                }

                // Set `pred.pred.succ` from `pred` to `bb`
                let pred_pred = pred.get_pred_bb().clone();
                for mut pred_pred_bb in pred_pred {
                    pred_pred_bb.replace_succ_bb(pred, bb);
                }

                // If pred is function entry, change function entry
                if fun.entry == Some(pred) {
                    fun.entry = Some(bb);
                }
            }
        }
    }
}
