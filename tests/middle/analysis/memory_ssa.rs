#[cfg(test)]
pub mod tests_memory_ssa {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            analysis::{effect_analysis::EffectAnalysis, memory_ssa::MemorySSA},
            irgen::gen,
            transform::{
                block_fuse, constant_fold, dead_code_elim, inst_combine, mem2reg, simple_gvn,
                unreachable_block_elim,
            },
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
        ; 3 = MemoryDef(0)
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
        ; MemoryUse(3)
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
        ; 2 = MemoryDef(0)
        store i32 3, ptr @b
        br label %exit

        exit:
        ret void

        MemorySSA for function: main
        entry:
        ; 3 (liveOnEntry)
        ; 4 = MemoryDef(3)
        store i32 2, ptr @a
        ; MemoryUse(4)
        ; 5 = MemoryDef(4)
        call void @f()
        ; MemoryUse(5)
        %load_13 = load i32, ptr @a
        br label %exit

        exit:
        ret i32 %load_13

        "###);
    }

    #[test]
    fn test_large() {
        let code = r#"
        #include "../../lib/sylib.h"

        int b = 5;
        int c[4] = {6, 7, 8, 9};

        int main()
        {
            int a;
            a = 1;
            {
                int a;
                a = 2;
                {
                    a = 3;
                    putint(a);
                }
                putint(a);
            }
            putint(a); putch(10);

            while (a < 5) {
                int a = 0;
                a = a + 1;
                if (a)
                    break;
            }
            putint(a); putch(10);

            {
                {
                    {
                        {}
                    }
                    c[2] = 1;
                    {
                        int c[2][8] = {{0, 9}, 8, 3};
                    }
                }
            }

            {
                int b = 2;
                if (c[2]) {
                    int c[7][1][5] = {{}, {}, {2, 1, 8}, {{}}};
                    putint(c[b][0][0]);
                    putint(c[b][0][1]);
                    putint(c[b][0][2]);
                }
            }
            putch(10);

            putint(b); putch(10);
            putint(c[0]); putint(c[1]); putint(c[2]); putint(c[3]); putch(10);
            return 0;
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        simple_gvn::optimize_program(&mut program).unwrap();
        unreachable_block_elim::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let memory_ssa = MemorySSA::new(&program, &effect_analysis);
        assert_snapshot!(memory_ssa.dump(), @r###"
        MemorySSA for function: main
        cond7:
        ; 0 (liveOnEntry)
        call void @putint(i32 3)
        call void @putint(i32 3)
        call void @putint(i32 1)
        call void @putch(i32 10)
        call void @putint(i32 1)
        call void @putch(i32 10)
        %getelementptr_43 = getelementptr [4 x i32], ptr @c, i32 0, i32 2
        ; 5 = MemoryDef(0)
        store i32 1, ptr %getelementptr_43
        %alloca_45 = alloca [2 x [8 x i32]]
        ; 6 = MemoryDef(0)
        call void @llvm.memset.p0.i32([2 x [8 x i32]]* %alloca_45, i8 0, i32 64, i1 false)
        %getelementptr_136 = getelementptr [2 x [8 x i32]], ptr %alloca_45, i32 0, i32 0, i32 0
        ; 7 = MemoryDef(6)
        store i32 0, ptr %getelementptr_136
        %Add_139 = add i32 0, 1
        %getelementptr_140 = getelementptr [2 x [8 x i32]], ptr %alloca_45, i32 0, i32 0, i32 %Add_139
        ; 8 = MemoryDef(7)
        store i32 9, ptr %getelementptr_140
        %getelementptr_144 = getelementptr [2 x [8 x i32]], ptr %alloca_45, i32 0, i32 %Add_139, i32 0
        ; 9 = MemoryDef(8)
        store i32 8, ptr %getelementptr_144
        %getelementptr_148 = getelementptr [2 x [8 x i32]], ptr %alloca_45, i32 0, i32 %Add_139, i32 %Add_139
        ; 10 = MemoryDef(9)
        store i32 3, ptr %getelementptr_148
        ; MemoryUse(5)
        %load_68 = load i32, ptr %getelementptr_43
        %icmp_69 = icmp ne i32 %load_68, 0
        br i1 %icmp_69, label %then8, label %alt9

        alt9:
        br label %exit

        exit:
        ; 1 = MemoryPhi([15, then8], [0, alt9])
        call void @putch(i32 10)
        ; MemoryUse(0)
        %load_113 = load i32, ptr @b
        call void @putint(i32 %load_113)
        call void @putch(i32 10)
        %getelementptr_116 = getelementptr [4 x i32], ptr @c, i32 0, i32 0
        ; MemoryUse(0)
        %load_117 = load i32, ptr %getelementptr_116
        call void @putint(i32 %load_117)
        %getelementptr_119 = getelementptr [4 x i32], ptr @c, i32 0, i32 1
        ; MemoryUse(0)
        %load_120 = load i32, ptr %getelementptr_119
        call void @putint(i32 %load_120)
        ; MemoryUse(5)
        %load_123 = load i32, ptr %getelementptr_43
        call void @putint(i32 %load_123)
        %getelementptr_125 = getelementptr [4 x i32], ptr @c, i32 0, i32 3
        ; MemoryUse(0)
        %load_126 = load i32, ptr %getelementptr_125
        call void @putint(i32 %load_126)
        call void @putch(i32 10)
        ret i32 0

        then8:
        %alloca_71 = alloca [7 x [1 x [5 x i32]]]
        ; 12 = MemoryDef(0)
        call void @llvm.memset.p0.i32([7 x [1 x [5 x i32]]]* %alloca_71, i8 0, i32 140, i1 false)
        %Add_157 = add i32 0, 2
        %getelementptr_164 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 %Add_157, i32 0, i32 0
        ; 13 = MemoryDef(12)
        store i32 2, ptr %getelementptr_164
        %getelementptr_168 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 %Add_157, i32 0, i32 %Add_139
        ; 14 = MemoryDef(13)
        store i32 1, ptr %getelementptr_168
        %getelementptr_170 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 %Add_157, i32 0, i32 %Add_157
        ; 15 = MemoryDef(14)
        store i32 8, ptr %getelementptr_170
        %getelementptr_182 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 2, i32 0, i32 0
        ; MemoryUse(15)
        %load_96 = load i32, ptr %getelementptr_182
        call void @putint(i32 %load_96)
        %getelementptr_186 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 2, i32 0, i32 1
        ; MemoryUse(15)
        %load_102 = load i32, ptr %getelementptr_186
        call void @putint(i32 %load_102)
        %getelementptr_190 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 2, i32 0, i32 2
        ; MemoryUse(15)
        %load_108 = load i32, ptr %getelementptr_190
        call void @putint(i32 %load_108)
        br label %exit

        "###);
    }
}
