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

    #[test]
    fn test_large_2() {
        let code = r#"
        #include "../../lib/sylib.h"
        int M;
        int L;
        int N;

        int tran(float a0[],float a1[], float a2[],float b0[],float b1[],float b2[],float c0[],float c1[],float c2[])
        {
            int i;
            i=0;
            c1[2]=a2[1];
            c2[1]=a1[2];
            c0[1]=a1[0];
            c0[2]=a2[0];
            c1[0]=a0[1];
            c2[0]=a0[2];
            c1[1]=a1[1];
            c2[2]=a2[2];
            c0[0]=a0[0];

            return 0;

        }

        int main()
        {
            N=3;
            M=3;
            L=3;
            float a0[3], a1[3], a2[3], b0[3], b1[3], b2[3], c0[6], c1[3], c2[3];
            int i;
            i=0;
            while(i<M)
            {
                a0[i]=i;
                a1[i]=i;
                a2[i]=i;
                b0[i]=i;
                b1[i]=i;
                b2[i]=i;
                i=i+1;
            }
            i=tran( a0, a1,  a2, b0, b1, b2, c0, c1, c2);
            int x;
            while(i<N)
            {
                x = c0[i];
                putint(x);
            
                i=i+1;
            }
            x = 10;
            putch(x);
            i=0;
            while(i<N)
            {
                x = c1[i];
                putint(x);
            
                i=i+1;
            }
            x = 10;
            i=0;
            putch(x);
            while(i<N)
            {
                x = c2[i];
                putint(x);
            
                i=i+1;
            }
            x = 10;
            putch(x);

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
        @M = dso_local global i32 0
        @L = dso_local global i32 0
        @N = dso_local global i32 0
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
        define i32 @tran(float* %a0, float* %a1, float* %a2, float* %b0, float* %b1, float* %b2, float* %c0, float* %c1, float* %c2) {
        exit:
        %getelementptr_26 = getelementptr float, ptr %a2, i32 1
        %getelementptr_28 = getelementptr float, ptr %c1, i32 2
        %load_29 = load float, ptr %getelementptr_26
        store float %load_29, ptr %getelementptr_28
        %getelementptr_32 = getelementptr float, ptr %a1, i32 2
        %getelementptr_34 = getelementptr float, ptr %c2, i32 1
        %load_35 = load float, ptr %getelementptr_32
        store float %load_35, ptr %getelementptr_34
        %getelementptr_38 = getelementptr float, ptr %a1, i32 0
        %getelementptr_40 = getelementptr float, ptr %c0, i32 1
        %load_41 = load float, ptr %getelementptr_38
        store float %load_41, ptr %getelementptr_40
        %getelementptr_44 = getelementptr float, ptr %a2, i32 0
        %getelementptr_46 = getelementptr float, ptr %c0, i32 2
        %load_47 = load float, ptr %getelementptr_44
        store float %load_47, ptr %getelementptr_46
        %getelementptr_50 = getelementptr float, ptr %a0, i32 1
        %getelementptr_52 = getelementptr float, ptr %c1, i32 0
        %load_53 = load float, ptr %getelementptr_50
        store float %load_53, ptr %getelementptr_52
        %getelementptr_56 = getelementptr float, ptr %a0, i32 2
        %getelementptr_58 = getelementptr float, ptr %c2, i32 0
        %load_59 = load float, ptr %getelementptr_56
        store float %load_59, ptr %getelementptr_58
        %getelementptr_62 = getelementptr float, ptr %a1, i32 1
        %getelementptr_64 = getelementptr float, ptr %c1, i32 1
        %load_65 = load float, ptr %getelementptr_62
        store float %load_65, ptr %getelementptr_64
        %getelementptr_68 = getelementptr float, ptr %a2, i32 2
        %getelementptr_70 = getelementptr float, ptr %c2, i32 2
        %load_71 = load float, ptr %getelementptr_68
        store float %load_71, ptr %getelementptr_70
        %getelementptr_74 = getelementptr float, ptr %a0, i32 0
        %getelementptr_76 = getelementptr float, ptr %c0, i32 0
        %load_77 = load float, ptr %getelementptr_74
        store float %load_77, ptr %getelementptr_76
        ret i32 0


        }
        define i32 @main() {
        entry:
        [-] store i32 3, ptr @N
        [-] store i32 3, ptr @M
        [-] store i32 3, ptr @L
        %alloca_89 = alloca [3 x float]
        %alloca_90 = alloca [3 x float]
        %alloca_91 = alloca [3 x float]
        [-] %alloca_92 = alloca [3 x float]
        [-] %alloca_93 = alloca [3 x float]
        %alloca_94 = alloca [3 x float]
        %alloca_95 = alloca [6 x float]
        %alloca_96 = alloca [3 x float]
        %alloca_97 = alloca [3 x float]
        br label %cond0

        cond0:
        %phi_224 = phi i32 [0, %entry], [%Add_135, %body1]
        [-] %load_139 = load i32, ptr @M
        [-] %icmp_140 = icmp slt i32 %phi_224, %load_139
        [+] %icmp_140 = icmp slt i32 %phi_224, 3
        br i1 %icmp_140, label %body1, label %final2_split2

        body1:
        [-] %getelementptr_105 = getelementptr [3 x float], ptr %alloca_89, i32 0, i32 %phi_224
        %itofp_107 = sitofp i32 %phi_224 to float
        [-] store float %itofp_107, ptr %getelementptr_105
        [-] %getelementptr_110 = getelementptr [3 x float], ptr %alloca_90, i32 0, i32 %phi_224
        [-] store float %itofp_107, ptr %getelementptr_110
        [-] %getelementptr_115 = getelementptr [3 x float], ptr %alloca_91, i32 0, i32 %phi_224
        [-] store float %itofp_107, ptr %getelementptr_115
        [-] %getelementptr_120 = getelementptr [3 x float], ptr %alloca_92, i32 0, i32 %phi_224
        [-] store float %itofp_107, ptr %getelementptr_120
        [-] %getelementptr_125 = getelementptr [3 x float], ptr %alloca_93, i32 0, i32 %phi_224
        [-] store float %itofp_107, ptr %getelementptr_125
        %getelementptr_130 = getelementptr [3 x float], ptr %alloca_94, i32 0, i32 %phi_224
        store float %itofp_107, ptr %getelementptr_130
        %Add_135 = add i32 %phi_224, 1
        br label %cond0

        final2_split2:
        %getelementptr_142 = getelementptr [3 x float], ptr %alloca_89, i32 0, i32 0
        [-] %getelementptr_143 = getelementptr [3 x float], ptr %alloca_90, i32 0, i32 0
        [-] %getelementptr_144 = getelementptr [3 x float], ptr %alloca_91, i32 0, i32 0
        %getelementptr_148 = getelementptr [6 x float], ptr %alloca_95, i32 0, i32 0
        [-] %getelementptr_149 = getelementptr [3 x float], ptr %alloca_96, i32 0, i32 0
        [-] %getelementptr_150 = getelementptr [3 x float], ptr %alloca_97, i32 0, i32 0
        [-] %Add_285 = add i32 0, 1
        [-] %getelementptr_286 = getelementptr [3 x float], ptr %alloca_91, i32 0, i32 %Add_285
        [-] %Add_287 = add i32 0, 2
        [-] %getelementptr_288 = getelementptr [3 x float], ptr %alloca_96, i32 0, i32 %Add_287
        [-] %load_245 = load float, ptr %getelementptr_286
        [-] store float %load_245, ptr %getelementptr_288
        [-] %getelementptr_290 = getelementptr [3 x float], ptr %alloca_90, i32 0, i32 %Add_287
        [-] %getelementptr_292 = getelementptr [3 x float], ptr %alloca_97, i32 0, i32 %Add_285
        [-] %load_249 = load float, ptr %getelementptr_290
        [-] store float %load_249, ptr %getelementptr_292
        [-] %getelementptr_296 = getelementptr [6 x float], ptr %alloca_95, i32 0, i32 %Add_285
        [-] %load_253 = load float, ptr %getelementptr_143
        [-] store float %load_253, ptr %getelementptr_296
        [-] %getelementptr_300 = getelementptr [6 x float], ptr %alloca_95, i32 0, i32 %Add_287
        [-] %load_257 = load float, ptr %getelementptr_144
        [-] store float %load_257, ptr %getelementptr_300
        [-] %getelementptr_302 = getelementptr [3 x float], ptr %alloca_89, i32 0, i32 %Add_285
        [-] %load_261 = load float, ptr %getelementptr_302
        [-] store float %load_261, ptr %getelementptr_149
        [-] %getelementptr_306 = getelementptr [3 x float], ptr %alloca_89, i32 0, i32 %Add_287
        [-] %load_265 = load float, ptr %getelementptr_306
        [-] store float %load_265, ptr %getelementptr_150
        [-] %getelementptr_310 = getelementptr [3 x float], ptr %alloca_90, i32 0, i32 %Add_285
        [-] %getelementptr_312 = getelementptr [3 x float], ptr %alloca_96, i32 0, i32 %Add_285
        [+] %getelementptr_310 = getelementptr [3 x float], ptr %alloca_90, i32 0, i32 1
        [+] %getelementptr_312 = getelementptr [3 x float], ptr %alloca_96, i32 0, i32 1
        %load_269 = load float, ptr %getelementptr_310
        store float %load_269, ptr %getelementptr_312
        [-] %getelementptr_314 = getelementptr [3 x float], ptr %alloca_91, i32 0, i32 %Add_287
        [-] %getelementptr_316 = getelementptr [3 x float], ptr %alloca_97, i32 0, i32 %Add_287
        [+] %getelementptr_314 = getelementptr [3 x float], ptr %alloca_91, i32 0, i32 2
        [+] %getelementptr_316 = getelementptr [3 x float], ptr %alloca_97, i32 0, i32 2
        %load_273 = load float, ptr %getelementptr_314
        store float %load_273, ptr %getelementptr_316
        %load_277 = load float, ptr %getelementptr_142
        store float %load_277, ptr %getelementptr_148
        br label %cond3

        cond3:
        %phi_225 = phi i32 [0, %final2_split2], [%Add_166, %body4]
        [-] %load_170 = load i32, ptr @N
        [-] %icmp_171 = icmp slt i32 %phi_225, %load_170
        [+] %icmp_171 = icmp slt i32 %phi_225, 3
        br i1 %icmp_171, label %body4, label %final5

        body4:
        %getelementptr_159 = getelementptr [6 x float], ptr %alloca_95, i32 0, i32 %phi_225
        %load_160 = load float, ptr %getelementptr_159
        %fptoi_161 = fptosi float %load_160 to i32
        call void @putint(i32 %fptoi_161)
        %Add_166 = add i32 %phi_225, 1
        br label %cond3

        final5:
        call void @putch(i32 10)
        br label %cond6

        cond6:
        %phi_226 = phi i32 [0, %final5], [%Add_189, %body7]
        [-] %load_193 = load i32, ptr @N
        [-] %icmp_194 = icmp slt i32 %phi_226, %load_193
        [+] %icmp_194 = icmp slt i32 %phi_226, 3
        br i1 %icmp_194, label %body7, label %final8

        body7:
        %getelementptr_182 = getelementptr [3 x float], ptr %alloca_96, i32 0, i32 %phi_226
        %load_183 = load float, ptr %getelementptr_182
        %fptoi_184 = fptosi float %load_183 to i32
        call void @putint(i32 %fptoi_184)
        %Add_189 = add i32 %phi_226, 1
        br label %cond6

        final8:
        call void @putch(i32 10)
        br label %cond9

        cond9:
        %phi_227 = phi i32 [0, %final8], [%Add_212, %body10]
        [-] %load_216 = load i32, ptr @N
        [-] %icmp_217 = icmp slt i32 %phi_227, %load_216
        [+] %icmp_217 = icmp slt i32 %phi_227, 3
        br i1 %icmp_217, label %body10, label %exit

        body10:
        %getelementptr_205 = getelementptr [3 x float], ptr %alloca_97, i32 0, i32 %phi_227
        %load_206 = load float, ptr %getelementptr_205
        %fptoi_207 = fptosi float %load_206 to i32
        call void @putint(i32 %fptoi_207)
        %Add_212 = add i32 %phi_227, 1
        br label %cond9

        exit:
        call void @putch(i32 10)
        ret i32 0


        }
        "###);
    }
}
