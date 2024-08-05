#[cfg(test)]
pub mod tests_constant_fold {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{constant_fold, mem2reg},
        },
        utils::diff::diff,
    };

    #[test]
    fn test_normal() {
        let code = r#"
        int main(){
            int x = 8; // 8
            x = x * 2; // 16
            if (x > 5) {
                putint(x);
            }
            x = x / 3; // 5
            x = x + 4; // 9
            return x;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        constant_fold::optimize_program(&mut program).unwrap();
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
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        [-] %Mul_8 = mul i32 8, 2
        br label %cond0

        cond0:
        [-] %icmp_16 = icmp sgt i32 %Mul_8, 5
        [-] br i1 %icmp_16, label %then1, label %alt2
        [+] br i1 true, label %then1, label %alt2

        then1:
        [-] call void @putint(i32 %Mul_8)
        [+] call void @putint(i32 16)
        br label %final3

        alt2:
        br label %final3

        final3:
        [-] %SDiv_23 = sdiv i32 %Mul_8, 3
        [-] %Add_26 = add i32 %SDiv_23, 4
        br label %exit

        exit:
        [-] ret i32 %Add_26
        [+] ret i32 9


        }
        "###);
    }
}
