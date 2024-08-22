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

pub mod ir;
pub mod parse;
pub mod preprocess;
pub mod transform;

pub use ir::*;
use transform::constant_fold;

use crate::errors::FrontendError;

#[allow(unused)]
pub fn parse(src: &str) -> Result<Program, FrontendError> {
    let preprocessed = preprocess::timing::process(src);
    let mut program = parse::program::parse(&preprocessed)?;
    match constant_fold::optimize_program(&mut program) {
        Ok(_) => Ok(program),
        Err(e) => Err(FrontendError::OptimizeError),
    }
}

#[allow(unused)]
pub fn optimize(program: &mut Program) {}
