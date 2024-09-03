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

use crate::ir::BBPtr;
use crate::ir::FunPtr;
use std::collections::{HashMap, HashSet};

pub mod alias_analysis;
pub mod call_graph;
pub mod dominator_tree;
pub mod effect_analysis;
pub mod loop_tools;
pub mod memory_ssa;
pub mod reachability;
pub mod simple_gvn;
