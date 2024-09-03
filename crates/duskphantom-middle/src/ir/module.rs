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

use super::*;

/// one module is one file
pub struct Module {
    /// global variables in this module
    pub global_variables: Vec<GlobalPtr>,

    /// functions in this module.
    /// Make sure that the first function is `main` function.
    pub functions: Vec<FunPtr>,

    pub mem_pool: ObjPtr<IRBuilder>,
}

impl Module {
    pub fn new(mem_pool: ObjPtr<IRBuilder>) -> Self {
        Self {
            functions: Vec::new(),
            mem_pool,
            global_variables: Vec::new(),
        }
    }

    pub fn gen_llvm_ir(&self) -> String {
        let mut ir = String::new();
        for global in self.global_variables.iter() {
            ir.push_str(&global.as_ref().gen_llvm_ir());
        }
        for fun in &self.functions {
            ir.push_str(&fun.gen_llvm_ir());
        }
        ir
    }
}
