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
    fn test_use_ind_var() {
        let code = r#"
        int A[9];
        int main() {
            int i = 3;
            int x = getint();
            while (i < x) {
                A[i] = i;
                i = i + 6;
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
        %call_8 = call i32 @getint()
        [+] %call_30 = call i32 @thrd_create(i32 4)
        [+] %Sub_31 = sub i32 %call_8, 3
        [+] %Mul_32 = mul i32 %call_30, %Sub_31
        [+] %SDiv_33 = sdiv i32 %Mul_32, 5
        [+] %Add_34 = add i32 %SDiv_33, 3
        [+] %Add_35 = add i32 %Mul_32, %Sub_31
        [+] %SDiv_36 = sdiv i32 %Add_35, 5
        [+] %Add_37 = add i32 %SDiv_36, 3
        br label %cond0

        cond0:
        [-] %phi_29 = phi i32 [3, %entry], [%Add_19, %body1]
        [-] %icmp_24 = icmp slt i32 %phi_29, %call_8
        [-] br i1 %icmp_24, label %body1, label %final2
        [+] %phi_29 = phi i32 [%Add_34, %entry], [%Add_19, %body1]
        [+] %icmp_38 = icmp slt i32 %phi_29, %Add_37
        [+] br i1 %icmp_38, label %body1, label %final2

        body1:
        %getelementptr_15 = getelementptr [9 x i32], ptr @A, i32 0, i32 %phi_29
        store i32 %phi_29, ptr %getelementptr_15
        %Add_19 = add i32 %phi_29, 6
        br label %cond0

        final2:
        [+] call void @thrd_join()
        [+] %Add_45 = add i32 %call_8, -4
        [+] %SDiv_41 = sdiv i32 %Add_45, 6
        [+] %Add_42 = add i32 %SDiv_41, 1
        [+] %Mul_43 = mul i32 %Add_42, 6
        [+] %Add_44 = add i32 %Mul_43, 3
        br label %exit

        exit:
        [-] ret i32 %phi_29
        [+] ret i32 %Add_44


        }
        "###);
    }

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
        [-] ret i32 %phi_31
        [+] ret i32 8


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
        [-] ret i32 %phi_29
        [+] ret i32 8


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
        [-] ret i32 %phi_35
        [+] ret i32 8


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
}
