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
use crate::frontend::{Expr, UnaryOp};
use crate::middle::ir::{Constant, ValueType};
use crate::middle::irgen::function_kit::FunctionKit;
use crate::middle::irgen::value::Value;
use anyhow::{anyhow, Context};

impl<'a> FunctionKit<'a> {
    /// Generate a unary expression
    pub fn gen_unary(&mut self, op: &UnaryOp, expr: &Expr) -> anyhow::Result<Value> {
        let Some(mut exit) = self.exit else {
            return Err(anyhow!("exit block can't be appended")).with_context(|| context!());
        };

        // Generate argument
        let val = self.gen_expr(expr)?;

        // Calculate type for operator polymorphism
        let ty = val.get_type();

        // Apply operation
        match op {
            UnaryOp::Neg => {
                // Return 0 - x
                let operand = val.load(ty.clone(), self)?;
                match ty {
                    ValueType::Int => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_sub(Constant::Int(0).into(), operand);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Float => {
                        let inst = self
                            .program
                            .mem_pool
                            .get_fsub(Constant::Float(0.0).into(), operand);
                        exit.push_back(inst);
                        Ok(Value::ReadOnly(inst.into()))
                    }
                    ValueType::Bool => {
                        // Convert to int and then make negative
                        let zext = self.program.mem_pool.get_zext(operand);
                        let sub = self
                            .program
                            .mem_pool
                            .get_sub(Constant::Int(0).into(), zext.into());
                        exit.push_back(zext);
                        exit.push_back(sub);
                        Ok(Value::ReadOnly(sub.into()))
                    }
                    _ => Err(anyhow!("`-` for NaN")).with_context(|| context!()),
                }
            }
            UnaryOp::Pos => {
                // Return operand directly
                let operand = val.load(ty.clone(), self)?;
                match ty {
                    ValueType::Int | ValueType::Float | ValueType::Bool => {
                        Ok(Value::ReadOnly(operand))
                    }
                    _ => Err(anyhow!("`+` for NaN")).with_context(|| context!()),
                }
            }
            UnaryOp::Not => {
                // Load as boolean
                let bool_op = val.load(ValueType::Bool, self)?;

                // Add "xor" instruction
                let inst = self
                    .program
                    .mem_pool
                    .get_xor(bool_op, Constant::Bool(true).into());
                exit.push_back(inst);
                Ok(Value::ReadOnly(inst.into()))
            }
        }
    }
}
