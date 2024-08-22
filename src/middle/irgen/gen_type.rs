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

use anyhow::{anyhow, Context, Result};

use crate::{
    context,
    frontend::Type,
    middle::ir::{Constant, ValueType},
};

use super::gen_const::gen_const;

/// Translate a frontend type to IR value type
pub fn gen_type(ty: &Type) -> Result<ValueType> {
    match ty {
        Type::Void => Ok(ValueType::Void),
        Type::Int => Ok(ValueType::Int),
        Type::Float => Ok(ValueType::Float),
        Type::Bool => Ok(ValueType::Bool),
        Type::Pointer(ty) => Ok(ValueType::Pointer(Box::new(gen_type(ty)?))),
        Type::Array(ty, index_expr) => {
            let index_constant = gen_const(index_expr)?;
            let Constant::Int(index) = index_constant else {
                return Err(anyhow!("index is not an integer")).with_context(|| context!());
            };
            Ok(ValueType::Array(Box::new(gen_type(ty)?), index as usize))
        }
        _ => Err(anyhow!("type {:?} can't translate to middle", ty)).with_context(|| context!()),
    }
}
