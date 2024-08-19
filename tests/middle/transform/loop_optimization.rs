#[cfg(test)]
pub mod tests_make_parallel {

    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            irgen::gen,
            transform::{
                dead_code_elim, inst_combine, loop_optimization, mem2reg, redundance_elim,
            },
        },
        utils::diff::diff,
    };

    #[test]
    fn test_use_arr() {
        let code = r#"
        int A[9];
        int B[9];
        int main() {
            int n = getarray(A);
            int i = 0;
            while (i < A[n - 1]) {
                B[i] = B[i] + 1;
                i = i + 1;
            }
            putarray(n, A);
            return 0;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        redundance_elim::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        loop_optimization::optimize_program(&mut program).unwrap();
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after), @r###"
        "###);
    }
}
