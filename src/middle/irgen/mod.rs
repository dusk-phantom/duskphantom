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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
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
        assert_snapshot!(llvm_ir);
    }
}
