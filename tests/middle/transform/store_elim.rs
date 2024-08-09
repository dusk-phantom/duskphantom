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
                mem2reg, simple_gvn, store_elim, unreachable_block_elim,
            },
        },
        utils::diff::diff,
    };

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
        simple_gvn::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        store_elim::optimize_program(&mut program, &mut memory_ssa, &effect_analysis).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
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
        simple_gvn::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        load_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        store_elim::optimize_program(&mut program, &mut memory_ssa, &effect_analysis).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
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
        simple_gvn::optimize_program(&mut program).unwrap();
        unreachable_block_elim::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        load_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        store_elim::optimize_program(&mut program, &mut memory_ssa, &effect_analysis).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
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
        simple_gvn::optimize_program(&mut program).unwrap();
        unreachable_block_elim::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        load_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        store_elim::optimize_program(&mut program, &mut memory_ssa, &effect_analysis).unwrap();
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
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @main() {
        exit:
        [-] %getelementptr_24 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 0, i32 0
        [-] store i32 1, ptr %getelementptr_24
        [-] %load_10 = load i32, ptr %getelementptr_24
        [-] %getelementptr_28 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 %load_10, i32 0
        [+] %getelementptr_28 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 1, i32 0
        store i32 2, ptr %getelementptr_28
        %call_14 = call i32 @getint()
        [-] %getelementptr_30 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 2, i32 %call_14
        [-] store i32 3, ptr %getelementptr_30
        %getelementptr_32 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 1, i32 0
        %load_20 = load i32, ptr %getelementptr_32
        ret i32 %load_20


        }
        "###);
    }
}
