// Copyright 2024 Duskphantom Authors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.
//
// SPDX-License-Identifier: Apache-2.0

use std::collections::{HashMap, VecDeque};

use super::*;
use crate::{
    analysis::{
        effect_analysis::EffectAnalysis,
        loop_tools::{self, LoopForest, LoopPtr},
        memory_ssa::MemorySSA,
    },
    ir::FunPtr,
    Program,
};
use anyhow::{Ok, Result};

pub fn optimize_program(program: &mut Program) -> Result<()> {
    let effect_analysis = EffectAnalysis::new(program);
    let mut memory_ssa = MemorySSA::new(program, &effect_analysis);
    let mut func_loop_map = program
        .module
        .functions
        .iter_mut()
        .filter_map(|func| loop_tools::LoopForest::make_forest(*func).map(|forest| (*func, forest)))
        .collect::<HashMap<FunPtr, LoopForest>>();

    for (_, forest) in func_loop_map.iter_mut() {
        loop_simplify::LoopSimplifier::new(&mut program.mem_pool).run(forest)?;
        licm::LICM::new(&mut program.mem_pool, &mut memory_ssa).run(forest)?;
        ldce::LDCE::new(&mut program.mem_pool, &effect_analysis).run(forest)?;
        loop_depth::LoopDepthTracer::run(forest)?;
    }

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
