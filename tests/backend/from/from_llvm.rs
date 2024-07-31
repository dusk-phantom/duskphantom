#[cfg(test)]
#[cfg(feature = "clang_enabled")]
mod from_llvm_tests {

    use anyhow::Result;
    use compiler::{backend, clang_frontend};
    use insta::{assert_debug_snapshot, assert_snapshot};

    fn parse_to_backend(code: &str) -> Result<backend::Program> {
        let tmp_cfile = tempfile::Builder::new().suffix(".c").tempfile()?;
        std::fs::write(tmp_cfile.path(), code)?;
        let front: clang_frontend::Program = clang_frontend::Program::parse_file(tmp_cfile.path())?;
        let program: backend::Program = backend::from_llvm::gen_from_clang(&front)?;
        Ok(program)
    }
    fn parse_to_single_module(code: &str) -> Result<backend::Module> {
        let prog = parse_to_backend(code)?;
        Ok(prog.modules.into_iter().next().unwrap())
    }

    fn find_func<'a>(module: &'a backend::Module, name: &str) -> Option<&'a backend::Func> {
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

    #[test]
    #[cfg(feature = "gen_virtual_asm")]
    fn test_extra_params_func() {
        let code = indoc::indoc! {r#"
            int testParam16(int a0, int a1, int a2, int a3, int a4, int a5, int a6, int a7,
                int a8, int a9, int a10, int a11) {
                    return a0 + a1 + a2 + a3 + a4 + a5 + a6 + a7 + a8 + a9 + a10 + a11;
            }
        "#};
        let module = parse_to_single_module(code).unwrap();
        let func = find_func(&module, "testParam16").unwrap();
        assert_snapshot!(func.gen_asm(), @r###"
        .text
        .align	3
        .globl	testParam16
        .type	testParam16, @function
        testParam16:
        testParam16_.LBB_12:
        mv x32,a0
        mv x33,a1
        mv x34,a2
        mv x35,a3
        mv x36,a4
        mv x37,a5
        mv x38,a6
        mv x39,a7
        ld x40,0(s0)
        ld x41,8(s0)
        ld x42,16(s0)
        ld x43,24(s0)
        store x32,[72-80]
        store x33,[64-72]
        store x34,[88-96]
        store x35,[48-56]
        store x36,[24-32]
        store x37,[32-40]
        store x38,[56-64]
        store x39,[0-8]
        store x40,[40-48]
        store x41,[16-24]
        store x42,[8-16]
        store x43,[80-88]
        load x44,[72-80]
        load x45,[64-72]
        add x46,x44,x45
        load x47,[88-96]
        add x48,x46,x47
        load x49,[48-56]
        add x50,x48,x49
        load x51,[24-32]
        add x52,x50,x51
        load x53,[32-40]
        add x54,x52,x53
        load x55,[56-64]
        add x56,x54,x55
        load x57,[0-8]
        add x58,x56,x57
        load x59,[40-48]
        add x60,x58,x59
        load x61,[16-24]
        add x62,x60,x61
        load x63,[8-16]
        add x64,x62,x63
        load x65,[80-88]
        add x66,x64,x65
        mv a0,x66
        ret
        .size	testParam16, .-testParam16
        "###);
    }
}
