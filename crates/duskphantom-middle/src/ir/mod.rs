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

pub mod basic_block;
pub mod function;
pub mod instruction;
#[macro_use]
mod macros;
pub mod constant;
pub mod global_variable;
pub mod ir_builder;
pub mod module;
pub mod operand;
pub mod value_type;

pub use self::basic_block::{BBPtr, BasicBlock};
pub use self::function::{FunPtr, Function, ParaPtr, Parameter};
pub use self::instruction::{InstPtr, Instruction};
pub use self::module::Module;
pub use constant::Constant;
pub use global_variable::{GlobalPtr, GlobalVariable};
pub use ir_builder::IRBuilder;
pub use operand::Operand;
pub use value_type::ValueType;

pub use duskphantom_utils::mem::{ObjPool, ObjPtr};
use std::collections::{HashSet, VecDeque};
use std::fmt::Display;
