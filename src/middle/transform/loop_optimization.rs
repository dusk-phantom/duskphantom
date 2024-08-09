use std::collections::HashMap;

use super::*;
use crate::middle::{
    analysis::loop_tools::{self, LoopForest},
    ir::FunPtr,
    transform::inst_combine,
    Program,
};
use anyhow::Result;
pub fn optimize_program(program: &mut Program) -> Result<()> {
    let mut func_loop_map = program
        .module
        .functions
        .iter_mut()
        .filter_map(|func| loop_tools::LoopForest::make_forest(*func).map(|forest| (*func, forest)))
        .collect::<HashMap<FunPtr, LoopForest>>();

    let mut simplifier = loop_simplify::LoopSimplifier::new(&mut program.mem_pool);
    for (_, forest) in func_loop_map.iter_mut() {
        simplifier.loop_simplify(forest)?;
    }
    inst_combine::optimize_program(program)?;

    Ok(())
}
