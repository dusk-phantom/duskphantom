#[cfg(test)]
#[cfg(feature = "clang_enabled")]

mod from_llvm_tests {

    use anyhow::Result;
    use compiler::backend::irs::*;
    use compiler::clang_frontend;
    use insta::assert_debug_snapshot;

    fn parse_to_backend(code: &str) -> Result<Program> {
        let tmp_cfile = tempfile::Builder::new().suffix(".c").tempfile()?;
        std::fs::write(tmp_cfile.path(), code)?;
        let front: clang_frontend::Program = clang_frontend::Program::parse_file(tmp_cfile.path())?;
        let program: Program = from_llvm::gen_from_clang(&front)?;
        Ok(program)
    }

    #[allow(unused)]
    fn parse_to_single_module(code: &str) -> Result<Module> {
        let prog = parse_to_backend(code)?;
        Ok(prog.modules.into_iter().next().unwrap())
    }

    #[allow(unused)]
    fn find_func<'a>(module: &'a Module, name: &str) -> Option<&'a Func> {
        module.funcs.iter().find(|f| f.name() == name)
    }

    #[test]
    fn test_global_array() {
        let code = r#"
            int arr[200];
            int arr2[2][3][4][5];
            float arr3[100];
            float arr4[20][30];
        "#;
        let program = parse_to_backend(code).unwrap();
        let m = &program.modules[0];
        let vars = &m.global;
        assert_debug_snapshot!(vars, @r###"
        [
            IntArr(
                ArrVar {
                    name: "arr",
                    capacity: 200,
                    init: [],
                    is_const: false,
                },
            ),
            IntArr(
                ArrVar {
                    name: "arr2",
                    capacity: 120,
                    init: [],
                    is_const: false,
                },
            ),
            FloatArr(
                ArrVar {
                    name: "arr3",
                    capacity: 100,
                    init: [],
                    is_const: false,
                },
            ),
            FloatArr(
                ArrVar {
                    name: "arr4",
                    capacity: 600,
                    init: [],
                    is_const: false,
                },
            ),
        ]
        "###);
    }
    #[test]
    fn test_global_array_init() {
        let code = r#"
            int arr2[200] = {1, 2, 3};
            int arr3[33][44]={{1},{0,3}};
            float arr4[20][30] = {{1.0,0.0},{},{ 0.0, 3.0}};
            float arr5[150]={1.0, 0.0, 3.0};
        "#;
        let program = parse_to_backend(code).unwrap();
        let m = &program.modules[0];
        let vars = &m.global;
        assert_debug_snapshot!(vars, @r###"
        [
            IntArr(
                ArrVar {
                    name: "arr2",
                    capacity: 200,
                    init: [
                        (
                            0,
                            1,
                        ),
                        (
                            1,
                            2,
                        ),
                        (
                            2,
                            3,
                        ),
                    ],
                    is_const: false,
                },
            ),
            IntArr(
                ArrVar {
                    name: "arr3",
                    capacity: 1452,
                    init: [
                        (
                            0,
                            1,
                        ),
                        (
                            45,
                            3,
                        ),
                    ],
                    is_const: false,
                },
            ),
            FloatArr(
                ArrVar {
                    name: "arr4",
                    capacity: 600,
                    init: [
                        (
                            0,
                            1.0,
                        ),
                        (
                            61,
                            3.0,
                        ),
                    ],
                    is_const: false,
                },
            ),
            FloatArr(
                ArrVar {
                    name: "arr5",
                    capacity: 150,
                    init: [
                        (
                            0,
                            1.0,
                        ),
                        (
                            2,
                            3.0,
                        ),
                    ],
                    is_const: false,
                },
            ),
        ]
        "###);
    }
}
