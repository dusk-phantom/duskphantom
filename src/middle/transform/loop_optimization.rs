use std::collections::{HashMap, VecDeque};

use super::*;
use crate::middle::{
    analysis::{
        effect_analysis::EffectAnalysis,
        loop_tools::{self, LoopForest, LoopPtr},
    },
    ir::FunPtr,
    transform::symbolic_eval,
    Program,
};
use anyhow::{Ok, Result};

pub fn optimize_program(program: &mut Program) -> Result<()> {
    let effect_analysis = EffectAnalysis::new(program);
    let mut func_loop_map = program
        .module
        .functions
        .iter_mut()
        .filter_map(|func| loop_tools::LoopForest::make_forest(*func).map(|forest| (*func, forest)))
        .collect::<HashMap<FunPtr, LoopForest>>();

    for (_, forest) in func_loop_map.iter_mut() {
        loop_simplify::LoopSimplifier::new(&mut program.mem_pool).run(forest)?;
        licm::LICM::new(&mut program.mem_pool).run(forest)?;
        ldce::LDCE::new(&mut program.mem_pool, &effect_analysis).run(forest)?;
    }
    symbolic_eval::optimize_program(program)?;

    Ok(())
}

pub fn loop_forest_post_order<F>(loop_forest: &mut LoopForest, mut f: F) -> Result<()>
where
    F: FnMut(LoopPtr) -> Result<()>,
{
    let mut stack = Vec::new();
    let mut queue = VecDeque::from(loop_forest.forest.clone());
    while let Some(lo) = queue.pop_front() {
        queue.extend(lo.sub_loops.iter());
        stack.push(lo);
    }

    while let Some(lo) = stack.pop() {
        f(lo)?;
    }

    Ok(())
}
