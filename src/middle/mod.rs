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

use crate::{/* errors::MiddleError, */ frontend};
use anyhow::Context;
use duskphantom_utils::mem::ObjPtr;
use ir::ir_builder::IRBuilder;
use transform::ultimate_pass;

pub mod analysis;
pub mod ir;
pub mod irgen;
pub mod transform;

use std::pin::Pin;

pub struct Program {
    pub module: ir::Module,
    pub mem_pool: Pin<Box<IRBuilder>>,
}

use crate::context;
use anyhow::Result;

pub fn gen(program: &frontend::Program) -> Result<Program> {
    irgen::gen(program).with_context(|| context!())
    // match irgen::gen(program) {
    //     Ok(program) => Ok(program),
    //     Err(_) => Err(MiddleError::GenError),
    // }
}

pub fn optimize(program: &mut Program) {
    ultimate_pass::optimize_program(program).unwrap();
}

impl Default for Program {
    fn default() -> Self {
        Self::new()
    }
}

impl Program {
    pub fn new() -> Self {
        let program_mem_pool = Box::pin(IRBuilder::new());
        let mem_pool: ObjPtr<IRBuilder> = ObjPtr::new(&program_mem_pool);
        Self {
            mem_pool: program_mem_pool,
            module: ir::Module::new(mem_pool),
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        self.mem_pool.clear();
    }
}
