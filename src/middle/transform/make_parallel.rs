use std::collections::HashMap;

use anyhow::Result;

use crate::middle::{
    analysis::loop_tools::{LoopForest, LoopPtr},
    ir::InstPtr,
    Program,
};

use super::Transform;

pub fn optimize_program<'a>(
    program: &'a mut Program,
    loop_forest: &'a mut LoopForest,
) -> Result<bool> {
    MakeParallel::new(program, loop_forest).run_and_log()
}

pub struct MakeParallel<'a> {
    program: &'a mut Program,
    loop_forest: &'a mut LoopForest,
    has_return: HashMap<LoopPtr, bool>,
}

impl<'a> Transform for MakeParallel<'a> {
    fn get_program_mut(&mut self) -> &mut Program {
        self.program
    }

    fn name() -> String {
        "make_parallel".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for lo in self.loop_forest.forest.clone() {
            let mut candidate = Vec::new();
            self.make_candidate(&mut candidate, lo)?;
            for c in candidate {
                changed |= self.make_parallel(c)?;
            }
        }
        Ok(changed)
    }
}

impl<'a> MakeParallel<'a> {
    pub fn new(program: &'a mut Program, loop_forest: &'a mut LoopForest) -> Self {
        Self {
            program,
            loop_forest,
            has_return: HashMap::new(),
        }
    }

    fn check_has_return(&mut self, lo: LoopPtr) -> bool {
        todo!("check if lo has `exit` as succ, store to `has_return`");
        for lo in lo.sub_loops.iter() {
            if self.check_has_return(*lo) {
                return true;
            }
        }
        return false;
    }

    fn make_candidate(&mut self, result: &mut Vec<ParallelCandidate>, lo: LoopPtr) -> Result<()> {
        todo!("get candidate info (unique exit, indvar) from current loop, check if body can be parallelized");
        for lo in lo.sub_loops.iter() {
            self.make_candidate(result, *lo)?;
        }
        Ok(())
    }

    fn make_parallel(&mut self, candidate: ParallelCandidate) -> Result<bool> {
        todo!("create parallized exit and indvar, join threads");
    }
}

/// A candidate for parallelization.
/// For example:
///
/// ```c
/// int i = 2;
/// while (i < 6) {
///     // body
///     i += 2;
/// }
/// ```
///
/// exit = br (indvar < 6), loop, exit
/// indvar = phi [2, pre_header], [indvar + 2, loop]
struct ParallelCandidate {
    lo: LoopPtr,
    exit: InstPtr,
    indvar: InstPtr,
}
