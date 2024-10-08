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

use crate::config::CONFIG;
use duskphantom_utils::fprintln;

use super::irs::*;
use std::collections::{HashMap, HashSet, VecDeque};

pub mod analysis;
#[allow(unused)]
pub mod inst_combine;

pub mod pre_inst_split;

pub mod post_inst_split;

pub mod schedule;

pub mod reg_alloc;

pub mod caller_callee;

/// 块相关的优化
pub mod block;

/// 栈相关的优化
pub mod stack;

pub fn optimize(program: &mut prog::Program) -> Result<()> {
    #[cfg(feature = "backend_opt")]
    {
        for m in program.modules.iter_mut() {
            if CONFIG.num_parallel_for_func_gen_asm <= 1 {
                println!("num_parallel_for_func_gen_asm <= 1,run in single thread");
                m.funcs.iter_mut().try_for_each(optimize_func)?;
            } else {
                let thread_pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(CONFIG.num_parallel_for_func_gen_asm)
                    .build()
                    .unwrap();
                thread_pool.install(|| m.funcs.par_iter_mut().try_for_each(optimize_func))?;
            }
        }
    }
    #[cfg(not(feature = "backend_opt"))]
    {
        phisicalize::phisicalize(program); // 直接物理化
    }
    Ok(())
}

#[allow(unused)]
fn test_symplify_and_desimplify_term(func: &mut Func) {
    fprintln!("2.s", "{}", func.gen_asm());
    for i in 0..1000 {
        func.simplify_term();
        fprintln!("2.s", "{}", func.gen_asm());
        func.desimplify_term();
        fprintln!("2.s", "{}", func.gen_asm());
    }
}

#[allow(unused)]
pub fn optimize_func(func: &mut Func) -> Result<()> {
    block::handle_block_simplify(func)?;

    // inst combine? 匹配一些模式,将多条指令合并成一条
    fprintln!("log/before_inst_combine.s", "{}", func.gen_asm());
    inst_combine::handle_inst_combine(func)?;

    // inst split? 将一条指令拆分成多条
    pre_inst_split::handle_mul_div_opt(func)?;

    phisicalize::handle_illegal_inst(func)?;

    phisicalize::handle_long_jump(func, &REG_T0, 20_0000);

    fprintln!("log/after_inst_scheduling.s", "{}", func.gen_asm());
    // register allocation
    reg_alloc::handle_reg_alloc(func)?;
    fprintln!("log/after_reg_alloc.s", "{}", func.gen_asm());

    // processing caller-save and callee-save
    caller_callee::handle_caller_callee(func)?;

    // processing stack frame's opening and closing
    stack::handle_stack(func)?;

    // inst scheduling
    schedule::handle_inst_scheduling(func)?;

    func.simplify_term();
    Ok(())
}
