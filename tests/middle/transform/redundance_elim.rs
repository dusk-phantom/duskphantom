#[cfg(test)]
pub mod tests_constant_fold {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{
                block_fuse, constant_fold, dead_code_elim, func_inline, inst_combine, mem2reg,
                redundance_elim,
            },
        },
        utils::diff::diff,
    };

    #[test]
    fn test_redundant_load() {
        let code = r#"
        int main() {
            int x[8];
            getarray(x);
            return x[4] + x[4] + x[4] + x[4];
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        redundance_elim::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
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
        entry:
        %alloca_5 = alloca [8 x i32]
        %getelementptr_6 = getelementptr [8 x i32], ptr %alloca_5, i32 0, i32 0
        %call_7 = call i32 @getarray(i32* %getelementptr_6)
        %getelementptr_8 = getelementptr [8 x i32], ptr %alloca_5, i32 0, i32 4
        [-] %getelementptr_9 = getelementptr [8 x i32], ptr %alloca_5, i32 0, i32 4
        %load_10 = load i32, ptr %getelementptr_8
        [-] %load_11 = load i32, ptr %getelementptr_9
        [-] %Add_12 = add i32 %load_10, %load_11
        [-] %getelementptr_13 = getelementptr [8 x i32], ptr %alloca_5, i32 0, i32 4
        [-] %load_14 = load i32, ptr %getelementptr_13
        [-] %Add_15 = add i32 %Add_12, %load_14
        [-] %getelementptr_16 = getelementptr [8 x i32], ptr %alloca_5, i32 0, i32 4
        [-] %load_17 = load i32, ptr %getelementptr_16
        [-] %Add_18 = add i32 %Add_15, %load_17
        [+] %Mul_23 = mul i32 %load_10, 4
        br label %exit

        exit:
        [-] ret i32 %Add_18
        [+] ret i32 %Mul_23


        }
        "###);
    }

    #[test]
    fn test_normal() {
        let code = r#"
        int main() {
            int x = getint();
            int y = getint();
            int z = x + y;
            int w = x + y;
            int a = z + w;
            int b = w + z;
            return a + b;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        redundance_elim::optimize_program(&mut program).unwrap();
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
        entry:
        %call_6 = call i32 @getint()
        %call_9 = call i32 @getint()
        %Add_14 = add i32 %call_6, %call_9
        [-] %Add_19 = add i32 %call_6, %call_9
        [-] %Add_24 = add i32 %Add_14, %Add_19
        [-] %Add_29 = add i32 %Add_19, %Add_14
        [-] %Add_33 = add i32 %Add_24, %Add_29
        [+] %Add_24 = add i32 %Add_14, %Add_14
        [+] %Add_33 = add i32 %Add_24, %Add_24
        br label %exit

        exit:
        ret i32 %Add_33


        }
        "###);
    }

    #[test]
    fn test_pure_function() {
        let code = r#"
        int f(int x) {
            return f(x) - f(x);
        }
        int main() {
            return f(6);
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        redundance_elim::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        func_inline::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
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
        [-] define i32 @f(i32 %x) {
        [-] entry:
        [-] %call_8 = call i32 @f(i32 %x)
        [-] %call_10 = call i32 @f(i32 %x)
        [-] %Sub_11 = sub i32 %call_8, %call_10
        [-] br label %exit
        [-] 
        [-] exit:
        [-] ret i32 %Sub_11
        [-] 
        [-] 
        [-] }
        define i32 @main() {
        [-] entry:
        [-] %call_19 = call i32 @f(i32 6)
        [-] br label %exit
        [-] 
        exit:
        [-] ret i32 %call_19
        [+] ret i32 0


        }
        "###);
    }

    #[test]
    fn test_max() {
        let code = r#"
        int MAX(int a, int b)
        {
            if (a == b)
                return a;
            // ICmp here should not be GVNed
            else if (a > b)
                return a;
            else
                return b;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        redundance_elim::optimize_program(&mut program).unwrap();
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
        define i32 @MAX(i32 %a, i32 %b) {
        entry:
        br label %cond0

        cond0:
        %icmp_16 = icmp eq i32 %a, %b
        br i1 %icmp_16, label %then1, label %alt2

        then1:
        br label %exit

        alt2:
        br label %cond4

        exit:
        %phi_38 = phi i32 [%a, %then1], [%a, %then5], [%b, %alt6]
        ret i32 %phi_38

        cond4:
        %icmp_28 = icmp sgt i32 %a, %b
        br i1 %icmp_28, label %then5, label %alt6

        then5:
        br label %exit

        alt6:
        br label %exit


        }
        "###);
    }
}
