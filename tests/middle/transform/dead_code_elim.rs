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
pub mod tests_dce {
    use insta::assert_snapshot;

    use super::*;
    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{dead_code_elim, mem2reg},
        },
    };

    #[test]
    fn test_if() {
        let code = r#"
        int ifElseIf() {
            int a;
            a = 5;
            int b;
            b = 10;
            if(a == 6 || b == 0xb) {
                return a;
            }
            else {
                if (b == 10 && a == 1)
                a = 25;
                else if (b == 10 && a == -5)
                a = a + 15;
                else
                a = -+a;
            }

            return a;
        }

        int main(){
            putint(ifElseIf());
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
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
        define i32 @ifElseIf() {
        entry:
        [-] %alloca_2 = alloca i32
        [-] %alloca_5 = alloca i32
        [-] %alloca_7 = alloca i32
        br label %cond0

        cond0:
        %icmp_15 = icmp eq i32 5, 6
        br i1 %icmp_15, label %final5, label %alt4

        final5:
        %phi_22 = phi i1 [true, %cond0], [%icmp_20, %alt4]
        br i1 %phi_22, label %then1, label %alt2

        alt4:
        %icmp_20 = icmp eq i32 10, 11
        br label %final5

        then1:
        br label %exit

        alt2:
        br label %cond6

        exit:
        [-] %phi_83 = phi i32 [5, %then1], [%phi_85, %final3]
        %phi_82 = phi i32 [5, %then1], [%phi_85, %final3]
        ret i32 %phi_82

        cond6:
        %icmp_33 = icmp eq i32 10, 10
        br i1 %icmp_33, label %alt10, label %final11

        alt10:
        %icmp_38 = icmp eq i32 5, 1
        br label %final11

        final11:
        %phi_40 = phi i1 [false, %cond6], [%icmp_38, %alt10]
        br i1 %phi_40, label %then7, label %alt8

        then7:
        br label %final9

        alt8:
        br label %cond12

        final9:
        %phi_85 = phi i32 [25, %then7], [%phi_84, %final15]
        br label %final3

        cond12:
        %icmp_50 = icmp eq i32 10, 10
        br i1 %icmp_50, label %alt16, label %final17

        final3:
        br label %exit

        alt16:
        %Sub_54 = sub i32 0, 5
        %icmp_56 = icmp eq i32 5, %Sub_54
        br label %final17

        final17:
        %phi_58 = phi i1 [false, %cond12], [%icmp_56, %alt16]
        br i1 %phi_58, label %then13, label %alt14

        then13:
        %Add_61 = add i32 5, 15
        br label %final15

        alt14:
        %Sub_65 = sub i32 0, 5
        br label %final15

        final15:
        %phi_84 = phi i32 [%Add_61, %then13], [%Sub_65, %alt14]
        br label %final9


        }
        define i32 @main() {
        entry:
        [-] %alloca_75 = alloca i32
        %call_78 = call i32 @ifElseIf()
        call void @putint(i32 %call_78)
        br label %exit

        exit:
        ret i32 0


        }
        "###);
    }

    #[test]
    fn test_dce_gvar() {
        let code = r#"
        int x = 8;
        int y[100] = {};
        int z = 4;
        int w = 7;
        int main() {
            x = 9;
            y[2] = 1;
            return z;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        @x = dso_local global i32 8
        @y = dso_local global [100 x i32] zeroinitializer
        @z = dso_local global i32 4
        [-] @w = dso_local global i32 7
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
        [-] %alloca_2 = alloca i32
        store i32 9, ptr @x
        %getelementptr_6 = getelementptr [100 x i32], ptr @y, i32 0, i32 2
        store i32 1, ptr %getelementptr_6
        %load_8 = load i32, ptr @z
        br label %exit

        exit:
        ret i32 %load_8


        }
        "###);
    }

    #[test]
    fn test_dce_loop() {
        let code = r#"
        int main() {
            int x = 8;
            int y = 2;
            while (y < 9) {
                x = x + 1;
                y = y + 1;
            }
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
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
        define i32 @main() {
        entry:
        [-] %alloca_2 = alloca i32
        [-] %alloca_5 = alloca i32
        [-] %alloca_7 = alloca i32
        br label %cond0

        cond0:
        %phi_26 = phi i32 [2, %entry], [%Add_17, %body1]
        %phi_25 = phi i32 [8, %entry], [%Add_14, %body1]
        %icmp_21 = icmp slt i32 %phi_26, 9
        br i1 %icmp_21, label %body1, label %final2

        body1:
        %Add_14 = add i32 %phi_25, 1
        %Add_17 = add i32 %phi_26, 1
        br label %cond0

        final2:
        br label %exit

        exit:
        ret i32 0


        }
        "###);
    }

    #[test]
    fn test_dce_1() {
        let code = r#"
        int loopCount = 0;
        int global = 0;

        void func(int i0)
        {
            int i1 = 1;
            int i2 = 2;
            int i3 = 3;
            int i4 = 4;
            int i5 = 5;
            global = i0;
            return;
        }

        int main()
        {
            int sum = 0;
            int i = 0;
            loopCount = getint();
            starttime();
            while(i<loopCount)
            {
                int tmp = 0;
                int j = 0;
                while(j<60)
                {
                func(i);
                tmp = tmp + global;
                j = j + 1;
                }
                tmp = tmp / 60;
                sum = sum + tmp;
                sum = sum % 134209537;
                i = i + 1;
            }
            stoptime();
            putint(sum);
            putch(10);
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
        @loopCount = dso_local global i32 0
        @global = dso_local global i32 0
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
        define void @func(i32 %i0) {
        entry:
        [-] %alloca_3 = alloca i32
        [-] %alloca_5 = alloca i32
        [-] %alloca_7 = alloca i32
        [-] %alloca_9 = alloca i32
        [-] %alloca_11 = alloca i32
        [-] %alloca_13 = alloca i32
        store i32 %i0, ptr @global
        br label %exit

        exit:
        ret void


        }
        define i32 @main() {
        entry:
        [-] %alloca_20 = alloca i32
        [-] %alloca_23 = alloca i32
        [-] %alloca_25 = alloca i32
        %call_27 = call i32 @getint()
        store i32 %call_27, ptr @loopCount
        call void @_sysy_starttime(i32 21)
        br label %cond0

        cond0:
        [-] %phi_83 = phi i32 [0, %entry], [%phi_84, %final5]
        [-] %phi_81 = phi i32 [0, %entry], [%SDiv_56, %final5]
        %phi_80 = phi i32 [0, %entry], [%Add_66, %final5]
        %phi_79 = phi i32 [0, %entry], [%SRem_63, %final5]
        %load_70 = load i32, ptr @loopCount
        %icmp_71 = icmp slt i32 %phi_80, %load_70
        br i1 %icmp_71, label %body1, label %final2

        body1:
        [-] %alloca_34 = alloca i32
        [-] %alloca_36 = alloca i32
        br label %cond3

        final2:
        call void @_sysy_stoptime(i32 37)
        call void @putint(i32 %phi_79)
        call void @putch(i32 10)
        br label %exit

        cond3:
        %phi_84 = phi i32 [0, %body1], [%Add_49, %body4]
        %phi_82 = phi i32 [0, %body1], [%Add_46, %body4]
        %icmp_53 = icmp slt i32 %phi_84, 60
        br i1 %icmp_53, label %body4, label %final5

        exit:
        ret i32 0

        body4:
        call void @func(i32 %phi_80)
        %load_45 = load i32, ptr @global
        %Add_46 = add i32 %phi_82, %load_45
        %Add_49 = add i32 %phi_84, 1
        br label %cond3

        final5:
        %SDiv_56 = sdiv i32 %phi_82, 60
        %Add_60 = add i32 %phi_79, %SDiv_56
        %SRem_63 = srem i32 %Add_60, 134209537
        %Add_66 = add i32 %phi_80, 1
        br label %cond0


        }
        "###);
    }
}
