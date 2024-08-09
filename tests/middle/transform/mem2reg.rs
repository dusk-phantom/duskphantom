#[cfg(test)]
pub mod tests_mem2reg {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{irgen::gen, transform::mem2reg},
        utils::diff::diff,
    };

    #[test]
    fn test_mem2reg_simple() {
        let code = r#"
            int main() {
                int a = 1;
                return a;
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg::optimize_program(&mut program).unwrap();
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
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        [-] store i32 1, ptr %alloca_5
        [-] %load_7 = load i32, ptr %alloca_5
        [-] store i32 %load_7, ptr %alloca_2
        br label %exit

        exit:
        [-] %load_3 = load i32, ptr %alloca_2
        [-] ret i32 %load_3
        [+] ret i32 1


        }
        "###);
    }

    #[test]
    fn test_mem2reg_branch() {
        let code = r#"
            int main() {
                int x = 0;
                if (x < 10) {
                    x = x + 1;
                } else {
                    x = x + 9;
                }
                return x;
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg::optimize_program(&mut program).unwrap();
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
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        [-] store i32 0, ptr %alloca_5
        br label %cond0

        cond0:
        [-] %load_12 = load i32, ptr %alloca_5
        [-] %icmp_13 = icmp slt i32 %load_12, 10
        [+] %icmp_13 = icmp slt i32 0, 10
        br i1 %icmp_13, label %then1, label %alt2

        then1:
        [-] %load_15 = load i32, ptr %alloca_5
        [-] %Add_16 = add i32 %load_15, 1
        [-] store i32 %Add_16, ptr %alloca_5
        [+] %Add_16 = add i32 0, 1
        br label %final3

        alt2:
        [-] %load_19 = load i32, ptr %alloca_5
        [-] %Add_20 = add i32 %load_19, 9
        [-] store i32 %Add_20, ptr %alloca_5
        [+] %Add_20 = add i32 0, 9
        br label %final3

        final3:
        [-] %load_23 = load i32, ptr %alloca_5
        [-] store i32 %load_23, ptr %alloca_2
        [+] %phi_26 = phi i32 [%Add_16, %then1], [%Add_20, %alt2]
        br label %exit

        exit:
        [-] %load_3 = load i32, ptr %alloca_2
        [-] ret i32 %load_3
        [+] ret i32 %phi_26


        }
        "###);
    }

    #[test]
    fn test_mem2reg_loop() {
        let code = r#"
            int main() {
                int x = 0;
                while (x < 10) {
                    x = x + 1;
                }
                return x;
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg::optimize_program(&mut program).unwrap();
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
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        [-] store i32 0, ptr %alloca_5
        br label %cond0

        cond0:
        [-] %load_15 = load i32, ptr %alloca_5
        [-] %icmp_16 = icmp slt i32 %load_15, 10
        [+] %phi_21 = phi i32 [0, %entry], [%Add_12, %body1]
        [+] %icmp_16 = icmp slt i32 %phi_21, 10
        br i1 %icmp_16, label %body1, label %final2

        body1:
        [-] %load_11 = load i32, ptr %alloca_5
        [-] %Add_12 = add i32 %load_11, 1
        [-] store i32 %Add_12, ptr %alloca_5
        [+] %Add_12 = add i32 %phi_21, 1
        br label %cond0

        final2:
        [-] %load_18 = load i32, ptr %alloca_5
        [-] store i32 %load_18, ptr %alloca_2
        br label %exit

        exit:
        [-] %load_3 = load i32, ptr %alloca_2
        [-] ret i32 %load_3
        [+] ret i32 %phi_21


        }
        "###);
    }

    #[test]
    fn test_mem2reg_nested() {
        let code = r#"
            int main() {
                int x = 0;
                while (x < 10) {
                    x = x + 2;
                    if (x > 5) while (x < 8) x = x + 1;
                }
                return x;
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg::optimize_program(&mut program).unwrap();
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
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        [-] store i32 0, ptr %alloca_5
        br label %cond0

        cond0:
        [-] %load_36 = load i32, ptr %alloca_5
        [-] %icmp_37 = icmp slt i32 %load_36, 10
        [+] %phi_42 = phi i32 [0, %entry], [%phi_44, %final6]
        [+] %icmp_37 = icmp slt i32 %phi_42, 10
        br i1 %icmp_37, label %body1, label %final2

        body1:
        [-] %load_11 = load i32, ptr %alloca_5
        [-] %Add_12 = add i32 %load_11, 2
        [-] store i32 %Add_12, ptr %alloca_5
        [+] %Add_12 = add i32 %phi_42, 2
        br label %cond3

        final2:
        [-] %load_39 = load i32, ptr %alloca_5
        [-] store i32 %load_39, ptr %alloca_2
        br label %exit

        cond3:
        [-] %load_19 = load i32, ptr %alloca_5
        [-] %icmp_20 = icmp sgt i32 %load_19, 5
        [+] %icmp_20 = icmp sgt i32 %Add_12, 5
        br i1 %icmp_20, label %then4, label %alt5

        exit:
        [-] %load_3 = load i32, ptr %alloca_2
        [-] ret i32 %load_3
        [+] ret i32 %phi_42

        then4:
        br label %cond7

        alt5:
        br label %final6

        cond7:
        [-] %load_30 = load i32, ptr %alloca_5
        [-] %icmp_31 = icmp slt i32 %load_30, 8
        [+] %phi_43 = phi i32 [%Add_12, %then4], [%Add_27, %body8]
        [+] %icmp_31 = icmp slt i32 %phi_43, 8
        br i1 %icmp_31, label %body8, label %final9

        final6:
        [+] %phi_44 = phi i32 [%phi_43, %final9], [%Add_12, %alt5]
        br label %cond0

        body8:
        [-] %load_26 = load i32, ptr %alloca_5
        [-] %Add_27 = add i32 %load_26, 1
        [-] store i32 %Add_27, ptr %alloca_5
        [+] %Add_27 = add i32 %phi_43, 1
        br label %cond7

        final9:
        br label %final6


        }
        "###);
    }

    #[test]
    fn test_mem2reg_uninitialized() {
        let code = r#"
            int main() {
                int x;
                // "x" is not initialized in this case
                // We expect the compiler not to panic
                return x;
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg::optimize_program(&mut program).unwrap();
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
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        [-] %load_6 = load i32, ptr %alloca_5
        [-] store i32 %load_6, ptr %alloca_2
        br label %exit

        exit:
        [-] %load_3 = load i32, ptr %alloca_2
        [-] ret i32 %load_3
        [+] ret i32 0


        }
        "###);
    }

    #[test]
    fn test_mem2reg_dead_block() {
        let code = r#"
            int a = 7;
            int func() {
                int b = a;
                int a = 1;
                // When both routes return, "final3" block will still be created,
                // but will never be visited
                if (a == b) {
                    a = a + 1;
                    return 1;
                }
                else
                    return 0;
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
        @a = dso_local global i32 7
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
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @func() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        %load_6 = load i32, ptr @a
        [-] store i32 %load_6, ptr %alloca_5
        %alloca_8 = alloca i32
        [-] store i32 1, ptr %alloca_8
        br label %cond0

        cond0:
        [-] %load_15 = load i32, ptr %alloca_8
        [-] %load_16 = load i32, ptr %alloca_5
        [-] %icmp_17 = icmp eq i32 %load_15, %load_16
        [+] %icmp_17 = icmp eq i32 1, %load_6
        br i1 %icmp_17, label %then1, label %alt2

        then1:
        [-] %load_19 = load i32, ptr %alloca_8
        [-] %Add_20 = add i32 %load_19, 1
        [-] store i32 %Add_20, ptr %alloca_8
        [-] store i32 1, ptr %alloca_2
        [+] %Add_20 = add i32 1, 1
        br label %exit

        alt2:
        [-] store i32 0, ptr %alloca_2
        br label %exit

        exit:
        [-] %load_3 = load i32, ptr %alloca_2
        [-] ret i32 %load_3
        [+] %phi_28 = phi i32 [%Add_20, %then1], [1, %alt2]
        [+] %phi_27 = phi i32 [1, %then1], [0, %alt2]
        [+] ret i32 %phi_27


        }
        "###);
    }

    #[test]
    fn test_mem2reg_array() {
        let code = r#"
            int main() {
                int arr[1] = {8};
                f(arr);
                putarray(1, arr);
                return 0;
            }

            int f(int a[]) {
                a[0] = 1;
                return a[0];
            }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        mem2reg::optimize_program(&mut program).unwrap();
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
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @main() {
        entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca [1 x i32]
        call void @llvm.memset.p0.i32([1 x i32]* %alloca_5, i8 0, i32 4, i1 false)
        %getelementptr_7 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %getelementptr_8 = getelementptr i32, ptr %getelementptr_7, i32 0
        store i32 8, ptr %getelementptr_8
        %getelementptr_10 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %call_11 = call i32 @f(i32* %getelementptr_10)
        %getelementptr_12 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        call void @putarray(i32 1, i32* %getelementptr_12)
        [-] store i32 0, ptr %alloca_2
        br label %exit

        exit:
        [-] %load_3 = load i32, ptr %alloca_2
        [-] ret i32 %load_3
        [+] ret i32 0


        }
        define i32 @f(i32* %a) {
        entry:
        %alloca_18 = alloca i32
        %alloca_21 = alloca i32*
        [-] store i32* %a, ptr %alloca_21
        [-] %load_23 = load i32*, ptr %alloca_21
        [-] %getelementptr_24 = getelementptr i32, ptr %load_23, i32 0
        [+] %getelementptr_24 = getelementptr i32, ptr %a, i32 0
        store i32 1, ptr %getelementptr_24
        [-] %load_26 = load i32*, ptr %alloca_21
        [-] %getelementptr_27 = getelementptr i32, ptr %load_26, i32 0
        [+] %getelementptr_27 = getelementptr i32, ptr %a, i32 0
        %load_28 = load i32, ptr %getelementptr_27
        [-] store i32 %load_28, ptr %alloca_18
        br label %exit

        exit:
        [-] %load_19 = load i32, ptr %alloca_18
        [-] ret i32 %load_19
        [+] ret i32 %load_28


        }
        "###);
    }
}
