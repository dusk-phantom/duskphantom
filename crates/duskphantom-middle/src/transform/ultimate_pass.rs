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

use anyhow::Result;

use crate::{config::CONFIG, Program};

use super::{
    block_fuse, dead_code_elim, func_inline, inst_combine, load_store_elim, loop_optimization,
    make_parallel, mem2reg, redundance_elim, sink_code,
};

#[allow(unused)]
pub fn optimize_program(program: &mut Program, level: usize) -> Result<bool> {
    mem2reg::optimize_program(program)?;
    main_loop(program)?;
    if CONFIG.open_auto_parallel {
        make_parallel::optimize_program::<5>(program)?;
    }
    eval_and_prune(program)?;
    sink_code::optimize_program(program)?;
    Ok(true)
}

pub fn main_loop(program: &mut Program) -> Result<bool> {
    loop {
        let mut changed = false;

        // Inline functions
        changed |= func_inline::optimize_program(program)?;

        // Simplify code
        changed |= eval_and_prune(program)?;

        // Remove redundancy
        changed |= redundance_elim::optimize_program(program)?;

        // Optimize loop
        // TODO add changed and timing info for this pass
        // TODO remove inst_combine in loop_optimization
        loop_optimization::optimize_program(program)?;

        // Fuse blocks
        changed |= block_fuse::optimize_program(program)?;

        // Break if unchanged
        if !changed {
            break;
        }
    }
    Ok(true)
}

pub fn eval_and_prune(program: &mut Program) -> Result<bool> {
    let mut changed = false;
    loop {
        let mut c = false;
        c |= inst_combine::optimize_program(program)?;
        c |= load_store_elim::optimize_program(program)?;
        c |= dead_code_elim::optimize_program(program)?;
        changed |= c;
        if !c {
            break;
        }
    }
    Ok(changed)
}
