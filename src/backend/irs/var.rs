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

use gen_asm::{Data, GenTool};

#[allow(unused)]
use super::*;

#[derive(Clone, Debug)]
pub enum Var {
    Prim(PrimVar),
    Str(Str),
    IntArr(ArrVar<u32>),
    FloatArr(ArrVar<f32>),
}

#[derive(Clone, Debug)]
pub enum PrimVar {
    IntVar(IntVar),
    FloatVar(FloatVar),
}
#[derive(Clone, Debug)]
pub struct IntVar {
    pub name: String,
    pub init: Option<i32>,
    pub is_const: bool,
}
#[derive(Clone, Debug)]
pub struct Str {
    pub name: String,
    pub init: Option<String>,
    pub is_const: bool,
}
impl Str {
    fn gen_asm(&self) -> String {
        let init = self.init.as_deref().unwrap_or("");
        GenTool::gen_const_str(&self.name, init)
    }
}
#[derive(Clone, Debug)]
pub struct FloatVar {
    pub name: String,
    pub init: Option<f32>,
    pub is_const: bool,
}
#[derive(Clone, Debug)]
pub struct ArrVar<T: Data> {
    pub name: String,
    pub capacity: usize,
    pub init: Vec<(usize, T)>,
    pub is_const: bool,
}
impl<T: Data> ArrVar<T> {
    pub fn new(name: String, capacity: usize, init: Vec<(usize, T)>, is_const: bool) -> Self {
        Self {
            name,
            capacity,
            init,
            is_const,
        }
    }
}
impl<T: Data> ArrVar<T> {
    pub fn gen_asm(&self) -> String {
        GenTool::gen_array(&self.name, self.capacity, &self.init)
    }
}
impl PrimVar {
    pub fn gen_asm(&self) -> String {
        match self {
            PrimVar::IntVar(var) => var.gen_asm(),
            PrimVar::FloatVar(var) => var.gen_asm(),
        }
    }
}
impl IntVar {
    pub fn gen_asm(&self) -> String {
        GenTool::gen_int(&self.name, self.init.unwrap_or(0))
    }
}
impl FloatVar {
    pub fn gen_asm(&self) -> String {
        GenTool::gen_float(&self.name, self.init.unwrap_or(0.0))
    }
}

impl Var {
    pub fn gen_asm(&self) -> String {
        match self {
            Var::Prim(prim) => prim.gen_asm(),
            Var::Str(str) => str.gen_asm(),
            Var::IntArr(arr) => arr.gen_asm(),
            Var::FloatArr(arr) => arr.gen_asm(),
        }
    }
}

// impl from for vars
impl From<FloatVar> for Var {
    fn from(value: FloatVar) -> Self {
        Var::Prim(PrimVar::FloatVar(value))
    }
}
impl From<IntVar> for Var {
    fn from(value: IntVar) -> Self {
        Var::Prim(PrimVar::IntVar(value))
    }
}
impl From<ArrVar<u32>> for Var {
    fn from(value: ArrVar<u32>) -> Self {
        Var::IntArr(value)
    }
}
impl From<ArrVar<f32>> for Var {
    fn from(value: ArrVar<f32>) -> Self {
        Var::FloatArr(value)
    }
}
