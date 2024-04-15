use std::collections::HashMap;
use program_kit::ProgramKit;
use crate::{
    errors::MiddelError,
    frontend,
    middle,
};

mod function_kit;
mod program_kit;
mod operand;
mod value_type;
mod value;
mod util;
mod constant;

/// Generate middle IR from a frontend AST
pub fn gen(program: &frontend::Program) -> Result<middle::Program, MiddelError> {
    let mut result = middle::Program::new();
    ProgramKit {
        program: &mut result,
        env: HashMap::new(),
        fun_env: HashMap::new(),
        ctx: HashMap::new(),
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
        assert_eq!(
            llvm_ir,
            "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%alloca_3 = alloca i32\nstore i32 2, ptr %alloca_3\n%alloca_5 = alloca i32\n%load_6 = load i32, ptr %alloca_1\n%load_7 = load i32, ptr %alloca_3\n%Add_8 = add i32, %load_6, %load_7\nstore i32 %Add_8, ptr %alloca_5\n%load_10 = load i32, ptr %alloca_5\nret %load_10\n\n\n}\n"
        );
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
        assert_eq!(
            llvm_ir,
            "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%alloca_3 = alloca i32\nstore i32 2, ptr %alloca_3\nbr label %cond\n\n%cond:\n%load_10 = load i32, ptr %alloca_1\n%load_11 = load i32, ptr %alloca_3\n%icmp_12 = icmp slt i32 %load_10, %load_11\nbr i1 %icmp_12, label %then, label %alt\n\n%then:\nstore i32 3, ptr %alloca_1\nbr label %final\n\n%alt:\nstore i32 4, ptr %alloca_1\nbr label %final\n\n%final:\n%load_18 = load i32, ptr %alloca_1\nret %load_18\n\n\n}\n"
        );
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
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %cond\n\n%cond:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body, label %final\n\n%body:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %cond\n\n%final:\n%load_14 = load i32, ptr %alloca_1\nret %load_14\n\n\n}\n");
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
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %body\n\n%body:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %cond\n\n%cond:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body, label %final\n\n%final:\n%load_14 = load i32, ptr %alloca_1\nret %load_14\n\n\n}\n");
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
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 0, ptr %alloca_1\nbr label %cond\n\n%cond:\n%load_7 = load i32, ptr %alloca_1\n%icmp_8 = icmp slt i32 %load_7, 10\nbr i1 %icmp_8, label %body, label %final\n\n%body:\n%load_10 = load i32, ptr %alloca_1\n%Add_11 = add i32, %load_10, 1\nstore i32 %Add_11, ptr %alloca_1\nbr label %final\nbr label %final\n\n%final:\n%load_15 = load i32, ptr %alloca_1\nret %load_15\n\n\n}\n");
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
        assert_eq!(llvm_ir, "define i32 @main() {\n%entry:\n%alloca_1 = alloca i32\nstore i32 1, ptr %alloca_1\n%alloca_3 = alloca float\nstore float 2, ptr %alloca_3\n%alloca_5 = alloca float\n%load_6 = load i32, ptr %alloca_1\n%load_7 = load float, ptr %alloca_3\n%itofp_8 = sitofp i32 %load_6 to float\n%FAdd_9 = fadd float, %itofp_8, %load_7\nstore float %FAdd_9, ptr %alloca_5\n%load_11 = load float, ptr %alloca_5\n%fptoi_12 = fptosi float %load_11 to i32\nret %fptoi_12\n\n\n}\n");
    }

    // #[test]
    // fn test_template() {
    //     let code = r#"
    //         int main() {
    //             int a = 0;
    //             while (a < 10) {
    //                 a = a + 1;
    //             }
    //             return a;
    //         }
    //     "#;
    //     let program = parse(code).unwrap();
    //     assert_eq!(format!("{:?}", program), "");
    //     let result = gen(&program).unwrap();
    //     let llvm_ir = result.module.gen_llvm_ir();
    //     assert_eq!(llvm_ir, "");
    // }
}
