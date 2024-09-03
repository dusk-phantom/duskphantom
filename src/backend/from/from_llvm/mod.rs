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

mod builder;

mod build_inst;
#[macro_use]
mod macros;
mod build_glob_var;
mod build_op;

pub use super::irs::*;
pub use anyhow::{anyhow, Context, Result};
pub use builder::IRBuilder;
pub use clang_front_back::clang_frontend;
pub use duskphantom_utils::context;

#[cfg(feature = "clang_enabled")]
#[allow(unused)]
pub fn gen_from_clang(program: &clang_frontend::Program) -> Result<Program> {
    builder::IRBuilder::gen_from_clang(program)
}
