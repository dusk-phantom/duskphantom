#[cfg(test)]
pub mod tests_memory_ssa {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            analysis::{effect_analysis::EffectAnalysis, memory_ssa::MemorySSA},
            irgen::gen,
            transform::{dead_code_elim, inst_combine, mem2reg, simple_gvn},
        },
    };

    #[test]
    fn test_trivial() {
        let code = r#"
        int a = 9;
        int main() {
            a = 6;
            return a;
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let memory_ssa = MemorySSA::new(&program, &effect_analysis);
        assert_snapshot!(memory_ssa.dump(), @r###"
        MemorySSA for function: main
        entry:
        ; 0 (liveOnEntry)
        ; 1 = MemoryDef(0)
        store i32 6, ptr @a
        ; MemoryUse(1)
        %load_6 = load i32, ptr @a
        br label %exit

        exit:
        ret i32 %load_6

        "###);
    }

    #[test]
    fn test_alt() {
        let code = r#"
        int a = 9;
        int main() {
            if (a < 6) {
                a = 8;
            }
            return a;
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let memory_ssa = MemorySSA::new(&program, &effect_analysis);
        assert_snapshot!(memory_ssa.dump(), @r###"
        MemorySSA for function: main
        entry:
        ; 0 (liveOnEntry)
        br label %cond0

        cond0:
        ; MemoryUse(0)
        %load_10 = load i32, ptr @a
        %icmp_11 = icmp slt i32 %load_10, 6
        br i1 %icmp_11, label %then1, label %alt2

        alt2:
        br label %final3

        final3:
        ; 1 = MemoryPhi([3, then1], [0, alt2])
        ; MemoryUse(1)
        %load_16 = load i32, ptr @a
        br label %exit

        exit:
        ret i32 %load_16

        then1:
        ; 3 = MemoryDef(0)
        store i32 8, ptr @a
        br label %final3

        "###);
    }

    #[test]
    fn test_array_non_overlap() {
        let code = r#"
        int a[3][3];
        int b = 2;
        int main() {
            a[2][2] = 2;
            a[b][1] = 1;
            return a[2][2];
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        simple_gvn::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let memory_ssa = MemorySSA::new(&program, &effect_analysis);
        assert_snapshot!(memory_ssa.dump(), @r###"
        MemorySSA for function: main
        entry:
        ; 0 (liveOnEntry)
        %getelementptr_18 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 2, i32 2
        ; 1 = MemoryDef(0)
        store i32 2, ptr %getelementptr_18
        ; MemoryUse(0)
        %load_8 = load i32, ptr @b
        %getelementptr_20 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 %load_8, i32 1
        ; 3 = MemoryDef(1)
        store i32 1, ptr %getelementptr_20
        ; MemoryUse(1)
        %load_14 = load i32, ptr %getelementptr_18
        br label %exit

        exit:
        ret i32 %load_14

        "###);
    }

    #[test]
    fn test_array_overlap() {
        let code = r#"
        int a[3][3];
        int b = 2;
        int main() {
            a[2][2] = 2;
            a[b][2] = 1;
            return a[2][2];
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        simple_gvn::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let memory_ssa = MemorySSA::new(&program, &effect_analysis);
        assert_snapshot!(memory_ssa.dump(), @r###"
        MemorySSA for function: main
        entry:
        ; 0 (liveOnEntry)
        %getelementptr_18 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 2, i32 2
        ; 1 = MemoryDef(0)
        store i32 2, ptr %getelementptr_18
        ; MemoryUse(0)
        %load_8 = load i32, ptr @b
        %getelementptr_20 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 %load_8, i32 2
        ; 3 = MemoryDef(1)
        store i32 1, ptr %getelementptr_20
        ; MemoryUse(3)
        %load_14 = load i32, ptr %getelementptr_18
        br label %exit

        exit:
        ret i32 %load_14

        "###);
    }

    #[test]
    fn test_function_non_overlap() {
        let code = r#"
        int a = 1;
        int b = 2;
        void f() {
            b = 3;
        }
        int main() {
            a = 2;
            f();
            return a;
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        simple_gvn::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let memory_ssa = MemorySSA::new(&program, &effect_analysis);
        assert_snapshot!(memory_ssa.dump(), @r###"
        MemorySSA for function: f
        entry:
        ; 0 (liveOnEntry)
        ; 1 = MemoryDef(0)
        store i32 3, ptr @b
        br label %exit

        exit:
        ret void

        MemorySSA for function: main
        entry:
        ; 2 (liveOnEntry)
        ; 3 = MemoryDef(2)
        store i32 2, ptr @a
        ; 4 = MemoryDef(3)
        call void @f()
        ; MemoryUse(4)
        %load_12 = load i32, ptr @a
        br label %exit

        exit:
        ret i32 %load_12

        "###);
    }

    #[test]
    fn test_function_overlap() {
        let code = r#"
        int a = 1;
        int b = 2;
        void f() {
            a = 9;
            b = 3;
        }
        int main() {
            a = 2;
            f();
            return a;
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        simple_gvn::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let memory_ssa = MemorySSA::new(&program, &effect_analysis);
        assert_snapshot!(memory_ssa.dump(), @r###"
        MemorySSA for function: f
        entry:
        ; 0 (liveOnEntry)
        ; 1 = MemoryDef(0)
        store i32 9, ptr @a
        ; 2 = MemoryDef(1)
        store i32 3, ptr @b
        br label %exit

        exit:
        ret void

        MemorySSA for function: main
        entry:
        ; 3 (liveOnEntry)
        ; 4 = MemoryDef(3)
        store i32 2, ptr @a
        ; 5 = MemoryDef(4)
        call void @f()
        ; MemoryUse(5)
        %load_13 = load i32, ptr @a
        br label %exit

        exit:
        ret i32 %load_13

        "###);
    }
}
