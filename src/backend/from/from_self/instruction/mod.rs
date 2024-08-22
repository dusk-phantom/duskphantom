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

mod call;
mod gep;
mod normal;

use std::collections::HashMap;

pub use super::*;

pub use builder::IRBuilder;

pub use crate::utils::mem::ObjPtr;

pub use crate::{backend::*, ssa2tac_three_float, ssa2tac_three_usual_Itype};
pub use crate::{context, middle};

pub use crate::middle::ir::instruction::binary_inst::BinaryInst;
pub use crate::middle::ir::instruction::downcast_ref;
pub use crate::middle::ir::Instruction;
pub use anyhow::{Context, Result};
pub use var::FloatVar;
