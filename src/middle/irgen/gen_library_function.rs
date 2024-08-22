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

use crate::middle::ir::ValueType;

use super::program_kit::ProgramKit;

impl<'a> ProgramKit<'a> {
    /// Declare library functions in the program
    pub fn gen_library_function(&mut self) {
        let mut insert = |name: &str, return_ty: ValueType, param_ty: Vec<ValueType>| {
            let mut fun_ptr = self
                .program
                .mem_pool
                .new_function(name.to_string(), return_ty);
            self.fun_env.insert(name.to_string(), fun_ptr);
            self.program.module.functions.push(fun_ptr);

            // Insert parameters
            for (i, ty) in param_ty.iter().enumerate() {
                let param = self
                    .program
                    .mem_pool
                    .new_parameter(format!("p{}", i), ty.clone());
                fun_ptr.params.push(param);
            }
        };

        insert("getint", ValueType::Int, vec![]);
        insert("getch", ValueType::Int, vec![]);
        insert("getfloat", ValueType::Float, vec![]);
        insert("putint", ValueType::Void, vec![ValueType::Int]);
        insert("putch", ValueType::Void, vec![ValueType::Int]);
        insert("putfloat", ValueType::Void, vec![ValueType::Float]);
        insert(
            "getarray",
            ValueType::Int,
            vec![ValueType::Pointer(ValueType::Int.into())],
        );
        insert(
            "getfarray",
            ValueType::Int,
            vec![ValueType::Pointer(ValueType::Float.into())],
        );
        insert(
            "putarray",
            ValueType::Void,
            vec![ValueType::Int, ValueType::Pointer(ValueType::Int.into())],
        );
        insert(
            "putfarray",
            ValueType::Void,
            vec![ValueType::Int, ValueType::Pointer(ValueType::Float.into())],
        );
        insert("_sysy_starttime", ValueType::Void, vec![ValueType::Int]);
        insert("_sysy_stoptime", ValueType::Void, vec![ValueType::Int]);
        insert("thrd_create", ValueType::Int, vec![ValueType::Int]);
        insert("thrd_join", ValueType::Void, vec![]);
        insert("putf", ValueType::Void, vec![]);
        insert(
            "llvm.memset.p0.i32",
            ValueType::Void,
            vec![
                ValueType::Pointer(ValueType::Int.into()),
                ValueType::SignedChar,
                ValueType::Int,
                ValueType::Bool,
            ],
        );
    }
}

pub fn is_argument_const(func_name: &str, index: usize) -> bool {
    func_name == "putf" && index == 0
}
