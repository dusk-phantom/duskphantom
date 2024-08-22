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

#[cfg(test)]
pub mod tests_symbolic_eval {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{constant_fold, dead_code_elim, mem2reg, inst_combine},
        },
        utils::diff::diff,
    };

    #[test]
    fn test_merge_phi() {
        let code = r#"
        int f(int x) {
            int a = getint();
            if (x < 5) {
                a = 1;
            } else {
                a = 1;
            }
            return a;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
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
        define i32 @f(i32 %x) {
        entry:
        %call_8 = call i32 @getint()
        br label %cond0

        cond0:
        %icmp_16 = icmp slt i32 %x, 5
        br i1 %icmp_16, label %then1, label %alt2

        then1:
        br label %final3

        alt2:
        br label %final3

        final3:
        [-] %phi_25 = phi i32 [1, %then1], [1, %alt2]
        br label %exit

        exit:
        [-] ret i32 %phi_25
        [+] ret i32 1


        }
        "###);
    }

    #[test]
    fn test_merge_add() {
        let code = r#"
        int f(int x) {
            int a = getint();
            int b = a + a;
            b = b + a;
            b = b + a;
            b = b + a;
            b = b + a;
            return b;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
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
        define i32 @f(i32 %x) {
        entry:
        %call_8 = call i32 @getint()
        [-] %Add_13 = add i32 %call_8, %call_8
        [-] %Add_17 = add i32 %Add_13, %call_8
        [-] %Add_21 = add i32 %Add_17, %call_8
        [-] %Add_25 = add i32 %Add_21, %call_8
        [-] %Add_29 = add i32 %Add_25, %call_8
        [+] %Mul_38 = mul i32 %call_8, 6
        br label %exit

        exit:
        [-] ret i32 %Add_29
        [+] ret i32 %Mul_38


        }
        "###);
    }

    #[test]
    fn test_normal() {
        let code = r#"
        int f(int x) {
            int x1 = x + 0 + 0 + x;
            int x2 = 2 * x1 * 5;
            int x3 = x2 * 3;
            return x3;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        inst_combine::optimize_program(&mut program).unwrap();
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
        define i32 @f(i32 %x) {
        entry:
        [-] %Add_9 = add i32 %x, 0
        [-] %Add_10 = add i32 %Add_9, 0
        [-] %Add_12 = add i32 %Add_10, %x
        [-] %Mul_16 = mul i32 2, %Add_12
        [-] %Mul_17 = mul i32 %Mul_16, 5
        [-] %Mul_21 = mul i32 %Mul_17, 3
        [+] %Mul_29 = mul i32 %x, 60
        br label %exit

        exit:
        [-] ret i32 %Mul_21
        [+] ret i32 %Mul_29


        }
        "###);
    }

    #[test]
    fn test_add_sub() {
        let code = r#"
        int f(int x) {
            int x1 = x + 1 - 4 + 6 + 8;
            return x1;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        inst_combine::optimize_program(&mut program).unwrap();
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
        define i32 @f(i32 %x) {
        entry:
        [-] %Add_9 = add i32 %x, 1
        [-] %Sub_10 = sub i32 %Add_9, 4
        [-] %Add_11 = add i32 %Sub_10, 6
        [-] %Add_12 = add i32 %Add_11, 8
        [+] %Add_19 = add i32 %x, 11
        br label %exit

        exit:
        [-] ret i32 %Add_12
        [+] ret i32 %Add_19


        }
        "###);
    }

    #[test]
    fn test_shift() {
        let code = r#"
        int f(int x) {
            int x1 = x * 2;
            int x2 = x1 * 2;
            int x3 = x2 * 2;
            return x3;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        inst_combine::optimize_program(&mut program).unwrap();
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
        define i32 @f(i32 %x) {
        entry:
        [-] %Mul_9 = mul i32 %x, 2
        [-] %Mul_13 = mul i32 %Mul_9, 2
        [-] %Mul_17 = mul i32 %Mul_13, 2
        [+] %Mul_23 = mul i32 %x, 8
        br label %exit

        exit:
        [-] ret i32 %Mul_17
        [+] ret i32 %Mul_23


        }
        "###);
    }

    #[test]
    fn test_div() {
        let code = r#"
        int f(int x) {
            int x1 = x / x;
            int x2 = x1 * x;
            int x3 = x2 / 2;
            int x4 = x3 / 2;
            int x5 = x4 / 2;
            return x5;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        inst_combine::optimize_program(&mut program).unwrap();
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
        define i32 @f(i32 %x) {
        entry:
        [-] %SDiv_10 = sdiv i32 %x, %x
        [-] %Mul_15 = mul i32 %SDiv_10, %x
        [-] %SDiv_19 = sdiv i32 %Mul_15, 2
        [-] %SDiv_23 = sdiv i32 %SDiv_19, 2
        [-] %SDiv_27 = sdiv i32 %SDiv_23, 2
        [+] %SDiv_33 = sdiv i32 %x, 8
        br label %exit

        exit:
        [-] ret i32 %SDiv_27
        [+] ret i32 %SDiv_33


        }
        "###);
    }

    #[test]
    fn test_div_overflow() {
        let code = r#"
        int f(int x0) {
            int x1 = x0 / 256;
            int x2 = x1 / 256;
            int x3 = x2 / 256;
            int x4 = x3 / 256;
            int x5 = x4 / 256;
            return x5;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        inst_combine::optimize_program(&mut program).unwrap();
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
        define i32 @f(i32 %x0) {
        entry:
        [-] %SDiv_9 = sdiv i32 %x0, 256
        [-] %SDiv_13 = sdiv i32 %SDiv_9, 256
        [-] %SDiv_17 = sdiv i32 %SDiv_13, 256
        [-] %SDiv_21 = sdiv i32 %SDiv_17, 256
        [-] %SDiv_25 = sdiv i32 %SDiv_21, 256
        br label %exit

        exit:
        [-] ret i32 %SDiv_25
        [+] ret i32 0


        }
        "###);
    }

    #[test]
    fn test_gvar() {
        let code = r#"
        #include "../../lib/sylib.h"
        //test domain of global var define and local define
        int a = 3;
        int b = 5;

        int main(){
            int a = 5;
            return a + b;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        @b = dso_local global i32 5
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
        %load_8 = load i32, ptr @b
        [-] %Add_9 = add i32 5, %load_8
        [+] %Add_9 = add i32 %load_8, 5
        br label %exit

        exit:
        ret i32 %Add_9


        }
        "###);
    }
}
