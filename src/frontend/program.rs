use super::*;

/// The full program.
/// A executable program is a set of modules with an entry module.
/// For now, only one module is supported, so the only module is entry.
#[derive(Clone, PartialEq, Debug)]
pub struct Program {
    /// The module of the program.
    /// Currently only one module is supported.
    pub module: Module,
}

impl Program {
    pub fn new(decls: Vec<Decl>) -> Self {
        Self { module: decls }
    }
}

/// A module is a single file.
/// Only declaration can appear at top level.
pub type Module = Vec<Decl>;

pub fn parse(src: &str) -> Result<Program, FrontendError> {
    preceded(blank, repeat(0.., decl))
        .map(Program::new)
        .parse(src)
        .map_err(|err| FrontendError::ParseError(err.to_string()))
}

#[allow(unused)]
pub fn optimize(program: &mut Program) {}

// Unit tests
#[cfg(test)]
pub mod tests_program {
    use insta::assert_snapshot;

    use super::*;

    #[test]
    fn test_minimal() {
        let code = "int main();";
        match parse(code) {
            Ok(result) => {
                assert_snapshot!(
                    format!("{:?}", result),
                    @r###"Program { module: [Func(Function(Int32, []), "main", None)] }"###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }

    #[test]
    fn test_simple_main() {
        let code = r#"
        int main() {
            return 0;
        }
        "#;
        match parse(code) {
            Ok(result) => {
                assert_snapshot!(
                    format!("{:?}", result),
                    @r###"Program { module: [Func(Function(Int32, []), "main", Some(Block([Return(Some(Int32(0)))])))] }"###
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
                assert_snapshot!(
                    format!("{:?}", result),
                    @r###"Program { module: [Func(Function(Int32, []), "main", Some(Block([Decl(Var(Int32, "a0", Some(Int32(3)))), Decl(Var(Int32, "a1", Some(Int32(64206)))), Decl(Var(Int32, "a2", Some(Int32(47828)))), Decl(Var(Int32, "a3", Some(Int32(511)))), Decl(Var(Float32, "b0", Some(Float32(3.7)))), Decl(Var(Float32, "b1", Some(Float32(2.0)))), Decl(Var(Float32, "b2", Some(Float32(0.9)))), Decl(Var(Float32, "c0", Some(Float32(23000.0)))), Decl(Var(Float32, "c1", Some(Float32(5e-10)))), Decl(Var(Float32, "c2", Some(Float32(1000.0)))), Decl(Var(Float32, "c3", Some(Float32(20000.0)))), Decl(Var(Float32, "c4", Some(Float32(5.0)))), Decl(Var(Float32, "d0", Some(Float32(15.0)))), Decl(Var(Float32, "d1", Some(Float32(1.078125)))), Decl(Var(Float32, "d2", Some(Float32(120.0)))), Decl(Var(Float32, "d3", Some(Float32(120.0)))), Decl(Var(Float32, "d4", Some(Float32(96.0)))), Decl(Var(Float32, "d5", Some(Float32(6.5))))])))] }"###
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
                assert_snapshot!(
                    format!("{:?}", result),
                    @r###"Program { module: [Var(Int32, "n", Some(Int32(3)))] }"###
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
                assert_snapshot!(
                    format!("{:?}", result),
                    @r###"Program { module: [Func(Function(Int32, []), "main", Some(Block([Decl(Var(Int32, "n", Some(Int32(3)))), Return(Some(Var("n")))])))] }"###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }

    #[test]
    fn test_large() {
        let code = r#"
        void move(int n, char pos1, char pos3)
        {
            //打印移动的过程
            // 1代表上面最小的盘子
            // 2代表中间位置的盘子
            // 3代表下面最大的盘子
            printf("盘子%d: 从 %c柱 移动到 %c柱\n", n, pos1, pos3);
        
        }
        
        void Hanoi(int n, char pos1, char pos2, char pos3)
        {
            //如果是1个盘子，直接从起始柱A移动到目标柱C
            if (n == 1) 
            {
                move(n, pos1, pos3);
            }
            else
            {
                //如果盘子大于1个，需要把n-1个盘子，从起始柱pos1，通过目标柱pos3，移动到中转柱pos2
                Hanoi(n-1, pos1, pos3, pos2); 
        
                //此时pos1上的n-1个盘子全部移动pos2上去了，那么可以直接把pos1上剩下的1个盘子，直接移动到pos3上
                move(n, pos1, pos3);
        
                //把pos2剩下的n-1个盘子，通过中转位置pos1，移动到目标位置pos3
                Hanoi(n-1, pos2, pos1, pos3);
            }
        }
        
        int main()
        {
            //盘子个数
            int n = 3;
        
            //起始柱A
            char pos1 = 'A';
        
            //中转柱B
            char pos2 = 'B';
        
            //目标柱C
            char pos3 = 'C';
        
            printf("移动%d个盘子的步骤如下↓\n", n);
        
            //汉诺塔函数
            Hanoi(n, pos1, pos2, pos3);

            return 0;
        }
        "#;
        match parse(code) {
            Ok(result) => {
                assert_snapshot!(
                    format!("{:?}", result),
                    @r###"Program { module: [Func(Function(Void, [TypedIdent { ty: Int32, id: Some("n") }, TypedIdent { ty: Char, id: Some("pos1") }, TypedIdent { ty: Char, id: Some("pos3") }]), "move", Some(Block([Expr(None, Call(Var("printf"), [String("盘子%d: 从 %c柱 移动到 %c柱\\n"), Var("n"), Var("pos1"), Var("pos3")]))]))), Func(Function(Void, [TypedIdent { ty: Int32, id: Some("n") }, TypedIdent { ty: Char, id: Some("pos1") }, TypedIdent { ty: Char, id: Some("pos2") }, TypedIdent { ty: Char, id: Some("pos3") }]), "Hanoi", Some(Block([If(Binary(Eq, Var("n"), Int32(1)), Block([Expr(None, Call(Var("move"), [Var("n"), Var("pos1"), Var("pos3")]))]), Block([Expr(None, Call(Var("Hanoi"), [Binary(Sub, Var("n"), Int32(1)), Var("pos1"), Var("pos3"), Var("pos2")])), Expr(None, Call(Var("move"), [Var("n"), Var("pos1"), Var("pos3")])), Expr(None, Call(Var("Hanoi"), [Binary(Sub, Var("n"), Int32(1)), Var("pos2"), Var("pos1"), Var("pos3")]))]))]))), Func(Function(Int32, []), "main", Some(Block([Decl(Var(Int32, "n", Some(Int32(3)))), Decl(Var(Char, "pos1", Some(Char('A')))), Decl(Var(Char, "pos2", Some(Char('B')))), Decl(Var(Char, "pos3", Some(Char('C')))), Expr(None, Call(Var("printf"), [String("移动%d个盘子的步骤如下↓\\n"), Var("n")])), Expr(None, Call(Var("Hanoi"), [Var("n"), Var("pos1"), Var("pos2"), Var("pos3")])), Return(Some(Int32(0)))])))] }"###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }

    #[test]
    fn test_large_2() {
        let code = r#"
        const int N = 1024;

        void mm(int n, int A[][N], int B[][N], int C[][N]){
            int i, j, k;

            i = 0; j = 0;
            while (i < n){
                j = 0;
                while (j < n){
                    C[i][j] = 0;
                    j = j + 1;
                }
                i = i + 1;
            }

            i = 0; j = 0; k = 0;

            while (k < n){
                i = 0;
                while (i < n){
                    if (A[i][k] == 0){
                        i = i + 1;
                        continue;
                    }
                    j = 0;
                    while (j < n){
                        C[i][j] = C[i][j] + A[i][k] * B[k][j];
                        j = j + 1;
                    }
                    i = i + 1;
                }
                k = k + 1;
            }
        }

        int A[N][N];
        int B[N][N];
        int C[N][N];

        int main(){
            int n = getint();
            int i, j;

            i = 0;
            j = 0;
            while (i < n){
                j = 0;
                while (j < n){
                    A[i][j] = getint();
                    j = j + 1;
                }
                i = i + 1;
            }
            i = 0;
            j = 0;
            while (i < n){
                j = 0;
                while (j < n){
                    B[i][j] = getint();
                    j = j + 1;
                }
                i = i + 1;
            }

            starttime();

            i = 0;
            while (i < 5){    
                mm(n, A, B, C);
                mm(n, A, C, B);
                i = i + 1;
            }

            int ans = 0;
            i = 0;
            while (i < n){
                j = 0;
                while (j < n){
                    ans = ans + B[i][j];
                    j = j + 1;
                }
                i = i + 1;
            }
            stoptime();
            putint(ans);
            putch(10);

            return 0;
        }
        "#;
        match parse(code) {
            Ok(result) => {
                assert_snapshot!(
                    format!("{:?}", result),
                    @r###"Program { module: [Const(Int32, "N", Some(Int32(1024))), Func(Function(Void, [TypedIdent { ty: Int32, id: Some("n") }, TypedIdent { ty: Pointer(Array(Int32, Var("N"))), id: Some("A") }, TypedIdent { ty: Pointer(Array(Int32, Var("N"))), id: Some("B") }, TypedIdent { ty: Pointer(Array(Int32, Var("N"))), id: Some("C") }]), "mm", Some(Block([Decl(Stack([Var(Int32, "i", None), Var(Int32, "j", None), Var(Int32, "k", None)])), Expr(Some(Var("i")), Int32(0)), Expr(Some(Var("j")), Int32(0)), While(Binary(Lt, Var("i"), Var("n")), Block([Expr(Some(Var("j")), Int32(0)), While(Binary(Lt, Var("j"), Var("n")), Block([Expr(Some(Index(Index(Var("C"), Var("i")), Var("j"))), Int32(0)), Expr(Some(Var("j")), Binary(Add, Var("j"), Int32(1)))])), Expr(Some(Var("i")), Binary(Add, Var("i"), Int32(1)))])), Expr(Some(Var("i")), Int32(0)), Expr(Some(Var("j")), Int32(0)), Expr(Some(Var("k")), Int32(0)), While(Binary(Lt, Var("k"), Var("n")), Block([Expr(Some(Var("i")), Int32(0)), While(Binary(Lt, Var("i"), Var("n")), Block([If(Binary(Eq, Index(Index(Var("A"), Var("i")), Var("k")), Int32(0)), Block([Expr(Some(Var("i")), Binary(Add, Var("i"), Int32(1))), Continue]), Block([])), Expr(Some(Var("j")), Int32(0)), While(Binary(Lt, Var("j"), Var("n")), Block([Expr(Some(Index(Index(Var("C"), Var("i")), Var("j"))), Binary(Add, Index(Index(Var("C"), Var("i")), Var("j")), Binary(Mul, Index(Index(Var("A"), Var("i")), Var("k")), Index(Index(Var("B"), Var("k")), Var("j"))))), Expr(Some(Var("j")), Binary(Add, Var("j"), Int32(1)))])), Expr(Some(Var("i")), Binary(Add, Var("i"), Int32(1)))])), Expr(Some(Var("k")), Binary(Add, Var("k"), Int32(1)))]))]))), Var(Array(Array(Int32, Var("N")), Var("N")), "A", None), Var(Array(Array(Int32, Var("N")), Var("N")), "B", None), Var(Array(Array(Int32, Var("N")), Var("N")), "C", None), Func(Function(Int32, []), "main", Some(Block([Decl(Var(Int32, "n", Some(Call(Var("getint"), [])))), Decl(Stack([Var(Int32, "i", None), Var(Int32, "j", None)])), Expr(Some(Var("i")), Int32(0)), Expr(Some(Var("j")), Int32(0)), While(Binary(Lt, Var("i"), Var("n")), Block([Expr(Some(Var("j")), Int32(0)), While(Binary(Lt, Var("j"), Var("n")), Block([Expr(Some(Index(Index(Var("A"), Var("i")), Var("j"))), Call(Var("getint"), [])), Expr(Some(Var("j")), Binary(Add, Var("j"), Int32(1)))])), Expr(Some(Var("i")), Binary(Add, Var("i"), Int32(1)))])), Expr(Some(Var("i")), Int32(0)), Expr(Some(Var("j")), Int32(0)), While(Binary(Lt, Var("i"), Var("n")), Block([Expr(Some(Var("j")), Int32(0)), While(Binary(Lt, Var("j"), Var("n")), Block([Expr(Some(Index(Index(Var("B"), Var("i")), Var("j"))), Call(Var("getint"), [])), Expr(Some(Var("j")), Binary(Add, Var("j"), Int32(1)))])), Expr(Some(Var("i")), Binary(Add, Var("i"), Int32(1)))])), Expr(None, Call(Var("starttime"), [])), Expr(Some(Var("i")), Int32(0)), While(Binary(Lt, Var("i"), Int32(5)), Block([Expr(None, Call(Var("mm"), [Var("n"), Var("A"), Var("B"), Var("C")])), Expr(None, Call(Var("mm"), [Var("n"), Var("A"), Var("C"), Var("B")])), Expr(Some(Var("i")), Binary(Add, Var("i"), Int32(1)))])), Decl(Var(Int32, "ans", Some(Int32(0)))), Expr(Some(Var("i")), Int32(0)), While(Binary(Lt, Var("i"), Var("n")), Block([Expr(Some(Var("j")), Int32(0)), While(Binary(Lt, Var("j"), Var("n")), Block([Expr(Some(Var("ans")), Binary(Add, Var("ans"), Index(Index(Var("B"), Var("i")), Var("j")))), Expr(Some(Var("j")), Binary(Add, Var("j"), Int32(1)))])), Expr(Some(Var("i")), Binary(Add, Var("i"), Int32(1)))])), Expr(None, Call(Var("stoptime"), [])), Expr(None, Call(Var("putint"), [Var("ans")])), Expr(None, Call(Var("putch"), [Int32(10)])), Return(Some(Int32(0)))])))] }"###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }

    #[test]
    fn test_large_3() {
        let code = r#"
        int main(){
            const int a[4][2] = {{1, 2}, {3, 4}, {}, 7};
            const int N = 3;
            int b[4][2] = {};
            int c[4][2] = {1, 2, 3, 4, 5, 6, 7, 8};
            int d[N + 1][2] = {1, 2, {3}, {5}, a[3][0], 8};
            int e[4][2][1] = {{d[2][1], {c[2][1]}}, {3, 4}, {5, 6}, {7, 8}};
            return e[3][1][0] + e[0][0][0] + e[0][1][0] + d[3][0];
        }
        "#;
        match parse(code) {
            Ok(result) => {
                assert_snapshot!(
                    format!("{:?}", result),
                    @r###"Program { module: [Func(Function(Int32, []), "main", Some(Block([Decl(Const(Array(Array(Int32, Int32(2)), Int32(4)), "a", Some(Pack([Pack([Int32(1), Int32(2)]), Pack([Int32(3), Int32(4)]), Pack([]), Int32(7)])))), Decl(Const(Int32, "N", Some(Int32(3)))), Decl(Var(Array(Array(Int32, Int32(2)), Int32(4)), "b", Some(Pack([])))), Decl(Var(Array(Array(Int32, Int32(2)), Int32(4)), "c", Some(Pack([Int32(1), Int32(2), Int32(3), Int32(4), Int32(5), Int32(6), Int32(7), Int32(8)])))), Decl(Var(Array(Array(Int32, Int32(2)), Binary(Add, Var("N"), Int32(1))), "d", Some(Pack([Int32(1), Int32(2), Pack([Int32(3)]), Pack([Int32(5)]), Index(Index(Var("a"), Int32(3)), Int32(0)), Int32(8)])))), Decl(Var(Array(Array(Array(Int32, Int32(1)), Int32(2)), Int32(4)), "e", Some(Pack([Pack([Index(Index(Var("d"), Int32(2)), Int32(1)), Pack([Index(Index(Var("c"), Int32(2)), Int32(1))])]), Pack([Int32(3), Int32(4)]), Pack([Int32(5), Int32(6)]), Pack([Int32(7), Int32(8)])])))), Return(Some(Binary(Add, Binary(Add, Binary(Add, Index(Index(Index(Var("e"), Int32(3)), Int32(1)), Int32(0)), Index(Index(Index(Var("e"), Int32(0)), Int32(0)), Int32(0))), Index(Index(Index(Var("e"), Int32(0)), Int32(1)), Int32(0))), Index(Index(Var("d"), Int32(3)), Int32(0)))))])))] }"###
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
                assert_snapshot!(
                    format!("{:?}", result),
                    @r###"Program { module: [Func(Function(Int32, []), "main", Some(Block([Decl(Var(Int32, "a", None)), Expr(Some(Var("a")), Int32(5)), Return(Some(Var("a")))])))] }"###
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }
}
