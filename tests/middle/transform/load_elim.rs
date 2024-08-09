#[cfg(test)]
pub mod tests_load_elim {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            analysis::{effect_analysis::EffectAnalysis, memory_ssa::MemorySSA},
            irgen::gen,
            transform::{dead_code_elim, load_elim, mem2reg, simple_gvn},
        },
        utils::diff::diff,
    };

    #[test]
    fn test_trivial_load() {
        let code = r#"
        int a;
        int main() {
            a = 2;
            return a;
        }
        "#;

        // Check before optimization
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        let mut memory_ssa = MemorySSA::new(&program, &effect_analysis);
        let llvm_before = program.module.gen_llvm_ir();

        // Check after optimization
        load_elim::optimize_program(&mut program, &mut memory_ssa).unwrap();
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        "###);
    }

    #[test]
    fn test_array_non_overlap() {
        let code = r#"
        int a[3][3];
        int main() {
            int x = getint();
            a[1][1] = 1;
            a[x][2] = 2;
            return a[1][1];
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
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        "###);
    }

    #[test]
    fn test_array_overlap() {
        let code = r#"
        int a[3][3];
        int main() {
            int x = getint();
            a[1][1] = 1;
            a[x][x] = 2;
            return a[1][1];
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
        let llvm_after = program.module.gen_llvm_ir();
        assert_snapshot!(diff(&llvm_before, &llvm_after),@r###"
        "###);
    }
}
