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

pub use anyhow::Result;

mod builder;
mod instruction;
mod operand;
mod utils;
mod vars;

#[macro_use]
mod macros;

pub use builder::*;
#[allow(unused)]
pub use instruction::*;
#[allow(unused)]
pub use operand::*;

pub use super::irs::*;

pub use duskphantom_utils::context;
pub use duskphantom_utils::fprintln;
pub use duskphantom_utils::mem;
pub use duskphantom_utils::mem::ObjPtr;

/// 中端层面，地址是唯一的
/// 因此我可以将地址作为 id
/// 用在 parameter 和 instruction 上
type Address = usize;

#[allow(unused)]
pub fn gen_from_self(program: &middle::Program) -> Result<Program> {
    builder::IRBuilder::gen_from_self(program)
}
