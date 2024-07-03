use crate::{frontend, middle};
use anyhow::Result;
use program_kit::ProgramKit;
use std::collections::HashMap;

mod constant;
mod function_kit;
mod gen_binary;
mod gen_const_binary;
mod gen_const_expr;
mod gen_const_unary;
mod gen_expr;
mod gen_global_decl;
mod gen_impl;
mod gen_inner_decl;
mod gen_library_function;
mod gen_stmt;
mod gen_unary;
mod program_kit;
mod value;
mod value_type;

/// Generate middle IR from a frontend AST
pub fn gen(program: &frontend::Program) -> Result<middle::Program> {
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
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            store i32 1, ptr %alloca_1
            %alloca_3 = alloca i32
            store i32 2, ptr %alloca_3
            %alloca_5 = alloca i32
            %load_6 = load i32, ptr %alloca_1
            %load_7 = load i32, ptr %alloca_3
            %Add_8 = add i32, %load_6, %load_7
            store i32 %Add_8, ptr %alloca_5
            %load_10 = load i32, ptr %alloca_5
            ret i32 %load_10
            
            
            }
        "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(1)))), Decl(Var(Int32, \"b\", Some(Int32(2)))), Decl(Var(Int32, \"c\", Some(Binary(Add, Var(\"a\"), Var(\"b\"))))), Return(Some(Var(\"c\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        // No constant folding, because a variable can be re-assigned in SysY
        // This behaviour is consistent with `clang -S -emit-llvm xxx.c`
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            store i32 1, ptr %alloca_1
            %alloca_3 = alloca i32
            store i32 2, ptr %alloca_3
            br label %cond0
            
            %cond0:
            %load_10 = load i32, ptr %alloca_1
            %load_11 = load i32, ptr %alloca_3
            %icmp_12 = icmp slt i32 %load_10, %load_11
            br i1 %icmp_12, label %then1, label %alt2
            
            %then1:
            store i32 3, ptr %alloca_1
            br label %final3
            
            %alt2:
            store i32 4, ptr %alloca_1
            br label %final3
            
            %final3:
            %load_18 = load i32, ptr %alloca_1
            ret i32 %load_18
            
            
            }
        "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(1)))), Decl(Var(Int32, \"b\", Some(Int32(2)))), If(Binary(Lt, Var(\"a\"), Var(\"b\")), Block([Expr(Some(Var(\"a\")), Int32(3))]), Block([Expr(Some(Var(\"a\")), Int32(4))])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            store i32 0, ptr %alloca_1
            br label %cond0
            
            %cond0:
            %load_11 = load i32, ptr %alloca_1
            %icmp_12 = icmp slt i32 %load_11, 10
            br i1 %icmp_12, label %body1, label %final2
            
            %body1:
            %load_7 = load i32, ptr %alloca_1
            %Add_8 = add i32, %load_7, 1
            store i32 %Add_8, ptr %alloca_1
            br label %cond0
            
            %final2:
            %load_14 = load i32, ptr %alloca_1
            ret i32 %load_14
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1)))])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            store i32 0, ptr %alloca_1
            br label %body0
            
            %body0:
            %load_7 = load i32, ptr %alloca_1
            %Add_8 = add i32, %load_7, 1
            store i32 %Add_8, ptr %alloca_1
            br label %cond1
            
            %cond1:
            %load_11 = load i32, ptr %alloca_1
            %icmp_12 = icmp slt i32 %load_11, 10
            br i1 %icmp_12, label %body0, label %final2
            
            %final2:
            %load_14 = load i32, ptr %alloca_1
            ret i32 %load_14
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), DoWhile(Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1)))]), Binary(Lt, Var(\"a\"), Int32(10))), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            store i32 0, ptr %alloca_1
            br label %cond0
            
            %cond0:
            %load_11 = load i32, ptr %alloca_1
            %icmp_12 = icmp slt i32 %load_11, 10
            br i1 %icmp_12, label %body1, label %final2
            
            %body1:
            %load_7 = load i32, ptr %alloca_1
            %Add_8 = add i32, %load_7, 1
            store i32 %Add_8, ptr %alloca_1
            br label %final2
            
            %final2:
            %load_14 = load i32, ptr %alloca_1
            ret i32 %load_14
            
            
            }
        "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1))), Break])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        // There are two `br` in `%body` block
        // Not preventing this can make `irgen` code simpler
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            store i32 0, ptr %alloca_1
            br label %cond0
            
            %cond0:
            %load_11 = load i32, ptr %alloca_1
            %icmp_12 = icmp slt i32 %load_11, 10
            br i1 %icmp_12, label %body1, label %final2
            
            %body1:
            %load_7 = load i32, ptr %alloca_1
            %Add_8 = add i32, %load_7, 1
            store i32 %Add_8, ptr %alloca_1
            br label %cond0
            
            %final2:
            %load_14 = load i32, ptr %alloca_1
            ret i32 %load_14
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1))), Continue])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            store i32 0, ptr %alloca_1
            br label %cond0
            
            %cond0:
            %load_21 = load i32, ptr %alloca_1
            %icmp_22 = icmp slt i32 %load_21, 10
            br i1 %icmp_22, label %body1, label %final2
            
            %body1:
            %load_7 = load i32, ptr %alloca_1
            %Add_8 = add i32, %load_7, 1
            store i32 %Add_8, ptr %alloca_1
            br label %cond3
            
            %final2:
            %load_24 = load i32, ptr %alloca_1
            ret i32 %load_24
            
            %cond3:
            %load_15 = load i32, ptr %alloca_1
            %icmp_16 = icmp eq i32 %load_15, 5
            br i1 %icmp_16, label %then4, label %alt5
            
            %then4:
            br label %final2
            
            %alt5:
            br label %cond0
            
            
            }
        "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1))), If(Binary(Eq, Var(\"a\"), Int32(5)), Block([Break]), Block([Continue]))])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_default_exit() {
        let code = r#"
            int main() {
                int a = 0;
                while (a < 10) {
                    a = a + 1;
                    if (a == 5) {
                        break;
                    }
                }
                return a;
            }
        "#;
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            store i32 0, ptr %alloca_1
            br label %cond0
            
            %cond0:
            %load_21 = load i32, ptr %alloca_1
            %icmp_22 = icmp slt i32 %load_21, 10
            br i1 %icmp_22, label %body1, label %final2
            
            %body1:
            %load_7 = load i32, ptr %alloca_1
            %Add_8 = add i32, %load_7, 1
            store i32 %Add_8, ptr %alloca_1
            br label %cond3
            
            %final2:
            %load_24 = load i32, ptr %alloca_1
            ret i32 %load_24
            
            %cond3:
            %load_15 = load i32, ptr %alloca_1
            %icmp_16 = icmp eq i32 %load_15, 5
            br i1 %icmp_16, label %then4, label %alt5
            
            %then4:
            br label %final2
            
            %alt5:
            br label %final6
            
            %final6:
            br label %cond0
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"a\", Some(Int32(0)))), While(Binary(Lt, Var(\"a\"), Int32(10)), Block([Expr(Some(Var(\"a\")), Binary(Add, Var(\"a\"), Int32(1))), If(Binary(Eq, Var(\"a\"), Int32(5)), Block([Break]), Block([]))])), Return(Some(Var(\"a\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_dead_code() {
        let code = r#"
            int main() {
                while (1) {
                    break;
                    continue;
                    return 0;
                    break;
                    return 0;
                    continue;
                    continue;
                    return 1;
                    break;
                }
                return 0;
            }
        "#;
        let expected = r#"define i32 @main() {
            %entry:
            br label %cond0
            
            %cond0:
            %icmp_6 = icmp ne i32 1, 0
            br i1 %icmp_6, label %body1, label %final2
            
            %body1:
            br label %final2
            
            %final2:
            ret i32 0
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([While(Int32(1), Block([Break, Continue, Return(Some(Int32(0))), Break, Return(Some(Int32(0))), Continue, Continue, Return(Some(Int32(1))), Break])), Return(Some(Int32(0)))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"@x = dso_local global i32 4
            @y = dso_local global i32 8
            define i32 @main() {
            %entry:
            %load_1 = load i32, ptr @x
            %load_2 = load i32, ptr @y
            %Add_3 = add i32, %load_1, %load_2
            store i32 %Add_3, ptr @x
            %load_5 = load i32, ptr @x
            ret i32 %load_5
            
            
            }
        "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Var(Int32, \"x\", Some(Int32(4))), Var(Int32, \"y\", Some(Int32(8))), Func(Function(Int32, []), \"main\", Some(Block([Expr(Some(Var(\"x\")), Binary(Add, Var(\"x\"), Var(\"y\"))), Return(Some(Var(\"x\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            store i32 1, ptr %alloca_1
            %alloca_3 = alloca float
            store float 2, ptr %alloca_3
            %alloca_5 = alloca float
            %load_6 = load i32, ptr %alloca_1
            %itofp_7 = sitofp i32 %load_6 to float
            %load_8 = load float, ptr %alloca_3
            %FAdd_9 = fadd float, %itofp_7, %load_8
            store float %FAdd_9, ptr %alloca_5
            %load_11 = load float, ptr %alloca_5
            %fptoi_12 = fptosi float %load_11 to i32
            ret i32 %fptoi_12
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"x\", Some(Int32(1)))), Decl(Var(Float32, \"y\", Some(Float32(2.0)))), Decl(Var(Float32, \"z\", Some(Binary(Add, Var(\"x\"), Var(\"y\"))))), Return(Some(Var(\"z\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_zext() {
        let code = r#"
            int main() {
                return (3 > 1) + (4 > 2);
            }
        "#;
        let expected = r#"define i32 @main() {
            %entry:
            %icmp_1 = icmp sgt i32 3, 1
            %icmp_2 = icmp sgt i32 4, 2
            %zext_3 = zext i1 %icmp_1 to i32
            %zext_4 = zext i1 %icmp_2 to i32
            %Add_5 = add i32, %zext_3, %zext_4
            ret i32 %Add_5
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Return(Some(Binary(Add, Binary(Gt, Int32(3), Int32(1)), Binary(Gt, Int32(4), Int32(2)))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_param() {
        let code = r#"
            int main(int arg) {
                return arg;
            }
        "#;
        let expected = r#"define i32 @main(i32 arg) {
            %entry:
            %alloca_1 = alloca i32
            store i32 %arg, ptr %alloca_1
            %load_3 = load i32, ptr %alloca_1
            ret i32 %load_3
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, [TypedIdent { ty: Int32, id: Some(\"arg\") }]), \"main\", Some(Block([Return(Some(Var(\"arg\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %fptoi_1 = fptosi float 1.7 to i32
            %call_2 = call i32 @f(i32 %fptoi_1)
            ret i32 %call_2
            
            
            }
            define i32 @f(i32 x) {
            %entry:
            %alloca_5 = alloca i32
            store i32 %x, ptr %alloca_5
            %load_7 = load i32, ptr %alloca_5
            %Add_8 = add i32, %load_7, 1
            ret i32 %Add_8
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Return(Some(Call(Var(\"f\"), [Float32(1.7)])))]))), Func(Function(Int32, [TypedIdent { ty: Int32, id: Some(\"x\") }]), \"f\", Some(Block([Return(Some(Binary(Add, Var(\"x\"), Int32(1))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %call_1 = call i32 @f(i32 1)
            %call_2 = call i32 @f(i32 %call_1)
            ret i32 %call_2
            
            
            }
            define i32 @f(i32 x) {
            %entry:
            %alloca_5 = alloca i32
            store i32 %x, ptr %alloca_5
            %load_7 = load i32, ptr %alloca_5
            %Add_8 = add i32, %load_7, 1
            ret i32 %Add_8
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Return(Some(Call(Var(\"f\"), [Call(Var(\"f\"), [Int32(1)])])))]))), Func(Function(Int32, [TypedIdent { ty: Int32, id: Some(\"x\") }]), \"f\", Some(Block([Return(Some(Binary(Add, Var(\"x\"), Int32(1))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_constant() {
        let code = r#"
            const float PI = 3.1415926;

            int main() {
                return PI;
            }
        "#;
        let expected = r#"@PI = dso_local constant float 3.1415925
            define i32 @main() {
            %entry:
            %load_1 = load float, ptr @PI
            %fptoi_2 = fptosi float %load_1 to i32
            ret i32 %fptoi_2
            
            
            }
        "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Const(Float32, \"PI\", Some(Float32(3.1415925))), Func(Function(Int32, []), \"main\", Some(Block([Return(Some(Var(\"PI\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_constant_array() {
        let code = r#"
            const float A[3][2][2] = {{1}, 1, 4, 5, 1, {4}};

            int main() {
                return A[0][0][0];
            }
        "#;
        let expected = r#"@A = dso_local constant [3 x [2 x [2 x float]]] [[2 x [2 x float]] [[2 x float] [float 1, float 0], [2 x float] [float 0, float 0]], [2 x [2 x float]] [[2 x float] [float 1, float 4], [2 x float] [float 5, float 1]], [2 x [2 x float]] [[2 x float] [float 4, float 0], [2 x float] [float 0, float 0]]]
            define i32 @main() {
            %entry:
            %getelementptr_1 = getelementptr [3 x [2 x [2 x float]]], ptr @A, i32 0, i32 0
            %getelementptr_2 = getelementptr [2 x [2 x float]], ptr %getelementptr_1, i32 0, i32 0
            %getelementptr_3 = getelementptr [2 x float], ptr %getelementptr_2, i32 0, i32 0
            %load_4 = load float, ptr %getelementptr_3
            %fptoi_5 = fptosi float %load_4 to i32
            ret i32 %fptoi_5
            
            
            }
        "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_variable_array() {
        let code = r#"
            int main() {
                float A[2][2][2] = {1, 1, 4, 5, {{1}, 4}};
                return A[1][1][1];
            }
        "#;
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca [2 x [2 x [2 x float]]]
            %getelementptr_2 = getelementptr [2 x [2 x [2 x float]]], ptr %alloca_1, i32 0, i32 0
            %getelementptr_3 = getelementptr [2 x [2 x float]], ptr %getelementptr_2, i32 0
            %getelementptr_4 = getelementptr [2 x [2 x float]], ptr %getelementptr_3, i32 0, i32 0
            %getelementptr_5 = getelementptr [2 x float], ptr %getelementptr_4, i32 0
            %getelementptr_6 = getelementptr [2 x float], ptr %getelementptr_5, i32 0, i32 0
            %getelementptr_7 = getelementptr float, ptr %getelementptr_6, i32 0
            %itofp_8 = sitofp i32 1 to float
            store float %itofp_8, ptr %getelementptr_7
            %getelementptr_10 = getelementptr float, ptr %getelementptr_6, i32 1
            %itofp_11 = sitofp i32 1 to float
            store float %itofp_11, ptr %getelementptr_10
            %getelementptr_13 = getelementptr [2 x float], ptr %getelementptr_4, i32 1
            %getelementptr_14 = getelementptr [2 x float], ptr %getelementptr_13, i32 0, i32 0
            %getelementptr_15 = getelementptr float, ptr %getelementptr_14, i32 0
            %itofp_16 = sitofp i32 4 to float
            store float %itofp_16, ptr %getelementptr_15
            %getelementptr_18 = getelementptr float, ptr %getelementptr_14, i32 1
            %itofp_19 = sitofp i32 5 to float
            store float %itofp_19, ptr %getelementptr_18
            %getelementptr_21 = getelementptr [2 x [2 x float]], ptr %getelementptr_2, i32 1
            %getelementptr_22 = getelementptr [2 x [2 x float]], ptr %getelementptr_21, i32 0, i32 0
            %getelementptr_23 = getelementptr [2 x float], ptr %getelementptr_22, i32 0
            %getelementptr_24 = getelementptr [2 x float], ptr %getelementptr_23, i32 0, i32 0
            %getelementptr_25 = getelementptr float, ptr %getelementptr_24, i32 0
            %itofp_26 = sitofp i32 1 to float
            store float %itofp_26, ptr %getelementptr_25
            %getelementptr_28 = getelementptr [2 x float], ptr %getelementptr_22, i32 1
            %getelementptr_29 = getelementptr [2 x float], ptr %getelementptr_28, i32 0, i32 0
            %getelementptr_30 = getelementptr float, ptr %getelementptr_29, i32 0
            %itofp_31 = sitofp i32 4 to float
            store float %itofp_31, ptr %getelementptr_30
            %getelementptr_33 = getelementptr [2 x [2 x [2 x float]]], ptr %alloca_1, i32 0, i32 1
            %getelementptr_34 = getelementptr [2 x [2 x float]], ptr %getelementptr_33, i32 0, i32 1
            %getelementptr_35 = getelementptr [2 x float], ptr %getelementptr_34, i32 0, i32 1
            %load_36 = load float, ptr %getelementptr_35
            %fptoi_37 = fptosi float %load_36 to i32
            ret i32 %fptoi_37
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_assign_array() {
        let code = r#"
            int main() {
                int A[1] = {0};
                A[A[0]] = 1;
                return A[0];
            }
        "#;
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca [1 x i32]
            %getelementptr_2 = getelementptr [1 x i32], ptr %alloca_1, i32 0, i32 0
            %getelementptr_3 = getelementptr i32, ptr %getelementptr_2, i32 0
            store i32 0, ptr %getelementptr_3
            %getelementptr_5 = getelementptr [1 x i32], ptr %alloca_1, i32 0, i32 0
            %load_6 = load i32, ptr %getelementptr_5
            %getelementptr_7 = getelementptr [1 x i32], ptr %alloca_1, i32 0, i32 %load_6
            store i32 1, ptr %getelementptr_7
            %getelementptr_9 = getelementptr [1 x i32], ptr %alloca_1, i32 0, i32 0
            %load_10 = load i32, ptr %getelementptr_9
            ret i32 %load_10
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Array(Int32, 1), \"A\", Some(Pack([Int32(0)])))), Expr(Some(Index(Var(\"A\"), Index(Var(\"A\"), Int32(0)))), Int32(1)), Return(Some(Index(Var(\"A\"), Int32(0))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_pointer() {
        let code = r#"
            int main() {
                int arr[1] = {8};
                f(arr);
                putarray(1, arr);
                return 0;
            }

            int f(int a[]) {
                a[0] = 1;
                return a[0];
            }
        "#;
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca [1 x i32]
            %getelementptr_2 = getelementptr [1 x i32], ptr %alloca_1, i32 0, i32 0
            %getelementptr_3 = getelementptr i32, ptr %getelementptr_2, i32 0
            store i32 8, ptr %getelementptr_3
            %getelementptr_5 = getelementptr [1 x i32], ptr %alloca_1, i32 0, i32 0
            %call_6 = call i32 @f(i32* %getelementptr_5)
            %getelementptr_7 = getelementptr [1 x i32], ptr %alloca_1, i32 0, i32 0
            %call_8 = call void @putarray(i32 1, i32* %getelementptr_7)
            ret i32 0
            
            
            }
            define i32 @f(i32* a) {
            %entry:
            %alloca_11 = alloca i32*
            store i32* %a, ptr %alloca_11
            %load_13 = load i32*, ptr %alloca_11
            store i32 1, ptr %load_13
            %load_15 = load i32*, ptr %alloca_11
            %load_16 = load i32, ptr %load_15
            ret i32 %load_16
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca float
            store float 5.4, ptr %alloca_1
            %alloca_3 = alloca i32
            store i32 8, ptr %alloca_3
            %alloca_5 = alloca i32
            store i32 0, ptr %alloca_5
            br label %cond0
            
            %cond0:
            %load_12 = load float, ptr %alloca_1
            %fcmp_13 = fcmp une float %load_12, 0
            br i1 %fcmp_13, label %then1, label %alt2
            
            %then1:
            store i32 1, ptr %alloca_5
            br label %final3
            
            %alt2:
            br label %final3
            
            %final3:
            br label %cond4
            
            %cond4:
            %load_23 = load i32, ptr %alloca_3
            %icmp_24 = icmp ne i32 %load_23, 0
            br i1 %icmp_24, label %then5, label %alt6
            
            %then5:
            store i32 2, ptr %alloca_5
            br label %final7
            
            %alt6:
            br label %final7
            
            %final7:
            %load_29 = load i32, ptr %alloca_5
            ret i32 %load_29
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Float32, \"a\", Some(Float32(5.4)))), Decl(Var(Int32, \"b\", Some(Int32(8)))), Decl(Var(Int32, \"z\", Some(Int32(0)))), If(Var(\"a\"), Block([Expr(Some(Var(\"z\")), Int32(1))]), Block([])), If(Var(\"b\"), Block([Expr(Some(Var(\"z\")), Int32(2))]), Block([])), Return(Some(Var(\"z\")))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_unary() {
        let code = r#"
            int main() {
                int x = 1;
                return !+-x;
            }
        "#;
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            store i32 1, ptr %alloca_1
            %load_3 = load i32, ptr %alloca_1
            %Sub_4 = sub i32, 0, %load_3
            %icmp_5 = icmp ne i32 %Sub_4, 0
            %Xor_6 = xor i1, %icmp_5, true
            %zext_7 = zext i1 %Xor_6 to i32
            ret i32 %zext_7
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"x\", Some(Int32(1)))), Return(Some(Unary(Not, Unary(Pos, Unary(Neg, Var(\"x\"))))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i1
            %icmp_2 = icmp slt i32 1, 2
            store i1 %icmp_2, ptr %alloca_1
            %alloca_4 = alloca i1
            %itofp_5 = sitofp i32 1 to float
            %fcmp_6 = fcmp ult float %itofp_5, 1.1
            store i1 %fcmp_6, ptr %alloca_4
            %load_10 = load i1, ptr %alloca_1
            br i1 %load_10, label %alt0, label %final1
            
            %alt0:
            %load_12 = load i1, ptr %alloca_4
            br label %final1
            
            %final1:
            %phi_14 = phi i1 [false, %entry], [%load_12, %alt0]
            %zext_15 = zext i1 %phi_14 to i32
            ret i32 %zext_15
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Boolean, \"x\", Some(Binary(Lt, Int32(1), Int32(2))))), Decl(Var(Boolean, \"y\", Some(Binary(Lt, Int32(1), Float32(1.1))))), Return(Some(Binary(And, Var(\"x\"), Var(\"y\"))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
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
        let expected = r#"@x = dso_local constant i32 4
            @y = dso_local constant i32 16
            define i32 @main() {
            %entry:
            %load_1 = load i32, ptr @x
            %load_2 = load i32, ptr @y
            %Add_3 = add i32, %load_1, %load_2
            ret i32 %Add_3
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Const(Int32, \"x\", Some(Binary(Add, Int32(1), Int32(3)))), Const(Int32, \"y\", Some(Binary(Mul, Var(\"x\"), Var(\"x\")))), Func(Function(Int32, []), \"main\", Some(Block([Return(Some(Binary(Add, Var(\"x\"), Var(\"y\"))))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_library_function() {
        let code = r#"
            int main() {
                int x = getint();
                putint(x + 3);
                return 0;
            }
        "#;
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            %call_2 = call i32 @getint()
            store i32 %call_2, ptr %alloca_1
            %load_4 = load i32, ptr %alloca_1
            %Add_5 = add i32, %load_4, 3
            %call_6 = call void @putint(i32 %Add_5)
            ret i32 0
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        assert_eq!(format!("{:?}", program), "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"x\", Some(Call(Var(\"getint\"), [])))), Expr(None, Call(Var(\"putint\"), [Binary(Add, Var(\"x\"), Int32(3))])), Return(Some(Int32(0)))])))] }");
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_lazy_eval() {
        let code = r#"
            int main() {
                int x = getint();
                (x > 1) && f(x);
                return 0;
            }

            int f(int x) {
                putint(x);
                return x;
            }
        "#;
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            %call_2 = call i32 @getint()
            store i32 %call_2, ptr %alloca_1
            %load_6 = load i32, ptr %alloca_1
            %icmp_7 = icmp sgt i32 %load_6, 1
            br i1 %icmp_7, label %alt0, label %final1
            
            %alt0:
            %load_9 = load i32, ptr %alloca_1
            %call_10 = call i32 @f(i32 %load_9)
            %icmp_11 = icmp ne i32 %call_10, 0
            br label %final1
            
            %final1:
            %phi_13 = phi i1 [false, %entry], [%icmp_11, %alt0]
            ret i32 0
            
            
            }
            define i32 @f(i32 x) {
            %entry:
            %alloca_16 = alloca i32
            store i32 %x, ptr %alloca_16
            %load_18 = load i32, ptr %alloca_16
            %call_19 = call void @putint(i32 %load_18)
            %load_20 = load i32, ptr %alloca_16
            ret i32 %load_20
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_lazy_eval_with_if() {
        let code = r#"
            int main() {
                int x = getint();
                if (x > 1 && x < 3) {
                    putint(x);
                }
                return 0;
            }
        "#;
        let expected = r#"define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            %call_2 = call i32 @getint()
            store i32 %call_2, ptr %alloca_1
            br label %cond0
            
            %cond0:
            %load_11 = load i32, ptr %alloca_1
            %icmp_12 = icmp sgt i32 %load_11, 1
            br i1 %icmp_12, label %alt4, label %final5
            
            %alt4:
            %load_14 = load i32, ptr %alloca_1
            %icmp_15 = icmp slt i32 %load_14, 3
            br label %final5
            
            %final5:
            %phi_17 = phi i1 [false, %cond0], [%icmp_15, %alt4]
            br i1 %phi_17, label %then1, label %alt2
            
            %then1:
            %load_19 = load i32, ptr %alloca_1
            %call_20 = call void @putint(i32 %load_19)
            br label %final3
            
            %alt2:
            br label %final3
            
            %final3:
            ret i32 0
            
            
            }
            "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }

    #[test]
    fn test_putf() {
        let code = r#"
            int main() {
                int x = getint();
                putf("x = %d", x);
                return 0;
            }
        "#;
        let expected = r#"@format0 = dso_local constant [7 x i32] [i32 120, i32 32, i32 61, i32 32, i32 37, i32 100, i32 0]
            define i32 @main() {
            %entry:
            %alloca_1 = alloca i32
            %call_2 = call i32 @getint()
            store i32 %call_2, ptr %alloca_1
            %getelementptr_4 = getelementptr [7 x i32], ptr @format0, i32 0, i32 0
            %load_5 = load i32, ptr %alloca_1
            %call_6 = call void @putf(i32* %getelementptr_4, i32 %load_5)
            ret i32 0
            
            
            }
        "#
        .split('\n')
        .map(|x| x.trim())
        .collect::<Vec<&str>>()
        .join("\n");
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_eq!(llvm_ir, expected);
    }
}
