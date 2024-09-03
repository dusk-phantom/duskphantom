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

use duskphantom_backend::irs::*;
use insta::assert_debug_snapshot;

#[test]
fn test_int_arr1() {
    let var = Var::IntArr(ArrVar {
        name: "arr".to_string(),
        capacity: 2,
        init: vec![],
        is_const: false,
    });
    assert_debug_snapshot!(var.gen_asm(), @r###"".bss\n.align\t3\n.globl\tarr\n.type\tarr, @object\n.size\tarr, 8\narr:\n.zero\t8""###);
}

#[test]
fn test_int_arr2() {
    let var = Var::IntArr(ArrVar {
        name: "arr".to_string(),
        capacity: 99,
        init: vec![(0, 1), (3, 2), (4, 3)],
        is_const: false,
    });
    assert_debug_snapshot!(var.gen_asm(), @r###"".data\n.align\t3\n.globl\tarr\n.type\tarr, @object\n.size\tarr, 396\narr:\n.word\t0x1\n.zero\t8\n.word\t0x2\n.word\t0x3\n.zero\t376""###);
}
