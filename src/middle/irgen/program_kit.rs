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

use crate::middle::ir::FunPtr;
use crate::middle::irgen::value::Value;
use crate::utils::frame_map::FrameMap;
use crate::{frontend, middle};

/// Kit for translating a program to middle IR
pub struct ProgramKit<'a> {
    pub env: FrameMap<'a, String, Value>,
    pub fun_env: FrameMap<'a, String, FunPtr>,
    pub program: &'a mut middle::Program,
}

/// A program kit (top level) can generate declarations
impl<'a> ProgramKit<'a> {
    pub fn gen(mut self, program: &frontend::Program) -> Result<()> {
        self.gen_library_function();
        for decl in program.module.iter() {
            self.gen_global_decl(decl)?;
        }
        for decl in program.module.iter() {
            self.gen_impl(decl)?;
        }
        Ok(())
    }
}
