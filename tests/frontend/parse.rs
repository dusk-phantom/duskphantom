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
                                Int,
                                [],
                            ),
                            "main",
                            Some(
                                Block(
                                    [
                                        Return(
                                            Some(
                                                Int(
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
                                Int,
                                [],
                            ),
                            "main",
                            Some(
                                Block(
                                    [
                                        Decl(
                                            Var(
                                                Int,
                                                "a0",
                                                Some(
                                                    Int(
                                                        3,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Int,
                                                "a1",
                                                Some(
                                                    Int(
                                                        64206,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Int,
                                                "a2",
                                                Some(
                                                    Int(
                                                        47828,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Int,
                                                "a3",
                                                Some(
                                                    Int(
                                                        511,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "b0",
                                                Some(
                                                    Float(
                                                        3.7,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "b1",
                                                Some(
                                                    Float(
                                                        2.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "b2",
                                                Some(
                                                    Float(
                                                        0.9,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "c0",
                                                Some(
                                                    Float(
                                                        23000.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "c1",
                                                Some(
                                                    Float(
                                                        5e-10,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "c2",
                                                Some(
                                                    Float(
                                                        1000.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "c3",
                                                Some(
                                                    Float(
                                                        20000.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "c4",
                                                Some(
                                                    Float(
                                                        5.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "d0",
                                                Some(
                                                    Float(
                                                        15.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "d1",
                                                Some(
                                                    Float(
                                                        1.078125,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "d2",
                                                Some(
                                                    Float(
                                                        120.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "d3",
                                                Some(
                                                    Float(
                                                        120.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "d4",
                                                Some(
                                                    Float(
                                                        96.0,
                                                    ),
                                                ),
                                            ),
                                        ),
                                        Decl(
                                            Var(
                                                Float,
                                                "d5",
                                                Some(
                                                    Float(
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
                            Int,
                            "n",
                            Some(
                                Int(
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
                                Int,
                                [],
                            ),
                            "main",
                            Some(
                                Block(
                                    [
                                        Decl(
                                            Var(
                                                Int,
                                                "n",
                                                Some(
                                                    Int(
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
                                Int,
                                [],
                            ),
                            "main",
                            Some(
                                Block(
                                    [
                                        Decl(
                                            Var(
                                                Int,
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
                                            Int(
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
                            Int,
                            "MAX",
                            Some(
                                Int(
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

    #[test]
    fn test_trailing_comment() {
        let code = r#"
        /*/skipher/*/
        //int main(){
        int main(){
            ////return 0;}/*
            /*}
            //}return 1;*/
            //}return 2;*//*
            return 3;
            //*/
        }
        //"#;
        match parse(code) {
            Ok(result) => {
                assert_debug_snapshot!(
                    result,
                    @r###"
                Program {
                    module: [
                        Func(
                            Function(
                                Int,
                                [],
                            ),
                            "main",
                            Some(
                                Block(
                                    [
                                        Return(
                                            Some(
                                                Int(
                                                    3,
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
}
