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

mod algebra;
mod control_flow;
mod convert;
mod data_move;
mod inst;
mod reg_def_use;
mod test;
pub use super::*;
pub use crate::{
    impl_inst_convert, impl_mem_inst, impl_three_op_inst, impl_two_op_inst, impl_unary_inst,
};
pub use algebra::*;
pub use control_flow::*;
pub use convert::*;
pub use data_move::*;
pub use inst::*;
pub use reg_def_use::*;
