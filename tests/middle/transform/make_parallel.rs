#[cfg(test)]
pub mod tests_make_parallel {
    use std::collections::HashMap;

    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            analysis::loop_tools::{self, LoopForest},
            ir::FunPtr,
            irgen::gen,
            transform::{
                dead_code_elim, inst_combine, loop_simplify::LoopSimplifier, make_parallel,
                mem2reg, redundance_elim, sink_code,
            },
        },
        utils::diff::diff,
    };

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
        let mut func_loop_map = program
            .module
            .functions
            .iter_mut()
            .filter_map(|func| {
                loop_tools::LoopForest::make_forest(*func).map(|forest| (*func, forest))
            })
            .collect::<HashMap<FunPtr, LoopForest>>();
        for (_, forest) in func_loop_map.iter_mut() {
            LoopSimplifier::new(&mut program.mem_pool)
                .run(forest)
                .unwrap();
        }
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        for (_, forest) in func_loop_map.iter_mut() {
            make_parallel::optimize_program(&mut program, forest).unwrap();
        }
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
        [+] %call_30 = call i32 @thrd_create(i32 4)
        [+] %Mul_32 = mul i32 %call_30, 8
        [+] %SDiv_33 = sdiv i32 %Mul_32, 5
        [+] %Add_35 = add i32 %Mul_32, 8
        [+] %SDiv_36 = sdiv i32 %Add_35, 5

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
        let mut func_loop_map = program
            .module
            .functions
            .iter_mut()
            .filter_map(|func| {
                loop_tools::LoopForest::make_forest(*func).map(|forest| (*func, forest))
            })
            .collect::<HashMap<FunPtr, LoopForest>>();
        for (_, forest) in func_loop_map.iter_mut() {
            LoopSimplifier::new(&mut program.mem_pool)
                .run(forest)
                .unwrap();
        }
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        for (_, forest) in func_loop_map.iter_mut() {
            make_parallel::optimize_program(&mut program, forest).unwrap();
        }
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
        [+] %call_36 = call i32 @thrd_create(i32 4)
        [+] %Mul_38 = mul i32 %call_36, 8
        [+] %SDiv_39 = sdiv i32 %Mul_38, 5
        [+] %Add_41 = add i32 %Mul_38, 8
        [+] %SDiv_42 = sdiv i32 %Add_41, 5

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
        let mut func_loop_map = program
            .module
            .functions
            .iter_mut()
            .filter_map(|func| {
                loop_tools::LoopForest::make_forest(*func).map(|forest| (*func, forest))
            })
            .collect::<HashMap<FunPtr, LoopForest>>();
        for (_, forest) in func_loop_map.iter_mut() {
            LoopSimplifier::new(&mut program.mem_pool)
                .run(forest)
                .unwrap();
        }
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        for (_, forest) in func_loop_map.iter_mut() {
            make_parallel::optimize_program(&mut program, forest).unwrap();
        }
        inst_combine::optimize_program(&mut program).unwrap();
        sink_code::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
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
        let mut func_loop_map = program
            .module
            .functions
            .iter_mut()
            .filter_map(|func| {
                loop_tools::LoopForest::make_forest(*func).map(|forest| (*func, forest))
            })
            .collect::<HashMap<FunPtr, LoopForest>>();
        for (_, forest) in func_loop_map.iter_mut() {
            LoopSimplifier::new(&mut program.mem_pool)
                .run(forest)
                .unwrap();
        }
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        for (_, forest) in func_loop_map.iter_mut() {
            make_parallel::optimize_program(&mut program, forest).unwrap();
        }
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
        br label %cond3
        [+] %call_51 = call i32 @thrd_create(i32 4)
        [+] %Mul_53 = mul i32 %call_51, 8
        [+] %SDiv_54 = sdiv i32 %Mul_53, 5
        [+] %Add_56 = add i32 %Mul_53, 8
        [+] %SDiv_57 = sdiv i32 %Add_56, 5

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
        let mut func_loop_map = program
            .module
            .functions
            .iter_mut()
            .filter_map(|func| {
                loop_tools::LoopForest::make_forest(*func).map(|forest| (*func, forest))
            })
            .collect::<HashMap<FunPtr, LoopForest>>();
        for (_, forest) in func_loop_map.iter_mut() {
            LoopSimplifier::new(&mut program.mem_pool)
                .run(forest)
                .unwrap();
        }
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        for (_, forest) in func_loop_map.iter_mut() {
            make_parallel::optimize_program(&mut program, forest).unwrap();
        }
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
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
                    B[i] = A[i] + 1;
                }
                
                // This loop will be parallelized too
                while (j < 99) {
                    j = j + 2;
                    B[i] = A[i] + 2;
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
        let mut func_loop_map = program
            .module
            .functions
            .iter_mut()
            .filter_map(|func| {
                loop_tools::LoopForest::make_forest(*func).map(|forest| (*func, forest))
            })
            .collect::<HashMap<FunPtr, LoopForest>>();
        for (_, forest) in func_loop_map.iter_mut() {
            LoopSimplifier::new(&mut program.mem_pool)
                .run(forest)
                .unwrap();
        }
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        for (_, forest) in func_loop_map.iter_mut() {
            make_parallel::optimize_program(&mut program, forest).unwrap();
        }
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
        "###);
    }
}
