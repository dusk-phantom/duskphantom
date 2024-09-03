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

use anyhow::{Ok, Result};

use crate::{
    ir::{BBPtr, FunPtr},
    Program,
};

use super::Transform;

pub fn optimize_program(program: &mut Program) -> Result<bool> {
    BlockFuse::new(program).run_and_log()
}

pub struct BlockFuse<'a> {
    program: &'a mut Program,
}

impl<'a> Transform for BlockFuse<'a> {
    fn get_program_mut(&mut self) -> &mut Program {
        self.program
    }

    fn name() -> String {
        "block_fuse".to_string()
    }

    fn run(&mut self) -> Result<bool> {
        let mut changed = false;
        for func in self.program.module.functions.clone() {
            if func.is_lib() {
                continue;
            }
            for bb in func.rpo_iter() {
                changed |= self.fuse_block(bb, func)?;
            }
        }
        Ok(changed)
    }
}

impl<'a> BlockFuse<'a> {
    pub fn new(program: &'a mut Program) -> Self {
        Self { program }
    }

    /// If block has only one predecessor, and that predecessor has only one successor,
    /// these two blocks can be fused as one.
    fn fuse_block(&mut self, mut bb: BBPtr, func: FunPtr) -> Result<bool> {
        let Some(mut pred) = bb.get_pred_bb().first().cloned() else {
            return Ok(false);
        };
        if pred.get_succ_bb().len() == 1 && bb.get_pred_bb().len() == 1 {
            // Last instruction is "br", move the rest to successor block
            for inst in pred.iter_rev().skip(1) {
                bb.push_front(inst);
            }

            // Replace `pred -> bb` with `bb`
            // TODO remove requirement of func in replace_entry
            pred.replace_entry(bb, func);

            // Remove `pred`
            pred.remove_self();
            return Ok(true);
        }
        Ok(false)
    }
}
