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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Operand {
    Constant(Constant),
    Global(GlobalPtr),
    Parameter(ParaPtr),
    Instruction(InstPtr),
}

impl Display for Operand {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Operand::Constant(c) => write!(f, "{}", c),
            Operand::Global(g) => write!(f, "{}", g),
            Operand::Parameter(p) => write!(f, "{}", p),
            Operand::Instruction(inst) => write!(f, "{}", inst),
        }
    }
}

impl Operand {
    pub fn get_type(&self) -> ValueType {
        match self {
            Operand::Constant(c) => c.get_type(),
            // Type of global var identifier (@gvar) is pointer
            Operand::Global(g) => ValueType::Pointer(g.as_ref().value_type.clone().into()),
            Operand::Parameter(p) => p.as_ref().value_type.clone(),
            Operand::Instruction(inst) => inst.get_value_type(),
        }
    }
}

impl From<Constant> for Operand {
    fn from(c: Constant) -> Self {
        Self::Constant(c)
    }
}

impl From<InstPtr> for Operand {
    fn from(inst: InstPtr) -> Self {
        Self::Instruction(inst)
    }
}

impl From<ParaPtr> for Operand {
    fn from(param: ParaPtr) -> Self {
        Self::Parameter(param)
    }
}

impl From<GlobalPtr> for Operand {
    fn from(gvar: GlobalPtr) -> Self {
        Self::Global(gvar)
    }
}

impl Operand {
    pub fn is_const(&self) -> bool {
        matches!(self, Operand::Constant(_))
    }
}
