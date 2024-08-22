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

use super::builder::IRBuilder;
use irs::var::ArrVar;

use super::*;

use anyhow::Result;

use crate::backend::*;
use crate::middle;

impl IRBuilder {
    pub fn build_global_var(self_global_vars: &Vec<middle::ir::GlobalPtr>) -> Result<Vec<Var>> {
        let mut global_vars = Vec::new();
        for global_var in self_global_vars {
            let name = &global_var.name;
            let new_var = match &global_var.initializer {
                middle::ir::Constant::SignedChar(_) => unimplemented!(),
                middle::ir::Constant::Int(i) => Self::build_int_var(name, *i)?,
                middle::ir::Constant::Float(f) => Self::build_float_var(name, *f)?,
                middle::ir::Constant::Bool(b) => Self::build_bool_var(name, *b)?,
                middle::ir::Constant::Array(arr) => Self::build_arr_var(name, arr)?,
                middle::ir::Constant::Zero(ty) => Self::build_zero_initializer(name, ty)?,
            };
            global_vars.push(new_var);
        }
        Ok(global_vars)
    }

    fn build_zero_initializer(name: &str, ty: &middle::ir::ValueType) -> Result<Var> {
        match ty.get_base_type() {
            middle::ir::ValueType::Int => {
                let var: ArrVar<u32> = ArrVar {
                    name: name.to_string(),
                    capacity: ty.size(),
                    init: vec![],
                    is_const: false,
                };
                Ok(var.into())
            }
            middle::ir::ValueType::Float => {
                let var: ArrVar<f32> = ArrVar {
                    name: name.to_string(),
                    capacity: ty.size(),
                    init: vec![],
                    is_const: false,
                };
                Ok(var.into())
            }
            _ => Err(anyhow!("can't zero init for type {:?}", ty)).with_context(|| context!()),
        }
    }

    fn build_arr_var(name: &str, arr: &[middle::ir::Constant]) -> Result<Var> {
        if Self::_is_int(arr) {
            let mut init = Vec::new();
            let mut len = 0;
            for (sz, item) in Self::_init_arr_i(arr)? {
                if item != 0 {
                    init.push((len, item));
                }
                len += sz;
            }
            let var: ArrVar<u32> = ArrVar {
                name: name.to_string(),
                capacity: len,
                init,
                is_const: false,
            };
            Ok(var.into())
        } else {
            let mut init = Vec::new();
            let mut len = 0;
            for (sz, item) in Self::_init_arr_f(arr)? {
                if item != (0 as f32) {
                    init.push((len, item));
                }
                len += sz;
            }
            let var: ArrVar<f32> = ArrVar {
                name: name.to_string(),
                capacity: len,
                init,
                is_const: false,
            };
            Ok(var.into())
        }
    }

    fn _init_arr_i(
        arr: &[middle::ir::Constant],
    ) -> Result<Vec<(usize /* 大小, 而不是下标 */, u32)>> {
        let mut init = Vec::new();
        for item in arr {
            match item {
                middle::ir::Constant::Int(i) => init.push((1, *i as u32)),
                middle::ir::Constant::SignedChar(c) => init.push((1, *c as u32)),
                middle::ir::Constant::Bool(b) => init.push((1, *b as u32)),
                middle::ir::Constant::Float(_) => {
                    return Err(anyhow!("float in int arr")).with_context(|| context!())
                }
                middle::ir::Constant::Array(arr) => {
                    let sub_init = Self::_init_arr_i(arr)?;
                    init.extend(sub_init);
                }
                middle::ir::Constant::Zero(zero) => {
                    init.push((zero.size(), 0));
                }
            }
        }
        Ok(init)
    }

    fn _init_arr_f(
        arr: &[middle::ir::Constant],
    ) -> Result<Vec<(usize /* 大小, 而不是下标 */, f32)>> {
        let mut init = Vec::new();
        for item in arr {
            match item {
                middle::ir::Constant::SignedChar(_)
                | middle::ir::Constant::Int(_)
                | middle::ir::Constant::Bool(_) => {
                    return Err(anyhow!("int in float arr")).with_context(|| context!())
                }
                middle::ir::Constant::Float(f) => init.push((1, *f)),
                middle::ir::Constant::Array(arr) => {
                    let sub_init = Self::_init_arr_f(arr)?;
                    init.extend(sub_init);
                }
                middle::ir::Constant::Zero(zero) => {
                    init.push((zero.size(), 0 as f32));
                }
            }
        }
        Ok(init)
    }

    /// can't handle mixed arr
    fn __is_int(con: &middle::ir::Constant) -> Option<bool> {
        match con {
            middle::ir::Constant::Int(_) => Some(true),
            middle::ir::Constant::SignedChar(_) => Some(true),
            middle::ir::Constant::Bool(_) => Some(true),
            middle::ir::Constant::Float(_) => Some(false),
            middle::ir::Constant::Array(arr) => {
                for element in arr {
                    if let Some(is_int) = Self::__is_int(element) {
                        return Some(is_int);
                    }
                }
                None // 这种情况是: 全部是 Zero 的情况, 按道理来说应该走 build_zero_initializer 这条分支
            }
            middle::ir::Constant::Zero(_) => None,
        }
    }

    /// 全部是 Zero 也算是 int
    fn _is_int(arr: &[middle::ir::Constant]) -> bool {
        for item in arr {
            if let Some(is_int) = Self::__is_int(item) {
                return is_int;
            }
        }
        true
    }

    fn build_int_var(name: &str, value: i32) -> Result<Var> {
        let var = var::Var::Prim(var::PrimVar::IntVar(var::IntVar {
            name: name.to_string(),
            init: Some(value),
            is_const: false,
        }));
        Ok(var)
    }

    fn build_bool_var(name: &str, value: bool) -> Result<Var> {
        let var = var::Var::Prim(var::PrimVar::IntVar(var::IntVar {
            name: name.to_string(),
            init: Some(value as i32),
            is_const: false,
        }));
        Ok(var)
    }

    pub fn build_float_var(name: &str, f: f32) -> Result<Var> {
        let var = var::Var::Prim(var::PrimVar::FloatVar(var::FloatVar {
            name: name.to_string(),
            init: Some(f),
            is_const: false,
        }));
        Ok(var)
    }
}
