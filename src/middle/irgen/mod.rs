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
    use insta::assert_snapshot;

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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 1, ptr %alloca_5
        %alloca_7 = alloca i32
        store i32 2, ptr %alloca_7
        %alloca_9 = alloca i32
        %load_10 = load i32, ptr %alloca_5
        %load_11 = load i32, ptr %alloca_7
        %Add_12 = add i32, %load_10, %load_11
        store i32 %Add_12, ptr %alloca_9
        %load_14 = load i32, ptr %alloca_9
        store i32 %load_14, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 1, ptr %alloca_5
        %alloca_7 = alloca i32
        store i32 2, ptr %alloca_7
        br label %cond0

        %cond0:
        %load_14 = load i32, ptr %alloca_5
        %load_15 = load i32, ptr %alloca_7
        %icmp_16 = icmp slt i32 %load_14, %load_15
        br i1 %icmp_16, label %then1, label %alt2

        %then1:
        store i32 3, ptr %alloca_5
        br label %final3

        %alt2:
        store i32 4, ptr %alloca_5
        br label %final3

        %final3:
        %load_22 = load i32, ptr %alloca_5
        store i32 %load_22, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 0, ptr %alloca_5
        br label %cond0

        %cond0:
        %load_15 = load i32, ptr %alloca_5
        %icmp_16 = icmp slt i32 %load_15, 10
        br i1 %icmp_16, label %body1, label %final2

        %body1:
        %load_11 = load i32, ptr %alloca_5
        %Add_12 = add i32, %load_11, 1
        store i32 %Add_12, ptr %alloca_5
        br label %cond0

        %final2:
        %load_18 = load i32, ptr %alloca_5
        store i32 %load_18, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 0, ptr %alloca_5
        br label %body0

        %body0:
        %load_11 = load i32, ptr %alloca_5
        %Add_12 = add i32, %load_11, 1
        store i32 %Add_12, ptr %alloca_5
        br label %cond1

        %cond1:
        %load_15 = load i32, ptr %alloca_5
        %icmp_16 = icmp slt i32 %load_15, 10
        br i1 %icmp_16, label %body0, label %final2

        %final2:
        %load_18 = load i32, ptr %alloca_5
        store i32 %load_18, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 0, ptr %alloca_5
        br label %cond0

        %cond0:
        %load_15 = load i32, ptr %alloca_5
        %icmp_16 = icmp slt i32 %load_15, 10
        br i1 %icmp_16, label %body1, label %final2

        %body1:
        %load_11 = load i32, ptr %alloca_5
        %Add_12 = add i32, %load_11, 1
        store i32 %Add_12, ptr %alloca_5
        br label %final2

        %final2:
        %load_18 = load i32, ptr %alloca_5
        store i32 %load_18, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 0, ptr %alloca_5
        br label %cond0

        %cond0:
        %load_15 = load i32, ptr %alloca_5
        %icmp_16 = icmp slt i32 %load_15, 10
        br i1 %icmp_16, label %body1, label %final2

        %body1:
        %load_11 = load i32, ptr %alloca_5
        %Add_12 = add i32, %load_11, 1
        store i32 %Add_12, ptr %alloca_5
        br label %cond0

        %final2:
        %load_18 = load i32, ptr %alloca_5
        store i32 %load_18, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 0, ptr %alloca_5
        br label %cond0

        %cond0:
        %load_25 = load i32, ptr %alloca_5
        %icmp_26 = icmp slt i32 %load_25, 10
        br i1 %icmp_26, label %body1, label %final2

        %body1:
        %load_11 = load i32, ptr %alloca_5
        %Add_12 = add i32, %load_11, 1
        store i32 %Add_12, ptr %alloca_5
        br label %cond3

        %final2:
        %load_28 = load i32, ptr %alloca_5
        store i32 %load_28, ptr %alloca_2
        br label %exit

        %cond3:
        %load_19 = load i32, ptr %alloca_5
        %icmp_20 = icmp eq i32 %load_19, 5
        br i1 %icmp_20, label %then4, label %alt5

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3

        %then4:
        br label %final2

        %alt5:
        br label %cond0


        }
        "###);
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
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 0, ptr %alloca_5
        br label %cond0

        %cond0:
        %load_25 = load i32, ptr %alloca_5
        %icmp_26 = icmp slt i32 %load_25, 10
        br i1 %icmp_26, label %body1, label %final2

        %body1:
        %load_11 = load i32, ptr %alloca_5
        %Add_12 = add i32, %load_11, 1
        store i32 %Add_12, ptr %alloca_5
        br label %cond3

        %final2:
        %load_28 = load i32, ptr %alloca_5
        store i32 %load_28, ptr %alloca_2
        br label %exit

        %cond3:
        %load_19 = load i32, ptr %alloca_5
        %icmp_20 = icmp eq i32 %load_19, 5
        br i1 %icmp_20, label %then4, label %alt5

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3

        %then4:
        br label %final2

        %alt5:
        br label %final6

        %final6:
        br label %cond0


        }
        "###);
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
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        br label %cond0

        %cond0:
        %icmp_10 = icmp ne i32 1, 0
        br i1 %icmp_10, label %body1, label %final2

        %body1:
        br label %final2

        %final2:
        store i32 0, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        @x = dso_local global i32 4
        @y = dso_local global i32 8
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %load_5 = load i32, ptr @x
        %load_6 = load i32, ptr @y
        %Add_7 = add i32, %load_5, %load_6
        store i32 %Add_7, ptr @x
        %load_9 = load i32, ptr @x
        store i32 %load_9, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 1, ptr %alloca_5
        %alloca_7 = alloca float
        store float 2, ptr %alloca_7
        %alloca_9 = alloca float
        %load_10 = load i32, ptr %alloca_5
        %itofp_11 = sitofp i32 %load_10 to float
        %load_12 = load float, ptr %alloca_7
        %FAdd_13 = fadd float, %itofp_11, %load_12
        store float %FAdd_13, ptr %alloca_9
        %load_15 = load float, ptr %alloca_9
        %fptoi_16 = fptosi float %load_15 to i32
        store i32 %fptoi_16, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
    }

    #[test]
    fn test_zext() {
        let code = r#"
            int main() {
                return (3 > 1) + (4 > 2);
            }
        "#;
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %icmp_5 = icmp sgt i32 3, 1
        %icmp_6 = icmp sgt i32 4, 2
        %zext_7 = zext i1 %icmp_5 to i32
        %zext_8 = zext i1 %icmp_6 to i32
        %Add_9 = add i32, %zext_7, %zext_8
        store i32 %Add_9, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
    }

    #[test]
    fn test_param() {
        let code = r#"
            int main(int arg) {
                return arg;
            }
        "#;
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main(i32 arg) {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 %arg, ptr %alloca_5
        %load_7 = load i32, ptr %alloca_5
        store i32 %load_7, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %fptoi_5 = fptosi float 1.7 to i32
        %call_6 = call i32 @f(i32 %fptoi_5)
        store i32 %call_6, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        define i32 @f(i32 x) {
        %entry:
        %alloca_11 = alloca i32
        %alloca_14 = alloca i32
        store i32 %x, ptr %alloca_14
        %load_16 = load i32, ptr %alloca_14
        %Add_17 = add i32, %load_16, 1
        store i32 %Add_17, ptr %alloca_11
        br label %exit

        %exit:
        %load_12 = load i32, ptr %alloca_11
        ret i32 %load_12


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %call_5 = call i32 @f(i32 1)
        %call_6 = call i32 @f(i32 %call_5)
        store i32 %call_6, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        define i32 @f(i32 x) {
        %entry:
        %alloca_11 = alloca i32
        %alloca_14 = alloca i32
        store i32 %x, ptr %alloca_14
        %load_16 = load i32, ptr %alloca_14
        %Add_17 = add i32, %load_16, 1
        store i32 %Add_17, ptr %alloca_11
        br label %exit

        %exit:
        %load_12 = load i32, ptr %alloca_11
        ret i32 %load_12


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        @PI = dso_local constant float 3.1415925
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %load_5 = load float, ptr @PI
        %fptoi_6 = fptosi float %load_5 to i32
        store i32 %fptoi_6, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
    }

    #[test]
    fn test_constant_array() {
        let code = r#"
            const float A[3][2][2] = {{1}, 1, 4, 5, 1, {4}};

            int main() {
                return A[0][0][0];
            }
        "#;
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        @A = dso_local constant [3 x [2 x [2 x float]]] [[2 x [2 x float]] [[2 x float] [float 1, float 0], [2 x float] [float 0, float 0]], [2 x [2 x float]] [[2 x float] [float 1, float 4], [2 x float] [float 5, float 1]], [2 x [2 x float]] [[2 x float] [float 4, float 0], [2 x float] [float 0, float 0]]]
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %getelementptr_5 = getelementptr [3 x [2 x [2 x float]]], ptr @A, i32 0, i32 0
        %getelementptr_6 = getelementptr [2 x [2 x float]], ptr %getelementptr_5, i32 0, i32 0
        %getelementptr_7 = getelementptr [2 x float], ptr %getelementptr_6, i32 0, i32 0
        %load_8 = load float, ptr %getelementptr_7
        %fptoi_9 = fptosi float %load_8 to i32
        store i32 %fptoi_9, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
    }

    #[test]
    fn test_variable_array() {
        let code = r#"
            int main() {
                float A[2][2][2] = {1, 1, 4, 5, {{1}, 4}};
                return A[1][1][1];
            }
        "#;
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca [2 x [2 x [2 x float]]]
        %getelementptr_6 = getelementptr [2 x [2 x [2 x float]]], ptr %alloca_5, i32 0, i32 0
        %getelementptr_7 = getelementptr [2 x [2 x float]], ptr %getelementptr_6, i32 0
        %getelementptr_8 = getelementptr [2 x [2 x float]], ptr %getelementptr_7, i32 0, i32 0
        %getelementptr_9 = getelementptr [2 x float], ptr %getelementptr_8, i32 0
        %getelementptr_10 = getelementptr [2 x float], ptr %getelementptr_9, i32 0, i32 0
        %getelementptr_11 = getelementptr float, ptr %getelementptr_10, i32 0
        %itofp_12 = sitofp i32 1 to float
        store float %itofp_12, ptr %getelementptr_11
        %getelementptr_14 = getelementptr float, ptr %getelementptr_10, i32 1
        %itofp_15 = sitofp i32 1 to float
        store float %itofp_15, ptr %getelementptr_14
        %getelementptr_17 = getelementptr [2 x float], ptr %getelementptr_8, i32 1
        %getelementptr_18 = getelementptr [2 x float], ptr %getelementptr_17, i32 0, i32 0
        %getelementptr_19 = getelementptr float, ptr %getelementptr_18, i32 0
        %itofp_20 = sitofp i32 4 to float
        store float %itofp_20, ptr %getelementptr_19
        %getelementptr_22 = getelementptr float, ptr %getelementptr_18, i32 1
        %itofp_23 = sitofp i32 5 to float
        store float %itofp_23, ptr %getelementptr_22
        %getelementptr_25 = getelementptr [2 x [2 x float]], ptr %getelementptr_6, i32 1
        %getelementptr_26 = getelementptr [2 x [2 x float]], ptr %getelementptr_25, i32 0, i32 0
        %getelementptr_27 = getelementptr [2 x float], ptr %getelementptr_26, i32 0
        %getelementptr_28 = getelementptr [2 x float], ptr %getelementptr_27, i32 0, i32 0
        %getelementptr_29 = getelementptr float, ptr %getelementptr_28, i32 0
        %itofp_30 = sitofp i32 1 to float
        store float %itofp_30, ptr %getelementptr_29
        %getelementptr_32 = getelementptr [2 x float], ptr %getelementptr_26, i32 1
        %getelementptr_33 = getelementptr [2 x float], ptr %getelementptr_32, i32 0, i32 0
        %getelementptr_34 = getelementptr float, ptr %getelementptr_33, i32 0
        %itofp_35 = sitofp i32 4 to float
        store float %itofp_35, ptr %getelementptr_34
        %getelementptr_37 = getelementptr [2 x [2 x [2 x float]]], ptr %alloca_5, i32 0, i32 1
        %getelementptr_38 = getelementptr [2 x [2 x float]], ptr %getelementptr_37, i32 0, i32 1
        %getelementptr_39 = getelementptr [2 x float], ptr %getelementptr_38, i32 0, i32 1
        %load_40 = load float, ptr %getelementptr_39
        %fptoi_41 = fptosi float %load_40 to i32
        store i32 %fptoi_41, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca [1 x i32]
        %getelementptr_6 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %getelementptr_7 = getelementptr i32, ptr %getelementptr_6, i32 0
        store i32 0, ptr %getelementptr_7
        %getelementptr_9 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %load_10 = load i32, ptr %getelementptr_9
        %getelementptr_11 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 %load_10
        store i32 1, ptr %getelementptr_11
        %getelementptr_13 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %load_14 = load i32, ptr %getelementptr_13
        store i32 %load_14, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca [1 x i32]
        %getelementptr_6 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %getelementptr_7 = getelementptr i32, ptr %getelementptr_6, i32 0
        store i32 8, ptr %getelementptr_7
        %getelementptr_9 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %call_10 = call i32 @f(i32* %getelementptr_9)
        %getelementptr_11 = getelementptr [1 x i32], ptr %alloca_5, i32 0, i32 0
        %call_12 = call void @putarray(i32 1, i32* %getelementptr_11)
        store i32 0, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        define i32 @f(i32* a) {
        %entry:
        %alloca_17 = alloca i32
        %alloca_20 = alloca i32*
        store i32* %a, ptr %alloca_20
        %load_22 = load i32*, ptr %alloca_20
        store i32 1, ptr %load_22
        %load_24 = load i32*, ptr %alloca_20
        %load_25 = load i32, ptr %load_24
        store i32 %load_25, ptr %alloca_17
        br label %exit

        %exit:
        %load_18 = load i32, ptr %alloca_17
        ret i32 %load_18


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca float
        store float 5.4, ptr %alloca_5
        %alloca_7 = alloca i32
        store i32 8, ptr %alloca_7
        %alloca_9 = alloca i32
        store i32 0, ptr %alloca_9
        br label %cond0

        %cond0:
        %load_16 = load float, ptr %alloca_5
        %fcmp_17 = fcmp une float %load_16, 0
        br i1 %fcmp_17, label %then1, label %alt2

        %then1:
        store i32 1, ptr %alloca_9
        br label %final3

        %alt2:
        br label %final3

        %final3:
        br label %cond4

        %cond4:
        %load_27 = load i32, ptr %alloca_7
        %icmp_28 = icmp ne i32 %load_27, 0
        br i1 %icmp_28, label %then5, label %alt6

        %then5:
        store i32 2, ptr %alloca_9
        br label %final7

        %alt6:
        br label %final7

        %final7:
        %load_33 = load i32, ptr %alloca_9
        store i32 %load_33, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        store i32 1, ptr %alloca_5
        %load_7 = load i32, ptr %alloca_5
        %Sub_8 = sub i32, 0, %load_7
        %icmp_9 = icmp ne i32 %Sub_8, 0
        %Xor_10 = xor i1, %icmp_9, true
        %zext_11 = zext i1 %Xor_10 to i32
        store i32 %zext_11, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i1
        %icmp_6 = icmp slt i32 1, 2
        store i1 %icmp_6, ptr %alloca_5
        %alloca_8 = alloca i1
        %itofp_9 = sitofp i32 1 to float
        %fcmp_10 = fcmp ult float %itofp_9, 1.1
        store i1 %fcmp_10, ptr %alloca_8
        %load_14 = load i1, ptr %alloca_5
        br i1 %load_14, label %alt0, label %final1

        %alt0:
        %load_16 = load i1, ptr %alloca_8
        br label %final1

        %final1:
        %phi_18 = phi i1 [false, %entry], [%load_16, %alt0]
        %zext_19 = zext i1 %phi_18 to i32
        store i32 %zext_19, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        @x = dso_local constant i32 4
        @y = dso_local constant i32 16
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %load_5 = load i32, ptr @x
        %load_6 = load i32, ptr @y
        %Add_7 = add i32, %load_5, %load_6
        store i32 %Add_7, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        %call_6 = call i32 @getint()
        store i32 %call_6, ptr %alloca_5
        %load_8 = load i32, ptr %alloca_5
        %Add_9 = add i32, %load_8, 3
        %call_10 = call void @putint(i32 %Add_9)
        store i32 0, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        %call_6 = call i32 @getint()
        store i32 %call_6, ptr %alloca_5
        %load_10 = load i32, ptr %alloca_5
        %icmp_11 = icmp sgt i32 %load_10, 1
        br i1 %icmp_11, label %alt0, label %final1

        %alt0:
        %load_13 = load i32, ptr %alloca_5
        %call_14 = call i32 @f(i32 %load_13)
        %icmp_15 = icmp ne i32 %call_14, 0
        br label %final1

        %final1:
        %phi_17 = phi i1 [false, %entry], [%icmp_15, %alt0]
        store i32 0, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        define i32 @f(i32 x) {
        %entry:
        %alloca_22 = alloca i32
        %alloca_25 = alloca i32
        store i32 %x, ptr %alloca_25
        %load_27 = load i32, ptr %alloca_25
        %call_28 = call void @putint(i32 %load_27)
        %load_29 = load i32, ptr %alloca_25
        store i32 %load_29, ptr %alloca_22
        br label %exit

        %exit:
        %load_23 = load i32, ptr %alloca_22
        ret i32 %load_23


        }
        "###);
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
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        %call_6 = call i32 @getint()
        store i32 %call_6, ptr %alloca_5
        br label %cond0

        %cond0:
        %load_15 = load i32, ptr %alloca_5
        %icmp_16 = icmp sgt i32 %load_15, 1
        br i1 %icmp_16, label %alt4, label %final5

        %alt4:
        %load_18 = load i32, ptr %alloca_5
        %icmp_19 = icmp slt i32 %load_18, 3
        br label %final5

        %final5:
        %phi_21 = phi i1 [false, %cond0], [%icmp_19, %alt4]
        br i1 %phi_21, label %then1, label %alt2

        %then1:
        %load_23 = load i32, ptr %alloca_5
        %call_24 = call void @putint(i32 %load_23)
        br label %final3

        %alt2:
        br label %final3

        %final3:
        store i32 0, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
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
        let program = parse(code).unwrap();
        let result = gen(&program).unwrap();
        let llvm_ir = result.module.gen_llvm_ir();
        assert_snapshot!(llvm_ir, @r###"
        @format0 = dso_local constant [7 x i32] [i32 120, i32 32, i32 61, i32 32, i32 37, i32 100, i32 0]
        define i32 @main() {
        %entry:
        %alloca_2 = alloca i32
        %alloca_5 = alloca i32
        %call_6 = call i32 @getint()
        store i32 %call_6, ptr %alloca_5
        %getelementptr_8 = getelementptr [7 x i32], ptr @format0, i32 0, i32 0
        %load_9 = load i32, ptr %alloca_5
        %call_10 = call void @putf(i32* %getelementptr_8, i32 %load_9)
        store i32 0, ptr %alloca_2
        br label %exit

        %exit:
        %load_3 = load i32, ptr %alloca_2
        ret i32 %load_3


        }
        "###);
    }
}
