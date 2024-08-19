use crate::middle::analysis::loop_tools::{LoopForest, LoopTree};

use anyhow::{Ok, Result};

#[derive(Default)]
pub struct LoopDepthTracer {}

impl LoopDepthTracer {
    pub fn run(loop_forest: &LoopForest) -> Result<()> {
        for loop_tree in loop_forest.forest.iter() {
            Self::run_a_loop(1, loop_tree)?;
        }
        Ok(())
    }

    fn run_a_loop(depth: usize, loop_tree: &LoopTree) -> Result<()> {
        for mut bb in loop_tree.blocks.iter().cloned() {
            bb.depth = depth;
        }

        for sub_loop in loop_tree.sub_loops.iter() {
            Self::run_a_loop(depth + 1, sub_loop)?;
        }
        Ok(())
    }
}
