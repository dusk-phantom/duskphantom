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

use duskphantom_frontend as frontend;
use anyhow::Result;
use duskphantom_utils::frame_map::FrameMap;
use program_kit::ProgramKit;

mod function_kit;
mod gen_binary;
mod gen_const;
mod gen_expr;
mod gen_global_decl;
mod gen_impl;
mod gen_inner_decl;
mod gen_library_function;
mod gen_stmt;
mod gen_type;
mod gen_unary;
mod program_kit;
mod value;

/// Generate middle IR from a frontend AST
pub fn gen(program: &frontend::Program) -> Result<crate::Program> {
    let mut result = crate::Program::new();
    ProgramKit {
        program: &mut result,
        env: FrameMap::new(),
        fun_env: FrameMap::new(),
    }
    .gen(program)?;
    Ok(result)
}
