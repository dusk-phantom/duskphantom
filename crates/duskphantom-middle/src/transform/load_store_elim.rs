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

use crate::{
    analysis::{effect_analysis::EffectAnalysis, memory_ssa::MemorySSA},
    Program,
};

use super::{load_elim, store_elim};

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    let effect_analysis = EffectAnalysis::new(program);
    let mut memory_ssa = MemorySSA::new(program, &effect_analysis);
    let mut changed = false;

    // Eliminate predictable load first, and then eliminate unused store
    changed |= load_elim::optimize_program(program, &mut memory_ssa)?;
    changed |= store_elim::optimize_program(program, &mut memory_ssa)?;
    Ok(changed)
}
