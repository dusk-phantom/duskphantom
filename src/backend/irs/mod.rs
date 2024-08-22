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

pub mod block;

pub mod func;
// pub mod inst;
#[macro_use]
mod macros;

pub mod checker;
pub mod instruction;
pub mod module;
pub mod operand;
pub mod prog;
pub mod stack_slot;
pub mod var;

pub use super::*;
pub use block::*;
pub use func::*;
pub use instruction::*;
pub use module::*;
pub use operand::*;
pub use prog::*;
pub use stack_slot::*;
pub use var::*;
