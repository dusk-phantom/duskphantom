#[cfg(test)]
pub mod tests_func_inline {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{
                block_fuse, constant_fold, dead_code_elim, func_inline, inst_combine, mem2reg,
                unreachable_block_elim,
            },
        },
        utils::diff::diff,
    };

    #[test]
    fn test_fun_in_cond() {
        let code = r#"
        int func(int n) {
            putint(n);
            return n;
        }

        int main() {
            int i = getint();
            if (i > 10 && func(i)) i = 1; else i = 0;
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        func_inline::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        unreachable_block_elim::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
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
        [-] define i32 @func(i32 %n) {
        [-] entry:
        [-] call void @putint(i32 %n)
        [-] br label %exit
        [-] 
        [-] exit:
        [-] ret i32 %n
        [-] 
        [-] 
        [-] }
        define i32 @main() {
        [-] entry:
        [-] %call_18 = call i32 @getint()
        [-] br label %cond0
        [-] 
        cond0:
        [+] %call_18 = call i32 @getint()
        %icmp_26 = icmp sgt i32 %call_18, 10
        [-] br i1 %icmp_26, label %alt4, label %final5
        [+] br i1 %icmp_26, label %alt4_split2, label %final5

        [-] alt4:
        [-] %call_31 = call i32 @func(i32 %call_18)
        [-] %icmp_32 = icmp ne i32 %call_31, 0
        [+] alt4_split2:
        [+] call void @putint(i32 %call_18)
        [+] %icmp_32 = icmp ne i32 %call_18, 0
        br label %final5

        final5:
        [-] %phi_34 = phi i1 [false, %cond0], [%icmp_32, %alt4]
        [+] %phi_34 = phi i1 [false, %cond0], [%icmp_32, %alt4_split2]
        br i1 %phi_34, label %then1, label %alt2

        then1:
        [-] br label %final3
        [+] br label %exit

        alt2:
        [-] br label %final3
        [-] 
        [-] final3:
        br label %exit

        exit:
        ret i32 0


        }
        "###);
    }

    #[test]
    fn test_phi() {
        let code = r#"
        int f(int x) {
            if (x > 5) return x + 1;
            return x + 2;
        }
        int main() {
            return f(5);
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        func_inline::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        unreachable_block_elim::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
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
        [-] define i32 @f(i32 %x) {
        [-] entry:
        [-] br label %cond0
        [-] 
        [-] cond0:
        [-] %icmp_13 = icmp sgt i32 %x, 5
        [-] br i1 %icmp_13, label %then1, label %alt2
        [-] 
        [-] then1:
        [-] %Add_16 = add i32 %x, 1
        [-] br label %exit
        [-] 
        [-] alt2:
        [-] br label %final3
        [-] 
        [-] exit:
        [-] %phi_32 = phi i32 [%Add_16, %then1], [%Add_21, %final3]
        [-] ret i32 %phi_32
        [-] 
        [-] final3:
        [-] %Add_21 = add i32 %x, 2
        [-] br label %exit
        [-] 
        [-] 
        [-] }
        define i32 @main() {
        [-] entry:
        [-] %call_29 = call i32 @f(i32 5)
        [-] br label %exit
        [-] 
        exit:
        [-] ret i32 %call_29
        [+] ret i32 7


        }
        "###);
    }

    #[test]
    fn test_normal() {
        let code = r#"
        int f(int x) {
            return x + 1;
        }
        int main() {
            return f(5);
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        func_inline::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        unreachable_block_elim::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
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
        [-] define i32 @f(i32 %x) {
        [-] entry:
        [-] %Add_8 = add i32 %x, 1
        [-] br label %exit
        [-] 
        [-] exit:
        [-] ret i32 %Add_8
        [-] 
        [-] 
        [-] }
        define i32 @main() {
        [-] entry:
        [-] %call_16 = call i32 @f(i32 5)
        [-] br label %exit
        [-] 
        exit:
        [-] ret i32 %call_16
        [+] ret i32 6


        }
        "###);
    }
}
