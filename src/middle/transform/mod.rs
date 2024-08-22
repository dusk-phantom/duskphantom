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

use std::time::Instant;

use anyhow::Result;

#[allow(unused)]
use crate::{cprintln, utils::diff::diff};

use super::Program;

pub mod block_fuse;
pub mod constant_fold;
pub mod dead_code_elim;
pub mod func_inline;
pub mod inst_combine;
pub mod ldce;
pub mod licm;
pub mod load_elim;
pub mod load_store_elim;
pub mod loop_depth;
pub mod loop_optimization;
pub mod loop_simplify;
pub mod make_parallel;
pub mod mem2reg;
pub mod redundance_elim;
pub mod sink_code;
pub mod store_elim;
pub mod ultimate_pass;

pub trait Transform {
    fn name() -> String;

    fn get_program_mut(&mut self) -> &mut Program;

    fn run(&mut self) -> Result<bool>;

    #[allow(unused)]
    fn run_and_log(&mut self) -> Result<bool> {
        let time_before = Instant::now();
        let program_before = self.get_program_mut().module.gen_llvm_ir();
        let changed = self.run()?;
        let elapsed = time_before.elapsed().as_micros();
        let program_after = self.get_program_mut().module.gen_llvm_ir();
        cprintln!(
            "## Pass {} {}\n\nTime elapsed = {} µs\n\nDiff:\n\n```diff\n{}```\n",
            Self::name(),
            if changed { "[CHANGED]" } else { "" },
            elapsed,
            diff(&program_before, &program_after)
        );
        Ok(changed)
    }
}
