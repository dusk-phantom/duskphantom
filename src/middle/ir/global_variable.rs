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

pub type GlobalPtr = ObjPtr<GlobalVariable>;
impl Display for GlobalPtr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.name)
    }
}

pub struct GlobalVariable {
    pub name: String,
    pub value_type: ValueType,
    /// True if the global variable is a global variable, false if it is a global constant.
    pub variable_or_constant: bool,
    pub initializer: Constant,
    user: Vec<InstPtr>,
}

impl Display for GlobalVariable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "@{}", self.name)
    }
}

impl GlobalVariable {
    pub fn new(
        name: String,
        value_type: ValueType,
        variable_or_constant: bool,
        initializer: Constant,
    ) -> Self {
        Self {
            name,
            value_type,
            variable_or_constant,
            initializer,
            user: Vec::new(),
        }
    }

    pub fn gen_llvm_ir(&self) -> String {
        format!(
            "{} = dso_local {} {} {}\n",
            self,
            if self.variable_or_constant {
                "global"
            } else {
                "constant"
            },
            self.value_type,
            self.initializer,
        )
    }

    pub fn get_user(&self) -> &[InstPtr] {
        &self.user
    }
    pub fn get_user_mut(&mut self) -> &mut Vec<InstPtr> {
        &mut self.user
    }
    /// # Safety
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn add_user(&mut self, inst: InstPtr) {
        self.user.push(inst);
    }
    /// # Safety
    /// FIXME: explain why it is unsafe,and describe the safety requirements
    pub unsafe fn remove_user(&mut self, inst: InstPtr) {
        self.user
            .iter()
            .position(|x| *x == inst)
            .map(|x| self.user.swap_remove(x));
    }
}
