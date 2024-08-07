#[cfg(test)]
pub mod tests_func_inline {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{constant_fold, deadcode_elimination, func_inline, mem2reg},
        },
        utils::diff::diff,
    };

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
        deadcode_elimination::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        func_inline::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
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
        %Add_8 = add i32 %x, 1
        br label %exit

        exit:
        ret i32 %Add_8


        }
        define i32 @main() {
        entry:
        [-] %call_16 = call i32 @f(i32 5)
        [+] br label %inlined_entry_0
        [+] 
        [+] inlined_entry_0:
        [+] br label %inlined_exit_1
        [+] 
        [+] inlined_exit_1:
        [+] br label %split_entry_2
        [+] 
        [+] split_entry_2:
        br label %exit

        exit:
        [-] ret i32 %call_16
        [+] ret i32 6


        }
        "###);
    }
}
