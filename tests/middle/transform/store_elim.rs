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
        declare void @putf()
        declare void @llvm.memset.p0.i32(i32* %p0, i8 %p1, i32 %p2, i1 %p3)
        define void @set_b() {
        exit:
        %call_3 = call i32 @getint()
        store i32 %call_3, ptr @b
        ret void


        }
        define i32 @io() {
        exit:
        call void @putint(i32 0)
        ret i32 3


        }
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
        %call_43 = call i32 @getint()
        [-] store i32 %call_43, ptr @b
        [-] %load_28 = load i32, ptr @a
        [-] %load_29 = load i32, ptr @b
        [-] %Add_30 = add i32 %load_28, %load_29
        [+] %Add_30 = add i32 3, %call_43
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
        [-] %alloca_5 = alloca [3 x i32]
        [-] call void @llvm.memset.p0.i32([3 x i32]* %alloca_5, i8 0, i32 12, i1 false)
        [-] %getelementptr_11 = getelementptr [3 x i32], ptr %alloca_5, i32 0, i32 1
        [-] %load_12 = load i32, ptr %getelementptr_11
        [-] ret i32 %load_12
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
        simple_gvn::optimize_program(&mut program).unwrap();
        unreachable_block_elim::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        for _ in 0..3 {
            let effect_analysis = EffectAnalysis::new(&program);
            let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
            load_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
            store_elim::optimize_program(&mut program, &mut memory_ssa, &effect_analysis).unwrap();
            constant_fold::optimize_program(&mut program).unwrap();
            inst_combine::optimize_program(&mut program).unwrap();
            simple_gvn::optimize_program(&mut program).unwrap();
            unreachable_block_elim::optimize_program(&mut program).unwrap();
            block_fuse::optimize_program(&mut program).unwrap();
        }
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
        [-] store i32 2, ptr %getelementptr_28
        %call_14 = call i32 @getint()
        [-] %getelementptr_30 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 2, i32 %call_14
        [-] store i32 3, ptr %getelementptr_30
        [-] %getelementptr_32 = getelementptr [3 x [3 x i32]], ptr @a, i32 0, i32 1, i32 0
        [-] %load_20 = load i32, ptr %getelementptr_32
        [-] ret i32 %load_20
        [+] ret i32 2


        }
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
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        for _ in 0..3 {
            let effect_analysis = EffectAnalysis::new(&program);
            let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
            load_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
            store_elim::optimize_program(&mut program, &mut memory_ssa, &effect_analysis).unwrap();
            constant_fold::optimize_program(&mut program).unwrap();
            inst_combine::optimize_program(&mut program).unwrap();
            simple_gvn::optimize_program(&mut program).unwrap();
            unreachable_block_elim::optimize_program(&mut program).unwrap();
            block_fuse::optimize_program(&mut program).unwrap();
        }
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        @b = dso_local global i32 5
        @c = dso_local global [4 x i32] [i32 6, i32 7, i32 8, i32 9]
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
        [-] cond7:
        [+] exit:
        call void @putint(i32 3)
        call void @putint(i32 3)
        call void @putint(i32 1)
        call void @putch(i32 10)
        call void @putint(i32 1)
        call void @putch(i32 10)
        [-] %getelementptr_43 = getelementptr [4 x i32], ptr @c, i32 0, i32 2
        [-] store i32 1, ptr %getelementptr_43
        [-] %alloca_45 = alloca [2 x [8 x i32]]
        [-] call void @llvm.memset.p0.i32([2 x [8 x i32]]* %alloca_45, i8 0, i32 64, i1 false)
        [-] %getelementptr_136 = getelementptr [2 x [8 x i32]], ptr %alloca_45, i32 0, i32 0, i32 0
        [-] store i32 0, ptr %getelementptr_136
        [-] %Add_139 = add i32 0, 1
        [-] %getelementptr_140 = getelementptr [2 x [8 x i32]], ptr %alloca_45, i32 0, i32 0, i32 %Add_139
        [-] store i32 9, ptr %getelementptr_140
        [-] %getelementptr_144 = getelementptr [2 x [8 x i32]], ptr %alloca_45, i32 0, i32 %Add_139, i32 0
        [-] store i32 8, ptr %getelementptr_144
        [-] %getelementptr_148 = getelementptr [2 x [8 x i32]], ptr %alloca_45, i32 0, i32 %Add_139, i32 %Add_139
        [-] store i32 3, ptr %getelementptr_148
        [-] %load_68 = load i32, ptr %getelementptr_43
        [-] %icmp_69 = icmp ne i32 %load_68, 0
        [-] br i1 %icmp_69, label %then8, label %alt9
        [-] 
        [-] then8:
        [-] %alloca_71 = alloca [7 x [1 x [5 x i32]]]
        [-] call void @llvm.memset.p0.i32([7 x [1 x [5 x i32]]]* %alloca_71, i8 0, i32 140, i1 false)
        [-] %Add_157 = add i32 0, 2
        [-] %getelementptr_164 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 %Add_157, i32 0, i32 0
        [-] store i32 2, ptr %getelementptr_164
        [-] %getelementptr_168 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 %Add_157, i32 0, i32 %Add_139
        [-] store i32 1, ptr %getelementptr_168
        [-] %getelementptr_170 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 %Add_157, i32 0, i32 %Add_157
        [-] store i32 8, ptr %getelementptr_170
        [-] %getelementptr_182 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 2, i32 0, i32 0
        [-] %load_96 = load i32, ptr %getelementptr_182
        [-] call void @putint(i32 %load_96)
        [-] %getelementptr_186 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 2, i32 0, i32 1
        [-] %load_102 = load i32, ptr %getelementptr_186
        [-] call void @putint(i32 %load_102)
        [-] %getelementptr_190 = getelementptr [7 x [1 x [5 x i32]]], ptr %alloca_71, i32 0, i32 2, i32 0, i32 2
        [-] %load_108 = load i32, ptr %getelementptr_190
        [-] call void @putint(i32 %load_108)
        [-] br label %exit
        [-] 
        [-] alt9:
        [-] br label %exit
        [-] 
        [-] exit:
        [+] call void @putint(i32 2)
        [+] call void @putint(i32 1)
        [+] call void @putint(i32 8)
        call void @putch(i32 10)
        [-] %load_113 = load i32, ptr @b
        [-] call void @putint(i32 %load_113)
        [+] call void @putint(i32 5)
        call void @putch(i32 10)
        [-] %getelementptr_116 = getelementptr [4 x i32], ptr @c, i32 0, i32 0
        [-] %load_117 = load i32, ptr %getelementptr_116
        [-] call void @putint(i32 %load_117)
        [-] %getelementptr_119 = getelementptr [4 x i32], ptr @c, i32 0, i32 1
        [-] %load_120 = load i32, ptr %getelementptr_119
        [-] call void @putint(i32 %load_120)
        [-] %load_123 = load i32, ptr %getelementptr_43
        [-] call void @putint(i32 %load_123)
        [-] %getelementptr_125 = getelementptr [4 x i32], ptr @c, i32 0, i32 3
        [-] %load_126 = load i32, ptr %getelementptr_125
        [-] call void @putint(i32 %load_126)
        [+] call void @putint(i32 6)
        [+] call void @putint(i32 7)
        [+] call void @putint(i32 1)
        [+] call void @putint(i32 9)
        call void @putch(i32 10)
        ret i32 0


        }
        "###);
    }
}
