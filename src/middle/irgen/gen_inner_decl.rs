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

use crate::frontend::Decl;
use crate::middle::ir::Operand;
use crate::middle::irgen::function_kit::FunctionKit;
use crate::{context, middle::ir::Constant};
use anyhow::{anyhow, Context};

use super::gen_const::gen_const;
use super::gen_type::gen_type;
use super::value::{alloc, Value};

impl<'a> FunctionKit<'a> {
    /// Generate a declaration as a statement into the program
    pub fn gen_inner_decl(&mut self, decl: &Decl) -> anyhow::Result<()> {
        match decl {
            Decl::Const(raw_ty, id, op) => {
                // Make sure constant has an initializer
                let Some(expr) = op else {
                    return Err(anyhow!("const declaration must have an initializer"))
                        .with_context(|| context!());
                };

                // Translate type
                let value_type = gen_type(raw_ty)?;

                // Generate constant value
                let initializer = gen_const(expr)?;

                // If constant is an array, collapse it and store into global variable
                let val = match initializer {
                    Constant::Array(_) => {
                        let name = self.unique_name(id);
                        let gvar = self.program.mem_pool.new_global_variable(
                            name,
                            value_type,
                            false,
                            initializer,
                        );
                        self.program.module.global_variables.push(gvar);
                        Value::ReadWrite(gvar.into())
                    }
                    _ => Value::ReadOnly(initializer.into()),
                };

                // Add value to environment
                self.env.insert(id.clone(), val);
                Ok(())
            }
            Decl::Var(raw_ty, id, op) => {
                // Allocate space for variable, add to environment
                let ty = gen_type(raw_ty)?;
                let lhs = alloc(ty.clone(), self);
                self.env.insert(id.clone(), lhs.clone());

                // Assign to the variable if it is defined
                if let Some(expr) = op {
                    // Generate expression as variable type
                    let rhs = self.gen_expr(expr)?;

                    // Memset 0 if `rhs` is array
                    if let Value::Array(_) = rhs {
                        let Value::ReadWrite(ref ptr) = lhs else {
                            return Err(anyhow!("allocated variable must be read-write"))
                                .with_context(|| context!());
                        };
                        let memset_func = self
                            .fun_env
                            .get(&"llvm.memset.p0.i32".to_string())
                            .copied()
                            .unwrap();
                        let memset_call = self.program.mem_pool.get_call(
                            memset_func,
                            vec![
                                ptr.clone(),
                                Operand::Constant(Constant::SignedChar(0)),
                                Operand::Constant(Constant::Int(ty.size() as i32 * 4)),
                                Operand::Constant(Constant::Bool(false)),
                            ],
                        );
                        self.exit.unwrap().push_back(memset_call);
                    }

                    // Assign operand to value
                    lhs.assign(self, rhs)?;
                };
                Ok(())
            }
            Decl::Stack(decls) => {
                // Generate each declaration
                for decl in decls.iter() {
                    self.gen_inner_decl(decl)?;
                }
                Ok(())
            }
            _ => Err(anyhow!("unrecognized declaration {:?}", decl)).with_context(|| context!()),
        }
    }
}
