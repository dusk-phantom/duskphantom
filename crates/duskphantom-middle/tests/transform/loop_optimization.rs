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
pub mod tests_make_parallel {

    use super::*;

    use duskphantom_frontend::parse;
    use duskphantom_middle::{
        irgen::gen,
        transform::{dead_code_elim, inst_combine, loop_optimization, mem2reg, redundance_elim},
    };

    use insta::assert_snapshot;

    #[test]
    fn test_use_arr() {
        let code = r#"
        int A[9];
        int B[9];
        int main() {
            int n = getarray(A);
            int i = 0;
            while (i < A[n - 1]) {
                B[i] = B[i] + 1;
                i = i + 1;
            }
            putarray(n, A);
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        redundance_elim::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        loop_optimization::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
        @A = dso_local global [9 x i32] zeroinitializer
        @B = dso_local global [9 x i32] zeroinitializer
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
        entry:
        %getelementptr_6 = getelementptr [9 x i32], ptr @A, i32 0, i32 0
        %call_7 = call i32 @getarray(i32* %getelementptr_6)
        [+] %Sub_27 = sub i32 %call_7, 1
        [+] %getelementptr_28 = getelementptr [9 x i32], ptr @A, i32 0, i32 %Sub_27
        [+] %load_30 = load i32, ptr %getelementptr_28
        br label %cond0

        cond0:
        %phi_38 = phi i32 [0, %entry], [%Add_23, %body1]
        [-] %Sub_27 = sub i32 %call_7, 1
        [-] %getelementptr_28 = getelementptr [9 x i32], ptr @A, i32 0, i32 %Sub_27
        [-] %load_30 = load i32, ptr %getelementptr_28
        %icmp_31 = icmp slt i32 %phi_38, %load_30
        br i1 %icmp_31, label %body1, label %final2

        body1:
        %getelementptr_16 = getelementptr [9 x i32], ptr @B, i32 0, i32 %phi_38
        %load_17 = load i32, ptr %getelementptr_16
        %Add_18 = add i32 %load_17, 1
        store i32 %Add_18, ptr %getelementptr_16
        %Add_23 = add i32 %phi_38, 1
        br label %cond0

        final2:
        call void @putarray(i32 %call_7, i32* %getelementptr_6)
        br label %exit

        exit:
        ret i32 0


        }
        "###);
    }
}
