#[cfg(test)]
pub mod tests_effect_analysis {
    use insta::assert_snapshot;

    use compiler::{
        frontend::parse,
        middle::{
            analysis::effect_analysis::EffectAnalysis,
            irgen::gen,
            transform::{constant_fold, dead_code_elim, inst_combine, mem2reg},
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
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        assert_snapshot!(effect_analysis.dump(), @r###"
        store i32 6, ptr @a:
          def: @a
          use: 

        %load_6 = load i32, ptr @a:
          def: 
          use: @a

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
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        assert_snapshot!(effect_analysis.dump(), @r###"
        %load_5 = load i32, ptr @y:
          def: 
          use: @y

        store i32 2, ptr @y:
          def: @y
          use: 

        %call_14 = call i32 @f():
          def: 
          use: @y

        call void @putint(i32 %call_14):
          def: 
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
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        assert_snapshot!(effect_analysis.dump(), @r###"
        store i32 1, ptr @x:
          def: @x
          use: 

        %call_6 = call i32 @main():
          def: @x
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
        inst_combine::optimize_program(&mut program).unwrap();
        dead_code_elim::optimize_program(&mut program).unwrap();
        let effect_analysis = EffectAnalysis::new(&program);
        assert_snapshot!(effect_analysis.dump(), @r###"
        %load_5 = load i32, ptr @y:
          def: 
          use: @y

        store i32 %load_5, ptr @x:
          def: @x
          use: 

        %call_7 = call i32 @g():
          def: @x, @y
          use: @x, @y

        %load_15 = load i32, ptr @x:
          def: 
          use: @x

        store i32 %load_15, ptr @y:
          def: @y
          use: 

        %call_17 = call i32 @f():
          def: @x, @y
          use: @x, @y

        %call_25 = call i32 @f():
          def: @x, @y
          use: @x, @y

        "###);
    }
}