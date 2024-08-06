#[cfg(test)]
pub mod tests_constant_fold {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{constant_fold, deadcode_elimination, inst_combine, mem2reg},
        },
        utils::diff::diff,
    };

    #[test]
    fn test_normal() {
        let code = r#"
        int f(int x) {
            int x1 = x + 0 + 0 + x;
            int x2 = 2 * x1 * 5;
            int x3 = x2 * 3;
            return x3;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        deadcode_elimination::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        deadcode_elimination::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        inst_combine::optimize_program(&mut program).unwrap();
        deadcode_elimination::optimize_program(&mut program).unwrap();
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
        define i32 @f(i32 %x) {
        entry:
        [-] %Add_9 = add i32 %x, 0
        [-] %Add_10 = add i32 %Add_9, 0
        [-] %Add_12 = add i32 %Add_10, %x
        [-] %Mul_16 = mul i32 2, %Add_12
        [-] %Mul_17 = mul i32 %Mul_16, 5
        [-] %Mul_21 = mul i32 %Mul_17, 3
        [+] %Mul_29 = mul i32 %x, 60
        br label %exit

        exit:
        [-] ret i32 %Mul_21
        [+] ret i32 %Mul_29


        }
        "###);
    }
}
