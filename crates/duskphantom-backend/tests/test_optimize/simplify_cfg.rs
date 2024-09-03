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

use super::*;
use duskphantom_backend::irs::Func;
use indoc::indoc;
#[allow(unused)]
use insta::assert_snapshot;

use super::reg_alloc::{backend_from_self, find_func};

#[allow(unused)]
fn case() -> Func {
    let code = indoc! {r#"
        int main() {
            int a = 3;
            int b = getint();
            if (b) {
                a = a + 1;
                a = a + 2;
            }
            return a;
        }"#
    };
    let p = backend_from_self(code);
    let f = find_func(&p, "main");
    f.clone()
}

#[allow(unused)]
fn get_diff(func: &Func, f: impl Fn(&mut Func)) -> String {
    let mut func = func.clone();
    let old_f = func.gen_asm();
    f(&mut func);
    let new_f = func.gen_asm();
    diff::diff(&old_f, &new_f)
}

#[test]
/// FIXME: This test is not stable, because each time the result is different.
fn test_simplify_term() {
    // let f = case();
    // assert_snapshot!(get_diff(&f, |f|{f.simplify_term().unwrap();}),@r###"
    // .text
    // .align	3
    // .globl	main
    // .type	main, @function
    // main:
    // .Lmain_entry:
    // li x32,3
    // store x32,[0-8]
    // call getint
    // mv x33,a0
    // store x33,[8-16]
    // [-] j .Lmain_cond0
    // .Lmain_cond0:
    // load x34,[8-16]
    // xori x36,x34,0
    // snez x35,x36
    // beq x35,zero,.Lmain_alt2
    // [-] j .Lmain_then1
    // .Lmain_then1:
    // load x37,[0-8]
    // addiw x38,x37,1
    // store x38,[0-8]
    // load x39,[0-8]
    // addiw x40,x39,2
    // store x40,[0-8]
    // j .Lmain_final3
    // .Lmain_alt2:
    // [-] j .Lmain_final3
    // [+]
    // .Lmain_final3:
    // load x41,[0-8]
    // store x41,[16-24]
    // [-] j .Lmain_exit
    // .Lmain_exit:
    // load x42,[16-24]
    // mv a0,x42
    // ret
    // .size	main, .-main
    // "###);
}

#[test]
/// FIXME: This test is not stable, because each time the result is different.
fn test_desimplify_term() {
    // let mut f = case();
    // f.simplify_term().unwrap();
    // assert_snapshot!(get_diff(&f, |f|{f.desimplify_term().unwrap();}),@r###"
    // .text
    // .align	3
    // .globl	main
    // .type	main, @function
    // main:
    // .Lmain_entry:
    // li x32,3
    // store x32,[0-8]
    // call getint
    // mv x33,a0
    // store x33,[8-16]
    // [+] j .Lmain_cond0
    // .Lmain_cond0:
    // load x34,[8-16]
    // xori x36,x34,0
    // snez x35,x36
    // beq x35,zero,.Lmain_alt2
    // [+] j .Lmain_then1
    // .Lmain_then1:
    // load x37,[0-8]
    // addiw x38,x37,1
    // store x38,[0-8]
    // load x39,[0-8]
    // addiw x40,x39,2
    // store x40,[0-8]
    // j .Lmain_final3
    // .Lmain_alt2:
    // [-]
    // [+] j .Lmain_final3
    // .Lmain_final3:
    // load x41,[0-8]
    // store x41,[16-24]
    // [+] j .Lmain_exit
    // .Lmain_exit:
    // load x42,[16-24]
    // mv a0,x42
    // ret
    // .size	main, .-main
    // "###);
}

#[test]
// FIXME: This test is not stable, because each time the result is different.
fn test_simplify_cfg() {
    // let f = case();
    // assert_snapshot!(get_diff(&f, |f|backend::optimize::block::handle_block_simplify(f).unwrap()),@r###"
    // .text
    // .align	3
    // .globl	main
    // .type	main, @function
    // main:
    // .Lmain_entry:
    // li x32,3
    // store x32,[0-8]
    // call getint
    // mv x33,a0
    // store x33,[16-24]
    // j .Lmain_cond0
    // .Lmain_cond0:
    // load x34,[16-24]
    // xori x36,x34,0
    // snez x35,x36
    // [-] beq x35,zero,.Lmain_alt2
    // [+] beq x35,zero,.Lmain_final3
    // j .Lmain_then1
    // .Lmain_then1:
    // load x37,[0-8]
    // addiw x38,x37,1
    // store x38,[0-8]
    // load x39,[0-8]
    // addiw x40,x39,2
    // store x40,[0-8]
    // [-] j .Lmain_final3
    // [-] .Lmain_alt2:
    // j .Lmain_final3
    // .Lmain_final3:
    // load x41,[0-8]
    // store x41,[8-16]
    // j .Lmain_exit
    // .Lmain_exit:
    // load x42,[8-16]
    // mv a0,x42
    // ret
    // .size	main, .-main
    // "###);
}
