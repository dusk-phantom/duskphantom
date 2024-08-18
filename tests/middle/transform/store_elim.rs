#[cfg(test)]
pub mod tests_store_elim {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            analysis::{effect_analysis::EffectAnalysis, memory_ssa::MemorySSA},
            irgen::gen,
            transform::{
                block_fuse, constant_fold, dead_code_elim, func_inline, inst_combine, load_elim,
                load_store_elim, mem2reg, redundance_elim, store_elim,
            },
        },
        utils::diff::diff,
    };

    #[test]
    fn test_gvar_redundant_store() {
        let code = r#"
        int a;
        int f() {
            return 0;
        }
        int main() {
            a = 2;
            a = 3;
            a = 5;
            a = 8;
            f();
            putint(a);
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        redundance_elim::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        store_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        @a = dso_local global i32 0
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
        define i32 @f() {
        entry:
        br label %exit

        exit:
        ret i32 0


        }
        define i32 @main() {
        entry:
        [-] store i32 2, ptr @a
        [-] store i32 3, ptr @a
        [-] store i32 5, ptr @a
        store i32 8, ptr @a
        %load_17 = load i32, ptr @a
        call void @putint(i32 %load_17)
        br label %exit

        exit:
        ret i32 0


        }
        "###);
    }

    #[test]
    fn test_gvar_store() {
        let code = r#"
        int a;
        int main() {
            a = 2;
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        redundance_elim::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        store_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        @a = dso_local global i32 0
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
        [-] store i32 2, ptr @a
        br label %exit

        exit:
        ret i32 0


        }
        "###);
    }

    #[test]
    fn test_gvar_interleave() {
        let code = r#"
        int a;
        int main() {
            a = 2;
            a = a;
            a = a;
            a = a;
            a = a;
            return a;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        redundance_elim::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        load_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        store_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        @a = dso_local global i32 0
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
        [-] store i32 2, ptr @a
        [-] %load_6 = load i32, ptr @a
        [-] store i32 %load_6, ptr @a
        [-] %load_8 = load i32, ptr @a
        [-] store i32 %load_8, ptr @a
        [-] %load_10 = load i32, ptr @a
        [-] store i32 %load_10, ptr @a
        [-] %load_12 = load i32, ptr @a
        [-] store i32 %load_12, ptr @a
        [-] %load_14 = load i32, ptr @a
        br label %exit

        exit:
        [-] ret i32 %load_14
        [+] ret i32 2


        }
        "###);
    }

    #[test]
    fn test_gvar_func() {
        let code = r#"
        int a;
        int b;
        void set_b() {
            b = getint();
        }
        int io() {
            putint(0);
            return 3;
        }
        int main() {
            a = 2;
            a = a;
            a = io();
            b = 2;
            b = b;
            set_b();
            return a + b;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        func_inline::optimize_program(&mut program).unwrap();
        redundance_elim::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        load_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        store_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        @a = dso_local global i32 0
        @b = dso_local global i32 0
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
        exit:
        [-] store i32 2, ptr @a
        [-] %load_20 = load i32, ptr @a
        [-] store i32 %load_20, ptr @a
        call void @putint(i32 0)
        [-] store i32 3, ptr @a
        [-] store i32 2, ptr @b
        [-] %load_25 = load i32, ptr @b
        [-] store i32 %load_25, ptr @b
        %call_34 = call i32 @getint()
        [-] store i32 %call_34, ptr @b
        [-] %load_28 = load i32, ptr @a
        [-] %load_29 = load i32, ptr @b
        [-] %Add_30 = add i32 %load_28, %load_29
        [+] %Add_30 = add i32 3, %call_34
        ret i32 %Add_30


        }
        "###);
    }

    #[test]
    fn test_memset() {
        let code = r#"
        int main() {
            int a[3] = {};
            int x = 1;
            return a[x];
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        redundance_elim::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        load_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        store_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
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
        exit:
        [-] %alloca_5 = alloca [3 x i32]
        [-] call void @llvm.memset.p0.i32([3 x i32]* %alloca_5, i8 0, i32 12, i1 false)
        [-] %getelementptr_10 = getelementptr [3 x i32], ptr %alloca_5, i32 0, i32 1
        [-] %load_11 = load i32, ptr %getelementptr_10
        [-] ret i32 %load_11
        [+] ret i32 0


        }
        "###);
    }

    #[test]
    fn test_array() {
        let code = r#"
        int a[3][3];
        int main() {
            a[0][0] = 1;
            a[a[0][0]][0] = 2;
            a[2][getint()] = 3;
            return a[1][0];
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        func_inline::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        redundance_elim::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        // TODO this much of load_store_elim is because of consecutive load, it should not happen
        // we should put load_elim inside symbolic eval, and utilize GVN for alias analysis
        load_store_elim::optimize_program(&mut program).unwrap();
        redundance_elim::optimize_program(&mut program).unwrap();
        load_store_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        @a = dso_local global [3 x [3 x i32]] zeroinitializer
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
        exit:
        [-] %getelementptr_5 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 0
        [-] %getelementptr_6 = getelementptr [3 x i32], ptr %getelementptr_5, i32 0, i32 0
        [-] store i32 1, ptr %getelementptr_6
        [-] %load_10 = load i32, ptr %getelementptr_6
        [-] %getelementptr_11 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 %load_10
        [-] %getelementptr_12 = getelementptr [3 x i32], ptr %getelementptr_11, i32 0, i32 0
        [-] store i32 2, ptr %getelementptr_12
        %call_14 = call i32 @getint()
        [-] %getelementptr_15 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 2
        [-] %getelementptr_16 = getelementptr [3 x i32], ptr %getelementptr_15, i32 0, i32 %call_14
        [-] store i32 3, ptr %getelementptr_16
        [-] %getelementptr_18 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 1
        [-] %getelementptr_19 = getelementptr [3 x i32], ptr %getelementptr_18, i32 0, i32 0
        [-] %load_20 = load i32, ptr %getelementptr_19
        [-] ret i32 %load_20
        [+] ret i32 2


        }
        "###);
    }

    #[test]
    fn test_array_redundant_store() {
        let code = r#"
        int a[3][3];
        int main() {
            int x = getint();
            int y = getint();
            a[1][x] = 3;
            a[1][y] = 5;
            a[1][x] = 7;
            a[1][y] = a[1][x];
            putarray(9, a);
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
        load_store_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        @a = dso_local global [3 x [3 x i32]] zeroinitializer
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
        %call_6 = call i32 @getint()
        %call_9 = call i32 @getint()
        %getelementptr_12 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 1
        %getelementptr_13 = getelementptr [3 x i32], ptr %getelementptr_12, i32 0, i32 %call_6
        [-] store i32 3, ptr %getelementptr_13
        %getelementptr_17 = getelementptr [3 x i32], ptr %getelementptr_12, i32 0, i32 %call_9
        [-] store i32 5, ptr %getelementptr_17
        store i32 7, ptr %getelementptr_13
        [-] %load_29 = load i32, ptr %getelementptr_13
        [-] store i32 %load_29, ptr %getelementptr_17
        [+] store i32 7, ptr %getelementptr_17
        %getelementptr_31 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 0
        call void @putarray(i32 9, [3 x i32]* %getelementptr_31)
        br label %exit

        exit:
        ret i32 0


        }
        "###);
    }
}
