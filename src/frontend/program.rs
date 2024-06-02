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
    use super::*;

    #[test]
    fn test_minimal() {
        let code = "int main();";
        match parse(code) {
            Ok(result) => {
                assert_eq!(
                    format!("{:?}", result),
                    "Program { module: [Func(Function(Int32, []), \"main\", None)] }"
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
                assert_eq!(
                    format!("{:?}", result),
                    "Program { module: [Var(Int32, \"n\", Some(Int32(3)))] }"
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
                assert_eq!(
                    format!("{:?}", result),
                    "Program { module: [Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"n\", Some(Int32(3)))), Return(Some(Var(\"n\")))])))] }"
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
                assert_eq!(
                    format!("{:?}", result),
                    "Program { module: [Func(Function(Void, [TypedIdent { ty: Int32, id: Some(\"n\") }, TypedIdent { ty: Char, id: Some(\"pos1\") }, TypedIdent { ty: Char, id: Some(\"pos3\") }]), \"move\", Some(Block([Expr(None, Call(Var(\"printf\"), [String(\"盘子%d: 从 %c柱 移动到 %c柱\\\\n\"), Var(\"n\"), Var(\"pos1\"), Var(\"pos3\")]))]))), Func(Function(Void, [TypedIdent { ty: Int32, id: Some(\"n\") }, TypedIdent { ty: Char, id: Some(\"pos1\") }, TypedIdent { ty: Char, id: Some(\"pos2\") }, TypedIdent { ty: Char, id: Some(\"pos3\") }]), \"Hanoi\", Some(Block([If(Binary(Eq, Var(\"n\"), Int32(1)), Block([Expr(None, Call(Var(\"move\"), [Var(\"n\"), Var(\"pos1\"), Var(\"pos3\")]))]), Block([Expr(None, Call(Var(\"Hanoi\"), [Binary(Sub, Var(\"n\"), Int32(1)), Var(\"pos1\"), Var(\"pos3\"), Var(\"pos2\")])), Expr(None, Call(Var(\"move\"), [Var(\"n\"), Var(\"pos1\"), Var(\"pos3\")])), Expr(None, Call(Var(\"Hanoi\"), [Binary(Sub, Var(\"n\"), Int32(1)), Var(\"pos2\"), Var(\"pos1\"), Var(\"pos3\")]))]))]))), Func(Function(Int32, []), \"main\", Some(Block([Decl(Var(Int32, \"n\", Some(Int32(3)))), Decl(Var(Char, \"pos1\", Some(Char('A')))), Decl(Var(Char, \"pos2\", Some(Char('B')))), Decl(Var(Char, \"pos3\", Some(Char('C')))), Expr(None, Call(Var(\"printf\"), [String(\"移动%d个盘子的步骤如下↓\\\\n\"), Var(\"n\")])), Expr(None, Call(Var(\"Hanoi\"), [Var(\"n\"), Var(\"pos1\"), Var(\"pos2\"), Var(\"pos3\")])), Return(Some(Int32(0)))])))] }"
                )
            }
            Err(err) => match err {
                FrontendError::ParseError(s) => panic!("{}", s),
                FrontendError::OptimizeError => panic!("optimize error"),
            },
        }
    }
}
