// Unit tests
#[cfg(test)]
pub mod tests_parse {
    use insta::assert_debug_snapshot;

    use compiler::{errors::FrontendError, frontend::*};

    #[test]
    fn test_simple_main() {
        let code = r#"
        int main() {
            return 0;
        }
        "#;
        match parse(code) {
            Ok(result) => {
                assert_debug_snapshot!(
                    result,
                    @r###"
                Program {
                    module: [
                        Func(
                            Function(
                                Int32,
                                [],
                            ),
                            "main",
                            Some(
                                Block(
                                    [
                                        Return(
                                            Some(
                                                Int32(
                                                    0,
                                                ),
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                        ),
                    ],
                }
                "###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }

    #[test]
    fn test_number() {
        let code = r#"
        int main() {
            int a0 = 3;
            int a1 = 0xFACE;
            int a2 = 0Xbad4;
            int a3 = 0777;
            float b0 = 3.7;
            float b1 = 2.;
            float b2 = .9;
            float c0 = 2.3e+4;
            float c1 = 0.5e-9;
            float c2 = 1e3;
            float c3 = 2.e4;
            float c4 = .5e1;
            float d0 = 0x1.ep+3;
            float d1 = 0x8.Ap-3;
            float d2 = 0xFp3;
            float d3 = 0Xfp3;
            float d4 = 0xc.p3;
            float d5 = 0x.Dp3;
        }
        "#;
        match parse(code) {
            Ok(result) => {
                assert_debug_snapshot!(
                    result,
                    @r###"
                Program {
                    module: [
                        Func(
                            Function(
                                Int32,
                                [],
                            ),
                            "main",
                            Some(
                                Block(
                                    [
                                        Decl(
                                            Var(
                                                Int32,
                                                "a0",
                                                Some(
                                                    Int32(
                                                        3,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Int32,
                                                "a1",
                                                Some(
                                                    Int32(
                                                        64206,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Int32,
                                                "a2",
                                                Some(
                                                    Int32(
                                                        47828,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Int32,
                                                "a3",
                                                Some(
                                                    Int32(
                                                        511,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "b0",
                                                Some(
                                                    Float32(
                                                        3.7,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "b1",
                                                Some(
                                                    Float32(
                                                        2.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "b2",
                                                Some(
                                                    Float32(
                                                        0.9,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "c0",
                                                Some(
                                                    Float32(
                                                        23000.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "c1",
                                                Some(
                                                    Float32(
                                                        5e-10,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "c2",
                                                Some(
                                                    Float32(
                                                        1000.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "c3",
                                                Some(
                                                    Float32(
                                                        20000.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "c4",
                                                Some(
                                                    Float32(
                                                        5.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "d0",
                                                Some(
                                                    Float32(
                                                        15.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "d1",
                                                Some(
                                                    Float32(
                                                        1.078125,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "d2",
                                                Some(
                                                    Float32(
                                                        120.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "d3",
                                                Some(
                                                    Float32(
                                                        120.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "d4",
                                                Some(
                                                    Float32(
                                                        96.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float32,
                                                "d5",
                                                Some(
                                                    Float32(
                                                        6.5,
                                                    ),
                                                ),
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                        ),
                    ],
                }
                "###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }

    #[test]
    fn test_assign() {
        let code = "int n = 3;";
        match parse(code) {
            Ok(result) => {
                assert_debug_snapshot!(
                    result,
                    @r###"
                Program {
                    module: [
                        Var(
                            Int32,
                            "n",
                            Some(
                                Int32(
                                    3,
                                ),
                            ),
                        ),
                    ],
                }
                "###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }

    #[test]
    fn test_assign_in_main() {
        let code = r#"
        int main() {
            int n = 3;
            return n;
        }
        "#;
        match parse(code) {
            Ok(result) => {
                assert_debug_snapshot!(
                    result,
                    @r###"
                Program {
                    module: [
                        Func(
                            Function(
                                Int32,
                                [],
                            ),
                            "main",
                            Some(
                                Block(
                                    [
                                        Decl(
                                            Var(
                                                Int32,
                                                "n",
                                                Some(
                                                    Int32(
                                                        3,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Return(
                                            Some(
                                                Var(
                                                    "n",
                                                ),
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                        ),
                    ],
                }
                "###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }

    #[test]
    fn test_comment() {
        let code = r#"
        //test comment
        int main(){
            int a;
            a = 5;
            //int b = 4;
            //a = b + a;
            /*/*
                b = 1;
                // b = 2
            */
            return a;
        }
        "#;
        match parse(code) {
            Ok(result) => {
                assert_debug_snapshot!(
                    result,
                    @r###"
                Program {
                    module: [
                        Func(
                            Function(
                                Int32,
                                [],
                            ),
                            "main",
                            Some(
                                Block(
                                    [
                                        Decl(
                                            Var(
                                                Int32,
                                                "a",
                                                None,
                                            ),
                                        ),
                                        Expr(
                                            Some(
                                                Var(
                                                    "a",
                                                ),
                                            ),
                                            Int32(
                                                5,
                                            ),
                                        ),
                                        Return(
                                            Some(
                                                Var(
                                                    "a",
                                                ),
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                        ),
                    ],
                }
                "###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }

    #[test]
    fn test_macro() {
        let code = r#"
            #include "sylib.h"
            #define MAX 100
        "#;
        match parse(code) {
            Ok(result) => {
                assert_debug_snapshot!(
                    result,
                    @r###"
                Program {
                    module: [
                        Stack(
                            [],
                        ),
                        Const(
                            Int32,
                            "MAX",
                            Some(
                                Int32(
                                    100,
                                ),
                            ),
                        ),
                    ],
                }
                "###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }
}
