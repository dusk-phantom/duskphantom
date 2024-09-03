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
#[cfg(test)]
pub mod tests_block_fuse {
    use insta::assert_snapshot;

    use duskphantom_frontend::parse;
    use duskphantom_middle::{
        irgen::gen,
        transform::{block_fuse, constant_fold, dead_code_elim, inst_combine, mem2reg},
    };

    use super::*;

    #[test]
    fn test_trivial_merge() {
        let code = r#"
        int main() {
            putint(1);
            if (2 > 5) {
                putint(2);
            } else {
                putint(3);
            }
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        block_fuse::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        declare i32 @getint()
        declare i32 @getch()
        declare float @getfloat()
        declare void @putint(i32 %p0)
        declare void @putch(i32 %p0)
        declare void @putfloat(float %p0)
        declare i32 @getarray(i32* %p0)
        declare i32 @getfarray(float* %p0)
        declare void @putarray(i32 %p0, i32* %p1)
        declare void @putfarray(i32 %p0, float* %p1)
        declare void @_sysy_starttime(i32 %p0)
        declare void @_sysy_stoptime(i32 %p0)
        declare i32 @thrd_create(i32 %p0)
        declare void @thrd_join()
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @main() {
        [-] entry:
        [+] exit:
        call void @putint(i32 1)
        [-] br label %cond0
        [-] 
        [-] cond0:
        [-] br label %alt2
        [-] 
        [-] alt2:
        call void @putint(i32 3)
        [-] br label %final3
        [-] 
        [-] final3:
        [-] br label %exit
        [-] 
        [-] exit:
        ret i32 0


        }
        "###);
    }

    #[test]
    fn test_single_br_merge() {
        let code = r#"
        int main() {
            putint(1);
            if (2 > 5) {
                putint(2);
            }
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        block_fuse::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        declare i32 @getint()
        declare i32 @getch()
        declare float @getfloat()
        declare void @putint(i32 %p0)
        declare void @putch(i32 %p0)
        declare void @putfloat(float %p0)
        declare i32 @getarray(i32* %p0)
        declare i32 @getfarray(float* %p0)
        declare void @putarray(i32 %p0, i32* %p1)
        declare void @putfarray(i32 %p0, float* %p1)
        declare void @_sysy_starttime(i32 %p0)
        declare void @_sysy_stoptime(i32 %p0)
        declare i32 @thrd_create(i32 %p0)
        declare void @thrd_join()
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @main() {
        [-] entry:
        [-] call void @putint(i32 1)
        [-] br label %cond0
        [-] 
        [-] cond0:
        [-] br label %alt2
        [-] 
        [-] alt2:
        [-] br label %final3
        [-] 
        [-] final3:
        [-] br label %exit
        [-] 
        exit:
        [+] call void @putint(i32 1)
        ret i32 0


        }
        "###);
    }
}
