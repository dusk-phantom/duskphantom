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

// 一个program是一个程序, 可能由多个 module组成
pub struct Program {
    /// optional entry module name, to specify if this program is a library or executable
    pub entry: Option<String>,
    pub modules: Vec<module::Module>,
}

impl Program {
    pub fn entry(&self) -> Option<&module::Module> {
        if let Some(entry) = self.entry.as_ref() {
            for module in self.modules.iter() {
                if module.name() == entry.as_str() {
                    return Some(module);
                }
            }
        }
        None
    }
    pub fn gen_asm(&self) -> String {
        #[cfg(not(feature = "gen_virtual_asm"))]
        assert!(backend::irs::checker::Riscv.check_prog(&program));
        // Note: only consider single module program now
        let mut asm = String::with_capacity(1024 * 1024);
        for module in self.modules.iter() {
            asm.push_str(module.gen_asm().as_str());
        }
        asm
    }
}
