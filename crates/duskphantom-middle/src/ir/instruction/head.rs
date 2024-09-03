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
pub struct Head {
    manager: InstManager,
}

impl IRBuilder {
    pub fn new_head(&mut self) -> InstPtr {
        self.new_instruction(Box::new(Head {
            manager: InstManager::new(ValueType::Void),
        }))
    }
}

impl Instruction for Head {
    gen_common_code!(Head, Head);
    fn copy_self(&self) -> Box<dyn Instruction> {
        Box::new(Head {
            manager: InstManager::new(ValueType::Void),
        })
    }
    #[inline]
    fn gen_llvm_ir(&self) -> String {
        String::new()
    }
}

impl Display for Head {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}
