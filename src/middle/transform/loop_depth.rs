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
