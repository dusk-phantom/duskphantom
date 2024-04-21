use crate::{errors::MiddelError, frontend, middle};
use program_kit::ProgramKit;
use std::collections::HashMap;

mod constant;
mod function_kit;
mod program_kit;
mod util;
mod value;
mod value_type;

/// Generate middle IR from a frontend AST
pub fn gen(program: &frontend::Program) -> Result<middle::Program, MiddelError> {
    let mut result = middle::Program::new();
    ProgramKit {
        program: &mut result,
        env: HashMap::new(),
        fun_env: HashMap::new(),
    }
    .gen(program)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frontend::program::parse;

    #[test]
    fn test_normal() {
        let code = r#"
            int main() {
                int a = 1;
                int b = 2;
                int c = a + b;
                return c;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(1)))), Decl(Var(Int32, \"b\", Some(Int32(2)))), Decl(Var(Int32, \"c\", Some(Binary(Add, Var(\"a\"), Var(\"b\"))))), Return(Some(Var(\"c\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        // No constant folding, because a variable can be re-assigned in SysY
        // This behaviour is consistent with `clang -S -emit-llvm xxx.c`
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%alloca_3 = alloca i32\nstore i32 2, ptr %alloca_3\n%alloca_5 = alloca i32\n%load_6 = load i32, ptr %alloca_1\n%load_7 = load i32, ptr %alloca_3\n%Add_8 = add i32, %load_6, %load_7\nstore i32 %Add_8, ptr %alloca_5\n%load_10 = load i32, ptr %alloca_5\nret %load_10\n\n\n}\n");
    }

    #[test]
    fn test_if() {
        let code = r#"
            int main() {
                int a = 1;
                int b = 2;
                if (a < b) {
                    a = 3;
                } else {
                    a = 4;
                }
                return a;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(1)))), Decl(Var(Int32, \"b\", Some(Int32(2)))), If(Binary(Lt, Var(\"a\"), Var(\"b\")), Block([Expr(Some(Var(\"a\")), Int32(3))]), Block([Expr(Some(Var(\"a\")), Int32(4))])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%alloca_3 = alloca i32\nstore i32 2, ptr %alloca_3\nbr label %cond0\n\n%cond0:\n%load_10 = load i32, ptr %alloca_1\n%load_11 = load i32, ptr %alloca_3\n%icmp_12 = icmp slt i32 %load_10, %load_11\nbr i1 %icmp_12, label %then1, label %alt2\n\n%then1:\nstore i32 3, ptr %alloca_1\nbr label %final3\n\n%alt2:\nstore i32 4, ptr %alloca_1\nbr label %final3\n\n%final3:\n%load_18 = load i32, ptr %alloca_1\nret %load_18\n\n\n}\n");
    }

    #[test]
    fn test_while() {
        let code = r#"
            int main() {
                int a = 0;
                while (a < 10) {
                    a = a + 1;
                }
                return a;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1)))])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %cond0\n\n%cond0:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body1, label %final2\n\n%body1:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %cond0\n\n%final2:\n%load_14 = load i32, ptr %alloca_1\nret %load_14\n\n\n}\n");
    }

    #[test]
    fn test_do_while() {
        let code = r#"
            int main() {
                int a = 0;
                do {
                    a = a + 1;
                } while (a < 10);
                return a;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), DoWhile(Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1)))]), Binary(Lt, Var(\"a\"), Int32(10))), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %body0\n\n%body0:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %cond1\n\n%cond1:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body0, label %final2\n\n%final2:\n%load_14 = load i32, ptr %alloca_1\nret %load_14\n\n\n}\n");
    }

    #[test]
    fn test_break() {
        let code = r#"
            int main() {
                int a = 0;
                while (a < 10) {
                    a = a + 1;
                    break;
                }
                return a;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1))), Break])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        // There are two `br` in `%body` block
        // Not preventing this can make `irgen` code simpler
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %cond0\n\n%cond0:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body1, label %final2\n\n%body1:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %final2\nbr label %final2\n\n%final2:\n%load_15 = load i32, ptr %alloca_1\nret %load_15\n\n\n}\n");
    }

    #[test]
    fn test_continue() {
        let code = r#"
            int main() {
                int a = 0;
                while (a < 10) {
                    a = a + 1;
                    continue;
                }
                return a;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1))), Continue])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %cond0\n\n%cond0:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body1, label %final2\n\n%body1:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %cond0\nbr label %cond0\n\n%final2:\n%load_15 = load i32, ptr %alloca_1\nret %load_15\n\n\n}\n");
    }

    #[test]
    fn test_control_flow() {
        let code = r#"
            int main() {
                int a = 0;
                while (a < 10) {
                    a = a + 1;
                    if (a == 5) {
                        break;
                    } else {
                        continue;
                    }
                }
                return a;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1))), If(Binary(Eq, Var(\"a\"), Int32(5)), Block([Break]), Block([Continue]))])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %cond0\n\n%cond0:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body1, label %final2\n\n%body1:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %cond3\nbr label %cond3\n\n%final2:\n%load_26 = load i32, ptr %alloca_1\nret %load_26\n\n%cond3:\n%load_18 = load i32, ptr %alloca_1\n%icmp_19 = icmp eq i32 %load_18, 5\nbr i1 %icmp_19, label %then4, label %alt5\n\n%then4:\nbr label %final2\nbr label %final2\n\n%alt5:\nbr label %cond0\nbr label %cond0\n\n\n}\n");
    }

    #[test]
    fn test_global_variable() {
        let code = r#"
            int x = 4;
            int y = 8;
            int main() {
                x = x + y;
                return x;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Var(Int32, \"x\", Some(Int32(4))), Var(Int32, \"y\", Some(Int32(8))), Func(Function(Int32, []), \"main\", Some(Block([Expr(Some(Var(\"x\")), Binary(Add, Var(\"x\"), Var(\"y\"))), Return(Some(Var(\"x\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "@x = dso_local global i32 [4]\n@y = dso_local global i32 [8]\ndefine i32 @main() {\n%entry:\n%load_1 = load i32, ptr @x\n%load_2 = load i32, ptr @y\n%Add_3 = add i32, %load_1, %load_2\nstore i32 %Add_3, ptr @x\n%load_5 = load i32, ptr @x\nret %load_5\n\n\n}\n");
    }

    #[test]
    fn test_conv() {
        let code = r#"
            int main() {
                int x = 1;
                float y = 2.0;
                float z = x + y;
                return z;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"x\", Some(Int32(1)))), Decl(Var(Float32, \"y\", Some(Float32(2.0)))), Decl(Var(Float32, \"z\", Some(Binary(Add, Var(\"x\"), Var(\"y\"))))), Return(Some(Var(\"z\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%alloca_3 = alloca float\nstore float 2, ptr %alloca_3\n%alloca_5 = alloca float\n%load_6 = load i32, ptr %alloca_1\n%itofp_7 = sitofp i32 %load_6 to float\n%load_8 = load float, ptr %alloca_3\n%FAdd_9 = fadd float, %itofp_7, %load_8\nstore float %FAdd_9, ptr %alloca_5\n%load_11 = load float, ptr %alloca_5\n%fptoi_12 = fptosi float %load_11 to i32\nret %fptoi_12\n\n\n}\n");
    }

    #[test]
    fn test_zext() {
        let code = r#"
            int main() {
                return (3 > 1) + (4 > 2);
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Return(Some(Binary(Add, Binary(Gt, Int32(3), Int32(1)), Binary(Gt, Int32(4), Int32(2)))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%icmp_1 = icmp sgt i32 3, 1\n%icmp_2 = icmp sgt i32 4, 2\n%zext_3 = zext i1 %icmp_1 to i32\n%zext_4 = zext i1 %icmp_2 to i32\n%Add_5 = add i32, %zext_3, %zext_4\nret %Add_5\n\n\n}\n");
    }

    #[test]
    fn test_param() {
        let code = r#"
            int main(int arg) {
                return arg;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, [TypedIdent { ty: Int32, id: Some(\"arg\") }]), \"main\", Some(Block([Return(Some(Var(\"arg\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main(i32 arg) {\n%entry:\n%alloca_1 = alloca i32\nstore i32 %arg, ptr %alloca_1\n%load_3 = load i32, ptr %alloca_1\nret %load_3\n\n\n}\n");
    }

    #[test]
    fn test_call() {
        let code = r#"
            int main() {
                return f(1.7);
            }

            int f(int x) {
                return x + 1;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Return(Some(Call(Var(\"f\"), [Float32(1.7)])))]))), Func(Function(Int32, [TypedIdent { ty: Int32, id: Some(\"x\") }]), \"f\", Some(Block([Return(Some(Binary(Add, Var(\"x\"), Int32(1))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%fptoi_1 = fptosi float 1.7 to i32\n%call_2 = call i32 @f(i32 %fptoi_1)\nret %call_2\n\n\n}\ndefine i32 @f(i32 x) {\n%entry:\n%alloca_5 = alloca i32\nstore i32 %x, ptr %alloca_5\n%load_7 = load i32, ptr %alloca_5\n%Add_8 = add i32, %load_7, 1\nret %Add_8\n\n\n}\n");
    }

    #[test]
    fn test_nested_call() {
        let code = r#"
            int main() {
                return f(f(1));
            }

            int f(int x) {
                return x + 1;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Return(Some(Call(Var(\"f\"), [Call(Var(\"f\"), [Int32(1)])])))]))), Func(Function(Int32, [TypedIdent { ty: Int32, id: Some(\"x\") }]), \"f\", Some(Block([Return(Some(Binary(Add, Var(\"x\"), Int32(1))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%call_1 = call i32 @f(i32 1)\n%call_2 = call i32 @f(i32 %call_1)\nret %call_2\n\n\n}\ndefine i32 @f(i32 x) {\n%entry:\n%alloca_5 = alloca i32\nstore i32 %x, ptr %alloca_5\n%load_7 = load i32, ptr %alloca_5\n%Add_8 = add i32, %load_7, 1\nret %Add_8\n\n\n}\n");
    }

    #[test]
    fn test_constant() {
        let code = r#"
            const float PI = 3.1415926;

            int main() {
                return PI;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Const(Float32, \"PI\", Some(Float32(3.1415925))), Func(Function(Int32, []), \"main\", Some(Block([Return(Some(Var(\"PI\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        // Constant folding is not avaliable yet
        assert_eq!(llvm_ir, "@PI = dso_local constant float [3.1415925]\ndefine i32 @main() {\n%entry:\n%load_1 = load float, ptr @PI\n%fptoi_2 = fptosi float %load_1 to i32\nret %fptoi_2\n\n\n}\n");
    }

    #[test]
    fn test_number_condition() {
        let code = r#"
            int main() {
                float a = 5.4;
                int b = 8;
                int z = 0;
                if (a) {
                    z = 1;
                }
                if (b) {
                    z = 2;
                }
                return z;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Float32, \"a\", Some(Float32(5.4)))), Decl(Var(Int32, \"b\", Some(Int32(8)))), Decl(Var(Int32, \"z\", Some(Int32(0)))), If(Var(\"a\"), Block([Expr(Some(Var(\"z\")), Int32(1))]), Block([])), If(Var(\"b\"), Block([Expr(Some(Var(\"z\")), Int32(2))]), Block([])), Return(Some(Var(\"z\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca float\nstore float 5.4, ptr %alloca_1\n%alloca_3 = alloca i32\nstore i32 8, ptr %alloca_3\n%alloca_5 = alloca i32\nstore i32 0, ptr %alloca_5\nbr label %cond0\n\n%cond0:\n%load_12 = load float, ptr %alloca_1\n%fcmp_13 = fcmp une float %load_12, 0\nbr i1 %fcmp_13, label %then1, label %alt2\n\n%then1:\nstore i32 1, ptr %alloca_5\nbr label %final3\n\n%alt2:\nbr label %final3\n\n%final3:\nbr label %cond4\n\n%cond4:\n%load_23 = load i32, ptr %alloca_3\n%icmp_24 = icmp ne i32 %load_23, 0\nbr i1 %icmp_24, label %then5, label %alt6\n\n%then5:\nstore i32 2, ptr %alloca_5\nbr label %final7\n\n%alt6:\nbr label %final7\n\n%final7:\n%load_29 = load i32, ptr %alloca_5\nret %load_29\n\n\n}\n");
    }

    #[test]
    fn test_unary() {
        let code = r#"
            int main() {
                int x = 1;
                return !+-x;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"x\", Some(Int32(1)))), Return(Some(Unary(Not, Unary(Pos, Unary(Neg, Var(\"x\"))))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%load_3 = load i32, ptr %alloca_1\n%Sub_4 = sub i32, 0, %load_3\n%icmp_5 = icmp ne i32 %Sub_4, 0\n%Xor_6 = xor i1, %icmp_5, true\n%zext_7 = zext i1 %Xor_6 to i32\nret %zext_7\n\n\n}\n");
    }

    #[test]
    fn test_cmp() {
        let code = r#"
            int main() {
                bool x = 1 < 2;
                bool y = 1 < 1.1;
                return x && y;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Boolean, \"x\", Some(Binary(Lt, Int32(1), Int32(2))))), Decl(Var(Boolean, \"y\", Some(Binary(Lt, Int32(1), Float32(1.1))))), Return(Some(Binary(And, Var(\"x\"), Var(\"y\"))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i1\n%icmp_2 = icmp slt i32 1, 2\nstore i1 %icmp_2, ptr %alloca_1\n%alloca_4 = alloca i1\n%itofp_5 = sitofp i32 1 to float\n%fcmp_6 = fcmp ult float %itofp_5, 1.1\nstore i1 %fcmp_6, ptr %alloca_4\n%load_8 = load i1, ptr %alloca_1\n%load_9 = load i1, ptr %alloca_4\n%And_10 = and i1, %load_8, %load_9\n%zext_11 = zext i1 %And_10 to i32\nret %zext_11\n\n\n}\n");
    }

    #[test]
    fn test_constant_fold() {
        let code = r#"
            const int x = 1 + 3;
            const int y = x * x;
            int main() {
                return x + y;
            }
        "#;
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Const(Int32, \"x\", Some(Binary(Add, Int32(1), Int32(3)))), Const(Int32, \"y\", Some(Binary(Mul, Var(\"x\"), Var(\"x\")))), Func(Function(Int32, []), \"main\", Some(Block([Return(Some(Binary(Add, Var(\"x\"), Var(\"y\"))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(
            llvm_ir,
            "@x = dso_local constant i32 [4]\n@y = dso_local constant i32 [16]\ndefine i32 @main() {\n%entry:\n%load_1 = load i32, ptr @x\n%load_2 = load i32, ptr @y\n%Add_3 = add i32, %load_1, %load_2\nret %Add_3\n\n\n}\n"
        );
    }
}
