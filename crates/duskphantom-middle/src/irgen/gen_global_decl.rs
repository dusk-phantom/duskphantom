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
use crate::frontend::{Decl, Type};
use crate::irgen::program_kit::ProgramKit;
use crate::irgen::value::Value;
use anyhow::{anyhow, Context};

use super::gen_const::gen_const;
use super::gen_type::gen_type;

impl<'a> ProgramKit<'a> {
    /// Generate a global declaration into the program
    /// Fails when declaration does not have a name
    pub fn gen_global_decl(&mut self, decl: &Decl) -> anyhow::Result<()> {
        match decl {
            Decl::Var(ty, name, val) | Decl::Const(ty, name, val) => {
                // Get variable type
                let value_type = gen_type(ty)?;

                // Get if value is global variable or constant
                let is_global_variable: bool = match decl {
                    Decl::Var(_, _, _) => true,
                    Decl::Const(_, _, _) => false,
                    _ => false,
                };

                // Get initializer
                let initializer = match val {
                    Some(v) => gen_const(v)?,
                    None => value_type.default_initializer()?,
                };

                // Get global variable
                let global_val = self.program.mem_pool.new_global_variable(
                    name.clone(),
                    value_type,
                    is_global_variable,
                    initializer,
                );

                // Add global variable (pointer) to environment
                self.env
                    .insert(name.clone(), Value::ReadWrite(global_val.into()));

                // Add global variable to program
                self.program.module.global_variables.push(global_val);
                Ok(())
            }
            Decl::Func(Type::Function(return_ty, params), id, _) => {
                // Get function type
                let fty = gen_type(return_ty)?;

                // Create function
                let mut fun_ptr = self.program.mem_pool.new_function(id.clone(), fty.clone());

                // Generate parameters
                for param in params.iter() {
                    let value_type = gen_type(&param.ty)?;
                    let param = self
                        .program
                        .mem_pool
                        .new_parameter(param.id.clone().unwrap_or("_".to_string()), value_type);
                    fun_ptr.params.push(param);
                }

                // Add function to environment
                self.fun_env.insert(id.clone(), fun_ptr);

                // Add function to program
                self.program.module.functions.push(fun_ptr);
                Ok(())
            }
            Decl::Stack(ls) => {
                for l in ls.iter() {
                    self.gen_global_decl(l)?;
                }
                Ok(())
            }
            _ => Err(anyhow!("unrecognized declaration {:?}", decl)).with_context(|| context!()),
        }
    }
}
