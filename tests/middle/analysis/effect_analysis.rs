#[cfg(test)]
pub mod tests_effect_analysis {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            analysis::effect_analysis::EffectAnalysis,
            irgen::gen,
            transform::{constant_fold, dead_code_elim, symbolic_eval, mem2reg},
        },
    };

    #[test]
    fn test_trivial() {
        let code = r#"
        int a = 9;
        int main() {
            a = 6;
            return a;
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        symbolic_eval::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        assert_snapshot!(effect_analysis.dump_inst(), @r###"
        store i32 6, ptr @a:
            def: @a
            use: 

        %load_6 = load i32, ptr @a:
            def: 
            use: @a

        "###);
    }

    #[test]
    fn test_memset() {
        let code = r#"
        int main() {
            int a[3] = {};
            return a[0];
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        symbolic_eval::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        assert_snapshot!(effect_analysis.dump_inst(), @r###"
        call void @llvm.memset.p0.i32([3 x i32]* %alloca_5, i8 0, i32 12, i1 false):
            def: %alloca_5
            use: 

        %load_8 = load i32, ptr %getelementptr_7:
            def: 
            use: %getelementptr_7

        "###);
    }

    #[test]
    fn test_function_param() {
        let code = r#"
        int f(int x[]) {
            x[2] = 3;
        }
        int main() {
            int a[3];
            f(a);
            return 0;
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        symbolic_eval::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        assert_snapshot!(effect_analysis.dump_inst(), @r###"
        store i32 3, ptr %getelementptr_8:
            def: %getelementptr_8
            use: 

        %call_18 = call i32 @f(i32* %getelementptr_17):
            def: all
            use: 

        "###);
    }

    #[test]
    fn test_function_call() {
        let code = r#"
        int x = 1;
        int y = 2;
        int f() {
            return y;
        }
        int main() {
            y = 2;
            putint(f());
            y = 3;
            return y;
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        symbolic_eval::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        assert_snapshot!(effect_analysis.dump_inst(), @r###"
        %load_5 = load i32, ptr @y:
            def: 
            use: @y

        store i32 2, ptr @y:
            def: @y
            use: 

        %call_14 = call i32 @f():
            def: all
            use: 

        store i32 3, ptr @y:
            def: @y
            use: 

        %load_17 = load i32, ptr @y:
            def: 
            use: @y

        "###);
    }

    #[test]
    fn test_recursive_function() {
        let code = r#"
        int x = 1;
        int y = 2;
        int main() {
            x = 1;
            return main();
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        symbolic_eval::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        assert_snapshot!(effect_analysis.dump_inst(), @r###"
        store i32 1, ptr @x:
            def: @x
            use: 

        %call_6 = call i32 @main():
            def: all
            use: 

        "###);
    }

    #[test]
    fn test_mutual_recursive_function() {
        let code = r#"
        int x = 1;
        int y = 2;
        int f() {
            x = y;
            return g();
        }
        int g() {
            y = x;
            return f();
        }
        int main() {
            return f();
        }
        "#;
        let parsed = parse(code).unwrap();
        let mut program = gen(&parsed).unwrap();
        mem2reg::optimize_program(&mut program).unwrap();
        constant_fold::optimize_program(&mut program).unwrap();
        symbolic_eval::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        assert_snapshot!(effect_analysis.dump_inst(), @r###"
        %load_5 = load i32, ptr @y:
            def: 
            use: @y

        store i32 %load_5, ptr @x:
            def: @x
            use: 

        %call_7 = call i32 @g():
            def: all
            use: 

        %load_15 = load i32, ptr @x:
            def: 
            use: @x

        store i32 %load_15, ptr @y:
            def: @y
            use: 

        %call_17 = call i32 @f():
            def: all
            use: 

        %call_25 = call i32 @f():
            def: all
            use: 

        "###);
    }
}
