#[cfg(test)]
pub mod tests_make_parallel {

    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{
                dead_code_elim, inst_combine, make_parallel, mem2reg, redundance_elim, sink_code,
            },
        },
        utils::diff::diff,
    };

    #[test]
    fn test_stack_ref() {
        let code = r#"
        int main() {
            int A[9];
            int B[9];
            int i = 0;
            getarray(A);
            while (i < 8) {
                i = i + 1;
                B[i] = A[i];
            }
            return i;
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
        make_parallel::optimize_program::<5>(&mut program).unwrap();
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
        define i32 @main() {
        entry:
        %alloca_5 = alloca [9 x i32]
        %alloca_6 = alloca [9 x i32]
        %getelementptr_9 = getelementptr [9 x i32], ptr %alloca_5, i32 0, i32 0
        %call_10 = call i32 @getarray(i32* %getelementptr_9)
        [+] %getelementptr_32 = getelementptr [9 x i32], ptr %alloca_6, i32 0
        [+] %getelementptr_33 = getelementptr [9 x i32], ptr %alloca_5, i32 0
        [+] %call_34 = call i32 @thrd_create(i32 4)
        [+] %Mul_36 = mul i32 %call_34, 8
        [+] %SDiv_37 = sdiv i32 %Mul_36, 5
        [+] %Add_39 = add i32 %Mul_36, 8
        [+] %SDiv_40 = sdiv i32 %Add_39, 5
        br label %cond0

        cond0:
        [-] %phi_31 = phi i32 [0, %entry], [%Add_16, %body1]
        [-] %icmp_26 = icmp slt i32 %phi_31, 8
        [-] br i1 %icmp_26, label %body1, label %final2
        [+] %phi_31 = phi i32 [%SDiv_37, %entry], [%Add_16, %body1]
        [+] %icmp_42 = icmp slt i32 %phi_31, %SDiv_40
        [+] br i1 %icmp_42, label %body1, label %final2

        body1:
        %Add_16 = add i32 %phi_31, 1
        [-] %getelementptr_19 = getelementptr [9 x i32], ptr %alloca_5, i32 0, i32 %Add_16
        [-] %getelementptr_21 = getelementptr [9 x i32], ptr %alloca_6, i32 0, i32 %Add_16
        [+] %getelementptr_19 = getelementptr [9 x i32], ptr %getelementptr_33, i32 0, i32 %Add_16
        [+] %getelementptr_21 = getelementptr [9 x i32], ptr %getelementptr_32, i32 0, i32 %Add_16
        %load_22 = load i32, ptr %getelementptr_19
        store i32 %load_22, ptr %getelementptr_21
        br label %cond0

        final2:
        [+] call void @thrd_join()
        br label %exit

        exit:
        ret i32 %phi_31


        }
        "###);
    }

    #[test]
    fn test_basic() {
        let code = r#"
        int A[9];
        int B[9];
        int main() {
            int i = 0;
            getarray(A);
            while (i < 8) {
                i = i + 1;
                B[i] = A[i];
            }
            return i;
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
        make_parallel::optimize_program::<5>(&mut program).unwrap();
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
        %getelementptr_7 = getelementptr [9 x i32], ptr @A, i32 0, i32 0
        %call_8 = call i32 @getarray(i32* %getelementptr_7)
        [+] %call_30 = call i32 @thrd_create(i32 4)
        [+] %Mul_32 = mul i32 %call_30, 8
        [+] %SDiv_33 = sdiv i32 %Mul_32, 5
        [+] %Add_35 = add i32 %Mul_32, 8
        [+] %SDiv_36 = sdiv i32 %Add_35, 5
        br label %cond0

        cond0:
        [-] %phi_29 = phi i32 [0, %entry], [%Add_14, %body1]
        [-] %icmp_24 = icmp slt i32 %phi_29, 8
        [-] br i1 %icmp_24, label %body1, label %final2
        [+] %phi_29 = phi i32 [%SDiv_33, %entry], [%Add_14, %body1]
        [+] %icmp_38 = icmp slt i32 %phi_29, %SDiv_36
        [+] br i1 %icmp_38, label %body1, label %final2

        body1:
        %Add_14 = add i32 %phi_29, 1
        %getelementptr_17 = getelementptr [9 x i32], ptr @A, i32 0, i32 %Add_14
        %getelementptr_19 = getelementptr [9 x i32], ptr @B, i32 0, i32 %Add_14
        %load_20 = load i32, ptr %getelementptr_17
        store i32 %load_20, ptr %getelementptr_19
        br label %cond0

        final2:
        [+] call void @thrd_join()
        br label %exit

        exit:
        ret i32 %phi_29


        }
        "###);
    }

    #[test]
    fn test_multiple_same_address() {
        let code = r#"
        int A[9];
        int B[9];
        int main() {
            int i = 0;
            getarray(A);
            while (i < 8) {
                i = i + 1;
                B[i] = A[i];
                B[i] = A[i];
            }
            return i;
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
        make_parallel::optimize_program::<5>(&mut program).unwrap();
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
        %getelementptr_7 = getelementptr [9 x i32], ptr @A, i32 0, i32 0
        %call_8 = call i32 @getarray(i32* %getelementptr_7)
        [+] %call_36 = call i32 @thrd_create(i32 4)
        [+] %Mul_38 = mul i32 %call_36, 8
        [+] %SDiv_39 = sdiv i32 %Mul_38, 5
        [+] %Add_41 = add i32 %Mul_38, 8
        [+] %SDiv_42 = sdiv i32 %Add_41, 5
        br label %cond0

        cond0:
        [-] %phi_35 = phi i32 [0, %entry], [%Add_14, %body1]
        [-] %icmp_30 = icmp slt i32 %phi_35, 8
        [-] br i1 %icmp_30, label %body1, label %final2
        [+] %phi_35 = phi i32 [%SDiv_39, %entry], [%Add_14, %body1]
        [+] %icmp_44 = icmp slt i32 %phi_35, %SDiv_42
        [+] br i1 %icmp_44, label %body1, label %final2

        body1:
        %Add_14 = add i32 %phi_35, 1
        %getelementptr_17 = getelementptr [9 x i32], ptr @A, i32 0, i32 %Add_14
        %getelementptr_19 = getelementptr [9 x i32], ptr @B, i32 0, i32 %Add_14
        %load_20 = load i32, ptr %getelementptr_17
        store i32 %load_20, ptr %getelementptr_19
        store i32 %load_20, ptr %getelementptr_19
        br label %cond0

        final2:
        [+] call void @thrd_join()
        br label %exit

        exit:
        ret i32 %phi_35


        }
        "###);
    }

    #[test]
    fn test_conflict_address() {
        let code = r#"
        int A[9];
        int B[9];
        int main() {
            int i = 0;
            getarray(A);
            while (i < 8) {
                i = i + 1;
                A[0] = 8;
            }
            return i;
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
        make_parallel::optimize_program::<5>(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        sink_code::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
        @A = dso_local global [9 x i32] zeroinitializer
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
        %getelementptr_7 = getelementptr [9 x i32], ptr @A, i32 0, i32 0
        %call_8 = call i32 @getarray(i32* %getelementptr_7)
        br label %cond0

        cond0:
        %phi_25 = phi i32 [0, %entry], [%Add_14, %body1]
        %icmp_20 = icmp slt i32 %phi_25, 8
        br i1 %icmp_20, label %body1, label %final2

        body1:
        %Add_14 = add i32 %phi_25, 1
        store i32 8, ptr %getelementptr_7
        br label %cond0

        final2:
        br label %exit

        exit:
        ret i32 %phi_25


        }
        "###);
    }

    #[test]
    fn test_nested_loop() {
        let code = r#"
        int A[9];
        int B[9];
        int main() {
            int i = 0;
            getarray(A);

            // This loop can't be parallelized because inner loop conflicts
            while (i < 8) {
                i = i + 1;
                B[i] = A[i];
                int j = 0;

                // This loop will be parallelized
                while (j < 8) {
                    j = j + 1;
                    B[j] = A[j];
                }
            }
            return i;
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
        make_parallel::optimize_program::<5>(&mut program).unwrap();
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
        %getelementptr_7 = getelementptr [9 x i32], ptr @A, i32 0, i32 0
        %call_8 = call i32 @getarray(i32* %getelementptr_7)
        br label %cond0

        cond0:
        %phi_48 = phi i32 [0, %entry], [%Add_14, %final5]
        %icmp_43 = icmp slt i32 %phi_48, 8
        br i1 %icmp_43, label %body1, label %final2

        body1:
        %Add_14 = add i32 %phi_48, 1
        %getelementptr_17 = getelementptr [9 x i32], ptr @A, i32 0, i32 %Add_14
        %getelementptr_19 = getelementptr [9 x i32], ptr @B, i32 0, i32 %Add_14
        %load_20 = load i32, ptr %getelementptr_17
        store i32 %load_20, ptr %getelementptr_19
        [+] %call_51 = call i32 @thrd_create(i32 4)
        [+] %Mul_53 = mul i32 %call_51, 8
        [+] %SDiv_54 = sdiv i32 %Mul_53, 5
        [+] %Add_56 = add i32 %Mul_53, 8
        [+] %SDiv_57 = sdiv i32 %Add_56, 5
        br label %cond3

        final2:
        br label %exit

        cond3:
        [-] %phi_50 = phi i32 [0, %body1], [%Add_29, %body4]
        [-] %icmp_39 = icmp slt i32 %phi_50, 8
        [-] br i1 %icmp_39, label %body4, label %final5
        [+] %phi_50 = phi i32 [%SDiv_54, %body1], [%Add_29, %body4]
        [+] %icmp_59 = icmp slt i32 %phi_50, %SDiv_57
        [+] br i1 %icmp_59, label %body4, label %final5

        exit:
        ret i32 %phi_48

        body4:
        %Add_29 = add i32 %phi_50, 1
        %getelementptr_32 = getelementptr [9 x i32], ptr @A, i32 0, i32 %Add_29
        %getelementptr_34 = getelementptr [9 x i32], ptr @B, i32 0, i32 %Add_29
        %load_35 = load i32, ptr %getelementptr_32
        store i32 %load_35, ptr %getelementptr_34
        br label %cond3

        final5:
        [+] call void @thrd_join()
        br label %cond0


        }
        "###);
    }

    #[test]
    fn test_nested_loop_2() {
        let code = r#"
        int A[9];
        int B[9];
        int main() {
            int i = 0;
            getarray(A);

            // This loop will be parallelized
            while (i < 8) {
                i = i + 1;
                B[i] = A[i];
                int j = 0;
                
                // This loop will not be parallelized because it's parent loop is parallelized
                while (j < 8) {
                    j = j + 1;
                    B[i] = A[i] + 3;
                }
            }
            return i;
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
        make_parallel::optimize_program::<5>(&mut program).unwrap();
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
        %getelementptr_7 = getelementptr [9 x i32], ptr @A, i32 0, i32 0
        %call_8 = call i32 @getarray(i32* %getelementptr_7)
        [+] %call_52 = call i32 @thrd_create(i32 4)
        [+] %Mul_54 = mul i32 %call_52, 8
        [+] %SDiv_55 = sdiv i32 %Mul_54, 5
        [+] %Add_57 = add i32 %Mul_54, 8
        [+] %SDiv_58 = sdiv i32 %Add_57, 5
        br label %cond0

        cond0:
        [-] %phi_49 = phi i32 [0, %entry], [%Add_14, %final5]
        [-] %icmp_44 = icmp slt i32 %phi_49, 8
        [-] br i1 %icmp_44, label %body1, label %final2
        [+] %phi_49 = phi i32 [%SDiv_55, %entry], [%Add_14, %final5]
        [+] %icmp_60 = icmp slt i32 %phi_49, %SDiv_58
        [+] br i1 %icmp_60, label %body1, label %final2

        body1:
        %Add_14 = add i32 %phi_49, 1
        %getelementptr_17 = getelementptr [9 x i32], ptr @A, i32 0, i32 %Add_14
        %getelementptr_19 = getelementptr [9 x i32], ptr @B, i32 0, i32 %Add_14
        %load_20 = load i32, ptr %getelementptr_17
        store i32 %load_20, ptr %getelementptr_19
        br label %cond3

        final2:
        [+] call void @thrd_join()
        br label %exit

        cond3:
        %phi_51 = phi i32 [0, %body1], [%Add_29, %body4]
        %icmp_40 = icmp slt i32 %phi_51, 8
        br i1 %icmp_40, label %body4, label %final5

        exit:
        ret i32 %phi_49

        body4:
        %Add_29 = add i32 %phi_51, 1
        %Add_34 = add i32 %load_20, 3
        store i32 %Add_34, ptr %getelementptr_19
        br label %cond3

        final5:
        br label %cond0


        }
        "###);
    }

    #[test]
    fn test_nested_loop_3() {
        let code = r#"
        int A[9];
        int B[9];
        int main() {
            int i = 0;
            getarray(A);

            // This loop won't be parallelized
            while (i < 8) {
                i = i + 1;
                B[0] = A[0];
                int j = 0;
                
                // This loop will be parallelized
                while (j < 8) {
                    j = j + 1;
                    B[j] = A[j] + 1;
                }
                
                // This loop will be parallelized too
                while (j < 99) {
                    j = j + 2;
                    B[j] = A[j] + 2;
                }
            }
            return i;
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
        make_parallel::optimize_program::<5>(&mut program).unwrap();
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
        %getelementptr_7 = getelementptr [9 x i32], ptr @A, i32 0, i32 0
        %call_8 = call i32 @getarray(i32* %getelementptr_7)
        br label %cond0

        cond0:
        %phi_65 = phi i32 [0, %entry], [%Add_14, %final8]
        %icmp_60 = icmp slt i32 %phi_65, 8
        br i1 %icmp_60, label %body1, label %final2

        body1:
        %Add_14 = add i32 %phi_65, 1
        %getelementptr_17 = getelementptr [9 x i32], ptr @B, i32 0, i32 0
        %load_18 = load i32, ptr %getelementptr_7
        store i32 %load_18, ptr %getelementptr_17
        [+] %call_69 = call i32 @thrd_create(i32 4)
        [+] %Mul_71 = mul i32 %call_69, 8
        [+] %SDiv_72 = sdiv i32 %Mul_71, 5
        [+] %Add_74 = add i32 %Mul_71, 8
        [+] %SDiv_75 = sdiv i32 %Add_74, 5
        br label %cond3

        final2:
        br label %exit

        cond3:
        [-] %phi_67 = phi i32 [0, %body1], [%Add_27, %body4]
        [-] %icmp_38 = icmp slt i32 %phi_67, 8
        [-] br i1 %icmp_38, label %body4, label %final5
        [+] %phi_67 = phi i32 [%SDiv_72, %body1], [%Add_27, %body4]
        [+] %icmp_77 = icmp slt i32 %phi_67, %SDiv_75
        [+] br i1 %icmp_77, label %body4, label %final5

        exit:
        ret i32 %phi_65

        body4:
        %Add_27 = add i32 %phi_67, 1
        %getelementptr_30 = getelementptr [9 x i32], ptr @A, i32 0, i32 %Add_27
        %load_31 = load i32, ptr %getelementptr_30
        %Add_32 = add i32 %load_31, 1
        %getelementptr_34 = getelementptr [9 x i32], ptr @B, i32 0, i32 %Add_27
        store i32 %Add_32, ptr %getelementptr_34
        br label %cond3

        final5:
        [+] call void @thrd_join()
        [+] %call_79 = call i32 @thrd_create(i32 4)
        [+] %Sub_80 = sub i32 99, %phi_67
        [+] %Mul_81 = mul i32 %call_79, %Sub_80
        [+] %SDiv_82 = sdiv i32 %Mul_81, 5
        [+] %Add_83 = add i32 %SDiv_82, %phi_67
        [+] %Add_84 = add i32 %Mul_81, %Sub_80
        [+] %SDiv_85 = sdiv i32 %Add_84, 5
        [+] %Add_86 = add i32 %SDiv_85, %phi_67
        br label %cond6

        cond6:
        [-] %phi_68 = phi i32 [%phi_67, %final5], [%Add_45, %body7]
        [-] %icmp_56 = icmp slt i32 %phi_68, 99
        [-] br i1 %icmp_56, label %body7, label %final8
        [+] %phi_68 = phi i32 [%Add_83, %final5], [%Add_45, %body7]
        [+] %icmp_87 = icmp slt i32 %phi_68, %Add_86
        [+] br i1 %icmp_87, label %body7, label %final8

        body7:
        %Add_45 = add i32 %phi_68, 2
        %getelementptr_48 = getelementptr [9 x i32], ptr @A, i32 0, i32 %Add_45
        %load_49 = load i32, ptr %getelementptr_48
        %Add_50 = add i32 %load_49, 2
        %getelementptr_52 = getelementptr [9 x i32], ptr @B, i32 0, i32 %Add_45
        store i32 %Add_50, ptr %getelementptr_52
        br label %cond6

        final8:
        [+] call void @thrd_join()
        br label %cond0


        }
        "###);
    }
}
