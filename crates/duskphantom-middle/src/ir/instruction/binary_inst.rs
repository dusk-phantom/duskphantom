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
use crate::impl_binary_inst;
use ValueType::{Bool, Float, Int};

/// impl for binary operation and bitwise binary_inst
pub trait BinaryInst {
    fn get_lhs(&self) -> &Operand;
    fn set_lhs(&mut self, lhs: Operand);
    fn get_rhs(&self) -> &Operand;
    fn set_rhs(&mut self, rhs: Operand);
}

impl_binary_inst!(Add, "i32", get_add, lhs, rhs, Int);
impl_binary_inst!(FAdd, "float", get_fadd, lhs, rhs, Float);
impl_binary_inst!(Sub, "i32", get_sub, lhs, rhs, Int);
impl_binary_inst!(FSub, "float", get_fsub, lhs, rhs, Float);
impl_binary_inst!(Mul, "i32", get_mul, lhs, rhs, Int);
impl_binary_inst!(FMul, "float", get_fmul, lhs, rhs, Float);
impl_binary_inst!(UDiv, "i32", get_udiv, lhs, rhs, Int);
impl_binary_inst!(SDiv, "i32", get_sdiv, lhs, rhs, Int);
impl_binary_inst!(FDiv, "float", get_fdiv, lhs, rhs, Float);
impl_binary_inst!(URem, "i32", get_urem, lhs, rhs, Int);
impl_binary_inst!(SRem, "i32", get_srem, lhs, rhs, Int);
impl_binary_inst!(Shl, "i32", get_shl, value, shiftamt, Int);
impl_binary_inst!(LShr, "i32", get_lshr, value, shiftamt, Int);
impl_binary_inst!(AShr, "i32", get_ashr, value, shiftamt, Int);
impl_binary_inst!(And, "i1", get_and, lhs, rhs, Bool);
impl_binary_inst!(Or, "i1", get_or, lhs, rhs, Bool);
impl_binary_inst!(Xor, "i1", get_xor, lhs, rhs, Bool);
