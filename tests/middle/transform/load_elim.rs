#[cfg(test)]
pub mod tests_load_elim {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            analysis::{effect_analysis::EffectAnalysis, memory_ssa::MemorySSA},
            irgen::gen,
            transform::{dead_code_elim, load_elim, mem2reg, simple_gvn},
        },
        utils::diff::diff,
    };

    #[test]
    fn test_trivial_load() {
        let code = r#"
        int a;
        int main() {
            a = 2;
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
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define i32 @main() {
        entry:
        store i32 2, ptr @a
        [-] %load_6 = load i32, ptr @a
        br label %exit

        exit:
        [-] ret i32 %load_6
        [+] ret i32 2


        }
        "###);
    }

    #[test]
    fn test_array_non_overlap() {
        let code = r#"
        int a[3][3];
        int main() {
            int x = getint();
            a[1][1] = 1;
            a[x][2] = 2;
            return a[1][1];
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
        entry:
        %call_6 = call i32 @getint()
        %getelementptr_8 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 1
        %getelementptr_9 = getelementptr [3 x i32], ptr %getelementptr_8, i32 0, i32 1
        store i32 1, ptr %getelementptr_9
        %getelementptr_12 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 %call_6
        %getelementptr_13 = getelementptr [3 x i32], ptr %getelementptr_12, i32 0, i32 2
        store i32 2, ptr %getelementptr_13
        [-] %load_17 = load i32, ptr %getelementptr_9
        br label %exit

        exit:
        [-] ret i32 %load_17
        [+] ret i32 1


        }
        "###);
    }

    #[test]
    fn test_array_overlap() {
        let code = r#"
        int a[3][3];
        int main() {
            int x = getint();
            a[1][1] = 1;
            a[x][x] = 2;
            return a[1][1];
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
        entry:
        %call_6 = call i32 @getint()
        %getelementptr_8 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 1
        %getelementptr_9 = getelementptr [3 x i32], ptr %getelementptr_8, i32 0, i32 1
        store i32 1, ptr %getelementptr_9
        %getelementptr_13 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 %call_6
        %getelementptr_14 = getelementptr [3 x i32], ptr %getelementptr_13, i32 0, i32 %call_6
        store i32 2, ptr %getelementptr_14
        %load_18 = load i32, ptr %getelementptr_9
        br label %exit

        exit:
        ret i32 %load_18


        }
        "###);
    }
}
