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

use crate::context;
use crate::frontend::Expr;
use crate::middle::ir::Constant;
use anyhow::{anyhow, Context, Result};

use super::gen_type::gen_type;

/// Generate constant expression
pub fn gen_const(expr: &Expr) -> Result<Constant> {
    match expr {
        Expr::Array(ls) => Ok(Constant::Array(
            ls.iter().map(gen_const).collect::<anyhow::Result<_, _>>()?,
        )),
        Expr::Zero(ty) => Ok(Constant::Zero(gen_type(ty)?)),
        Expr::Int(x) => Ok(Constant::Int(*x)),
        Expr::Float(x) => Ok(Constant::Float(*x)),
        Expr::String(str) => {
            let mut vec = vec![];

            // Add trailing zero to bytes, pad bytes to multiple of 4
            let mut bytes = str.as_bytes().to_vec();
            bytes.push(0);
            while bytes.len() % 4 != 0 {
                bytes.push(0);
            }

            // Convert to little indian
            for i in 0..(bytes.len() / 4) {
                let mut val: u32 = bytes[i * 4 + 3] as u32;
                val = val * 256 + bytes[i * 4 + 2] as u32;
                val = val * 256 + bytes[i * 4 + 1] as u32;
                val = val * 256 + bytes[i * 4] as u32;
                vec.push(Constant::Int(val as i32));
            }
            Ok(Constant::Array(vec))
        }
        _ => Err(anyhow!("expression {:?} is not constant", expr)).with_context(|| context!()),
    }
}
