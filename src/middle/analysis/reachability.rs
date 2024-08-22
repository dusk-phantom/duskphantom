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

use std::collections::HashSet;

use crate::middle::ir::{BBPtr, FunPtr};

pub struct Reachability {
    reachable: HashSet<BBPtr>,
}

impl Reachability {
    pub fn new(func: FunPtr) -> Self {
        let mut reachable = HashSet::new();
        for bb in func.dfs_iter() {
            reachable.insert(bb);
        }
        Self { reachable }
    }

    pub fn is_reachable(&self, bb: BBPtr) -> bool {
        self.reachable.contains(&bb)
    }
}
