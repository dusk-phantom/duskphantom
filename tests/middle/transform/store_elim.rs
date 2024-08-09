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
        let code = r#"#include "../../lib/sylib.h"
        #define INF 65535
        int e[16][16];
        int book[16];
        int dis[16];
        int n, m;
        int v1, v2, w;

        void Dijkstra()
        {
            int i, j;

            i = 1;
            while (i <= n) {
                dis[i] = e[1][i];
                book[i] = 0;
                i = i + 1;
            }
            book[1] = 1;

            i = 1;
            while (i <= n - 1) {
                int min_num = INF;
                int min_index = 0;
                int k = 1;
                while (k <= n) {
                    if (min_num > dis[k] && book[k] == 0) {
                        min_num = dis[k];
                        min_index = k;
                    }
                    k = k + 1;
                }
                book[min_index] = 1;
                int j = 1;
                while (j <= n) {
                    if (e[min_index][j] < INF) {
                        if (dis[j] > dis[min_index] + e[min_index][j]) {
                            dis[j] = dis[min_index] + e[min_index][j];
                        }
                    }
                    j = j + 1;
                }
                i = i + 1;
            }
        }

        int main()
        {
            int i;
            n = getint();
            m = getint();

            i = 1;
            while (i <= n) {
                int j = 1;
                while (j <= n) {
                    if (i == j)
                        e[i][j] = 0;
                    else
                        e[i][j] = INF;
                    j = j + 1;
                }
                i = i + 1;
            }

            i = 1;
            while (i <= m) {
                int u = getint(), v = getint();
                e[u][v] = getint();
                i = i + 1;
            }

            Dijkstra();

            i = 1;
            while (i <= n) {
                putint(dis[i]);
                putch(32);
                i = i + 1;
            }
            putch(10);
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
        @INF = dso_local constant i32 65535
        @e = dso_local global [16 x [16 x i32]] zeroinitializer
        @book = dso_local global [16 x i32] zeroinitializer
        @dis = dso_local global [16 x i32] zeroinitializer
        @n = dso_local global i32 0
        @m = dso_local global i32 0
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
        define void @Dijkstra() {
        entry:
        br label %cond0

        cond0:
        %phi_263 = phi i32 [1, %entry], [%Add_21, %body1]
        %load_25 = load i32, ptr @n
        %icmp_26 = icmp sle i32 %phi_263, %load_25
        br i1 %icmp_26, label %body1, label %final2

        body1:
        %getelementptr_422 = getelementptr [16 x [16 x i32]], ptr @e, i32 0, i32 1, i32 %phi_263
        %getelementptr_14 = getelementptr [16 x i32], ptr @dis, i32 0, i32 %phi_263
        %load_15 = load i32, ptr %getelementptr_422
        store i32 %load_15, ptr %getelementptr_14
        %getelementptr_18 = getelementptr [16 x i32], ptr @book, i32 0, i32 %phi_263
        store i32 0, ptr %getelementptr_18
        %Add_21 = add i32 %phi_263, 1
        br label %cond0

        final2:
        %getelementptr_28 = getelementptr [16 x i32], ptr @book, i32 0, i32 1
        store i32 1, ptr %getelementptr_28
        br label %cond3

        cond3:
        %phi_264 = phi i32 [1, %final2], [%Add_148, %final17]
        %load_151 = load i32, ptr @n
        %Sub_152 = sub i32 %load_151, 1
        %icmp_154 = icmp sle i32 %phi_264, %Sub_152
        br i1 %icmp_154, label %body4, label %exit

        body4:
        %load_36 = load i32, ptr @INF
        br label %cond6

        exit:
        ret void

        cond6:
        %phi_272 = phi i32 [1, %body4], [%Add_75, %final12]
        %phi_269 = phi i32 [0, %body4], [%phi_270, %final12]
        %phi_266 = phi i32 [%load_36, %body4], [%phi_267, %final12]
        %load_79 = load i32, ptr @n
        %icmp_80 = icmp sle i32 %phi_272, %load_79
        br i1 %icmp_80, label %cond9, label %final8

        cond9:
        %getelementptr_52 = getelementptr [16 x i32], ptr @dis, i32 0, i32 %phi_272
        %load_54 = load i32, ptr %getelementptr_52
        %icmp_55 = icmp sgt i32 %phi_266, %load_54
        br i1 %icmp_55, label %alt13, label %final14

        final8:
        %getelementptr_83 = getelementptr [16 x i32], ptr @book, i32 0, i32 %phi_269
        store i32 1, ptr %getelementptr_83
        br label %cond15

        alt13:
        %getelementptr_60 = getelementptr [16 x i32], ptr @book, i32 0, i32 %phi_272
        %load_61 = load i32, ptr %getelementptr_60
        %icmp_62 = icmp eq i32 %load_61, 0
        br label %final14

        final14:
        %phi_64 = phi i1 [false, %cond9], [%icmp_62, %alt13]
        br i1 %phi_64, label %then10, label %alt11

        cond15:
        %phi_273 = phi i32 [1, %final8], [%Add_140, %final21]
        %load_144 = load i32, ptr @n
        %icmp_145 = icmp sle i32 %phi_273, %load_144
        br i1 %icmp_145, label %cond18, label %final17

        then10:
        %load_68 = load i32, ptr %getelementptr_52
        br label %final12

        alt11:
        br label %final12

        cond18:
        %getelementptr_416 = getelementptr [16 x [16 x i32]], ptr @e, i32 0, i32 %phi_269, i32 %phi_273
        %load_100 = load i32, ptr %getelementptr_416
        %load_101 = load i32, ptr @INF
        %icmp_102 = icmp slt i32 %load_100, %load_101
        br i1 %icmp_102, label %cond22, label %alt20

        final17:
        %Add_148 = add i32 %phi_264, 1
        br label %cond3

        final12:
        %phi_270 = phi i32 [%phi_272, %then10], [%phi_269, %alt11]
        %phi_267 = phi i32 [%load_68, %then10], [%phi_266, %alt11]
        %Add_75 = add i32 %phi_272, 1
        br label %cond6

        cond22:
        %getelementptr_110 = getelementptr [16 x i32], ptr @dis, i32 0, i32 %phi_273
        %getelementptr_112 = getelementptr [16 x i32], ptr @dis, i32 0, i32 %phi_269
        %load_117 = load i32, ptr %getelementptr_112
        %load_118 = load i32, ptr %getelementptr_416
        %Add_119 = add i32 %load_117, %load_118
        %load_120 = load i32, ptr %getelementptr_110
        %icmp_121 = icmp sgt i32 %load_120, %Add_119
        br i1 %icmp_121, label %then23, label %alt24

        alt20:
        br label %final21

        then23:
        %load_129 = load i32, ptr %getelementptr_112
        %load_130 = load i32, ptr %getelementptr_416
        %Add_131 = add i32 %load_129, %load_130
        store i32 %Add_131, ptr %getelementptr_110
        br label %final25

        alt24:
        br label %final25

        final21:
        %Add_140 = add i32 %phi_273, 1
        br label %cond15

        final25:
        br label %final21


        }
        define i32 @main() {
        entry:
        %call_163 = call i32 @getint()
        store i32 %call_163, ptr @n
        %call_165 = call i32 @getint()
        store i32 %call_165, ptr @m
        br label %cond0

        cond0:
        %phi_275 = phi i32 [1, %entry], [%Add_209, %final5]
        [-] %load_213 = load i32, ptr @n
        [-] %icmp_214 = icmp sle i32 %phi_275, %load_213
        [+] %icmp_214 = icmp sle i32 %phi_275, %call_163
        br i1 %icmp_214, label %body1, label %final2

        body1:
        br label %cond3

        final2:
        br label %cond10

        cond3:
        %phi_279 = phi i32 [1, %body1], [%Add_201, %final9]
        [-] %load_205 = load i32, ptr @n
        [-] %icmp_206 = icmp sle i32 %phi_279, %load_205
        [+] %icmp_206 = icmp sle i32 %phi_279, %call_163
        br i1 %icmp_206, label %cond6, label %final5

        cond10:
        %phi_276 = phi i32 [1, %final2], [%Add_234, %body11]
        [-] %load_238 = load i32, ptr @m
        [-] %icmp_239 = icmp sle i32 %phi_276, %load_238
        [+] %icmp_239 = icmp sle i32 %phi_276, %call_165
        br i1 %icmp_239, label %body11, label %entry_inline0

        cond6:
        %icmp_185 = icmp eq i32 %phi_275, %phi_279
        br i1 %icmp_185, label %then7, label %alt8

        final5:
        %Add_209 = add i32 %phi_275, 1
        br label %cond0

        body11:
        %call_222 = call i32 @getint()
        %call_225 = call i32 @getint()
        %call_227 = call i32 @getint()
        %getelementptr_432 = getelementptr [16 x [16 x i32]], ptr @e, i32 0, i32 %call_222, i32 %call_225
        store i32 %call_227, ptr %getelementptr_432
        %Add_234 = add i32 %phi_276, 1
        br label %cond10

        entry_inline0:
        br label %cond0_inline1

        then7:
        %getelementptr_436 = getelementptr [16 x [16 x i32]], ptr @e, i32 0, i32 %phi_275, i32 %phi_279
        store i32 0, ptr %getelementptr_436
        br label %final9

        alt8:
        %getelementptr_434 = getelementptr [16 x [16 x i32]], ptr @e, i32 0, i32 %phi_275, i32 %phi_279
        [-] %load_197 = load i32, ptr @INF
        [-] store i32 %load_197, ptr %getelementptr_434
        [+] store i32 65535, ptr %getelementptr_434
        br label %final9

        cond0_inline1:
        %phi_287 = phi i32 [1, %entry_inline0], [%Add_410, %body1_inline27]
        [-] %load_288 = load i32, ptr @n
        [-] %icmp_289 = icmp sle i32 %phi_287, %load_288
        [+] %icmp_289 = icmp sle i32 %phi_287, %call_163
        br i1 %icmp_289, label %body1_inline27, label %final2_inline2

        final9:
        %Add_201 = add i32 %phi_279, 1
        br label %cond3

        body1_inline27:
        %getelementptr_430 = getelementptr [16 x [16 x i32]], ptr @e, i32 0, i32 1, i32 %phi_287
        %getelementptr_405 = getelementptr [16 x i32], ptr @dis, i32 0, i32 %phi_287
        %load_406 = load i32, ptr %getelementptr_430
        store i32 %load_406, ptr %getelementptr_405
        %getelementptr_408 = getelementptr [16 x i32], ptr @book, i32 0, i32 %phi_287
        store i32 0, ptr %getelementptr_408
        %Add_410 = add i32 %phi_287, 1
        br label %cond0_inline1

        final2_inline2:
        %getelementptr_292 = getelementptr [16 x i32], ptr @book, i32 0, i32 1
        store i32 1, ptr %getelementptr_292
        br label %cond3_inline3

        cond3_inline3:
        %phi_300 = phi i32 [1, %final2_inline2], [%Add_333, %final17_inline10]
        [-] %load_301 = load i32, ptr @n
        [-] %Sub_302 = sub i32 %load_301, 1
        [+] %Sub_302 = sub i32 %call_163, 1
        %icmp_303 = icmp sle i32 %phi_300, %Sub_302
        br i1 %icmp_303, label %body4_inline6, label %final12_split28

        body4_inline6:
        [-] %load_311 = load i32, ptr @INF
        br label %cond6_inline7

        final12_split28:
        br label %cond13

        cond6_inline7:
        %phi_316 = phi i32 [1, %body4_inline6], [%Add_391, %final12_inline24]
        %phi_317 = phi i32 [0, %body4_inline6], [%phi_389, %final12_inline24]
        [-] %phi_318 = phi i32 [%load_311, %body4_inline6], [%phi_390, %final12_inline24]
        [-] %load_319 = load i32, ptr @n
        [-] %icmp_320 = icmp sle i32 %phi_316, %load_319
        [+] %phi_318 = phi i32 [65535, %body4_inline6], [%phi_390, %final12_inline24]
        [+] %icmp_320 = icmp sle i32 %phi_316, %call_163
        br i1 %icmp_320, label %cond9_inline21, label %final8_inline8

        cond13:
        %phi_277 = phi i32 [1, %final12_split28], [%Add_253, %body14]
        [-] %load_257 = load i32, ptr @n
        [-] %icmp_258 = icmp sle i32 %phi_277, %load_257
        [+] %icmp_258 = icmp sle i32 %phi_277, %call_163
        br i1 %icmp_258, label %body14, label %exit

        cond9_inline21:
        %getelementptr_379 = getelementptr [16 x i32], ptr @dis, i32 0, i32 %phi_316
        %load_380 = load i32, ptr %getelementptr_379
        %icmp_381 = icmp sgt i32 %phi_318, %load_380
        br i1 %icmp_381, label %alt13_inline26, label %final14_inline22

        final8_inline8:
        %getelementptr_323 = getelementptr [16 x i32], ptr @book, i32 0, i32 %phi_317
        store i32 1, ptr %getelementptr_323
        br label %cond15_inline9

        body14:
        %getelementptr_248 = getelementptr [16 x i32], ptr @dis, i32 0, i32 %phi_277
        %load_249 = load i32, ptr %getelementptr_248
        call void @putint(i32 %load_249)
        call void @putch(i32 32)
        %Add_253 = add i32 %phi_277, 1
        br label %cond13

        exit:
        call void @putch(i32 10)
        ret i32 0

        alt13_inline26:
        %getelementptr_398 = getelementptr [16 x i32], ptr @book, i32 0, i32 %phi_316
        %load_399 = load i32, ptr %getelementptr_398
        %icmp_400 = icmp eq i32 %load_399, 0
        br label %final14_inline22

        final14_inline22:
        %phi_384 = phi i1 [false, %cond9_inline21], [%icmp_400, %alt13_inline26]
        br i1 %phi_384, label %then10_inline25, label %alt11_inline23

        cond15_inline9:
        %phi_328 = phi i32 [1, %final8_inline8], [%Add_347, %final21_inline14]
        [-] %load_329 = load i32, ptr @n
        [-] %icmp_330 = icmp sle i32 %phi_328, %load_329
        [+] %icmp_330 = icmp sle i32 %phi_328, %call_163
        br i1 %icmp_330, label %cond18_inline12, label %final17_inline10

        then10_inline25:
        %load_395 = load i32, ptr %getelementptr_379
        br label %final12_inline24

        alt11_inline23:
        br label %final12_inline24

        cond18_inline12:
        %getelementptr_424 = getelementptr [16 x [16 x i32]], ptr @e, i32 0, i32 %phi_317, i32 %phi_328
        %load_340 = load i32, ptr %getelementptr_424
        [-] %load_341 = load i32, ptr @INF
        [-] %icmp_342 = icmp slt i32 %load_340, %load_341
        [+] %icmp_342 = icmp slt i32 %load_340, 65535
        br i1 %icmp_342, label %cond22_inline16, label %alt20_inline13

        final17_inline10:
        %Add_333 = add i32 %phi_300, 1
        br label %cond3_inline3

        final12_inline24:
        %phi_389 = phi i32 [%phi_316, %then10_inline25], [%phi_317, %alt11_inline23]
        %phi_390 = phi i32 [%load_395, %then10_inline25], [%phi_318, %alt11_inline23]
        %Add_391 = add i32 %phi_316, 1
        br label %cond6_inline7

        cond22_inline16:
        %getelementptr_352 = getelementptr [16 x i32], ptr @dis, i32 0, i32 %phi_328
        %getelementptr_353 = getelementptr [16 x i32], ptr @dis, i32 0, i32 %phi_317
        %load_356 = load i32, ptr %getelementptr_353
        %load_357 = load i32, ptr %getelementptr_424
        %Add_358 = add i32 %load_356, %load_357
        %load_359 = load i32, ptr %getelementptr_352
        %icmp_360 = icmp sgt i32 %load_359, %Add_358
        br i1 %icmp_360, label %then23_inline19, label %alt24_inline17

        alt20_inline13:
        br label %final21_inline14

        then23_inline19:
        %load_370 = load i32, ptr %getelementptr_353
        %load_371 = load i32, ptr %getelementptr_424
        %Add_372 = add i32 %load_370, %load_371
        store i32 %Add_372, ptr %getelementptr_352
        br label %final25_inline18

        alt24_inline17:
        br label %final25_inline18

        final21_inline14:
        %Add_347 = add i32 %phi_328, 1
        br label %cond15_inline9

        final25_inline18:
        br label %final21_inline14


        }
        "###);
    }
}
