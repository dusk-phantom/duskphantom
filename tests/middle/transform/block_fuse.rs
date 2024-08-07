#[cfg(test)]
pub mod tests_func_inline {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{block_fuse, constant_fold, deadcode_elimination, inst_combine, mem2reg},
        },
        utils::diff::diff,
    };

    #[test]
    fn test_trivial_merge() {
        let code = r#"
        int main() {
            putint(1);
            if (2 > 5) {
                putint(2);
            } else {
                putint(3);
            }
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        deadcode_elimination::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
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
        define i32 @main() {
        [-] entry:
        [-] call void @putint(i32 1)
        [-] br label %cond0
        [-] 
        [-] cond0:
        [-] br label %alt2
        [-] 
        alt2:
        [+] call void @putint(i32 1)
        call void @putint(i32 3)
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
    fn test_single_br_merge() {
        let code = r#"
        int main() {
            putint(1);
            if (2 > 5) {
                putint(2);
            }
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        deadcode_elimination::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        constant_fold::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        block_fuse::optimize_program(&mut program).unwrap();
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
        define i32 @main() {
        [-] entry:
        [-] call void @putint(i32 1)
        [-] br label %cond0
        [-] 
        [-] cond0:
        [-] br label %alt2
        [-] 
        alt2:
        [-] br label %final3
        [-] 
        [-] final3:
        [+] call void @putint(i32 1)
        br label %exit

        exit:
        ret i32 0


        }
        "###);
    }
}
