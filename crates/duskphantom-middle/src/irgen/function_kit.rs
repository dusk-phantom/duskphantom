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


use crate::ir::{BBPtr, FunPtr, ValueType};
use crate::irgen::value::Value;
use duskphantom_utils::frame_map::FrameMap;

/// Kit for translating a function to middle IR
pub struct FunctionKit<'a> {
    pub env: FrameMap<'a, String, Value>,
    pub fun_env: FrameMap<'a, String, FunPtr>,
    pub program: &'a mut crate::Program,
    pub exit: Option<BBPtr>,
    pub break_to: Option<BBPtr>,
    pub continue_to: Option<BBPtr>,
    pub return_to: BBPtr,
    pub return_value: Option<Value>,
    pub return_type: ValueType,
    pub counter: &'a mut usize,
}

/// Context for FunctionKit
pub struct FunctionContext<'a> {
    pub env: FrameMap<'a, String, Value>,
    pub fun_env: FrameMap<'a, String, FunPtr>,
    pub program: &'a mut crate::Program,
    pub counter: &'a mut usize,
}

/// Routing for FunctionKit
pub struct FunctionRouting {
    pub exit: Option<BBPtr>,
    pub break_to: Option<BBPtr>,
    pub continue_to: Option<BBPtr>,
    pub return_to: BBPtr,
    pub return_value: Option<Value>,
    pub return_type: ValueType,
}

/// A function kit can generate statements
impl<'a> FunctionKit<'a> {
    /// Create a new function kit
    /// Environment will remain the same after drop
    pub fn new(ctx: FunctionContext<'a>, routing: FunctionRouting) -> Self {
        // Create function kit
        FunctionKit {
            env: ctx.env,
            fun_env: ctx.fun_env,
            program: ctx.program,
            exit: routing.exit,
            break_to: routing.break_to,
            continue_to: routing.continue_to,
            return_to: routing.return_to,
            return_value: routing.return_value,
            return_type: routing.return_type,
            counter: ctx.counter,
        }
    }

    /// Generate a new function kit from existing one
    pub fn gen_function_kit(
        &mut self,
        exit: Option<BBPtr>,
        break_to: Option<BBPtr>,
        continue_to: Option<BBPtr>,
    ) -> FunctionKit {
        FunctionKit::new(
            FunctionContext {
                env: self.env.branch(),
                fun_env: self.fun_env.branch(),
                program: self.program,
                counter: self.counter,
            },
            FunctionRouting {
                exit,
                break_to,
                continue_to,
                return_to: self.return_to,
                return_value: self.return_value.clone(),
                return_type: self.return_type.clone(),
            },
        )
    }

    /// Generate a unique basic block name
    pub fn unique_name(&mut self, base: &str) -> String {
        let name = format!("{}{}", base, self.counter);
        *self.counter += 1;
        name
    }
}
