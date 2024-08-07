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
        "###);
    }
}
