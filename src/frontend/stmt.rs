use super::*;

/// A statement.
/// Statements can not appear at top level.
/// Example: `continue`
#[derive(Clone, PartialEq, Debug)]
pub enum Stmt {
    /// A declaration as statement.
    /// Example:
    /// `int x;` is `Decl(Var(Int32, "x"))`
    Decl(Decl),

    /// An expression as statement.
    /// Example:
    /// `x++;` is `Expr(UnaryOperator(...))`
    Expr(Expr),

    /// A conditional branch.
    /// If the third argument is None, it means there's no else block.
    /// Example:
    /// `if (x == 4) ... else ...` is `If(Binary(...), ..., ...)`
    If(Expr, Box<Stmt>, Option<Box<Stmt>>),

    /// A while-loop.
    /// Example:
    /// `while (true) ...` is `While(True, ...)`
    While(Expr, Box<Stmt>),

    /// A do-while-loop.
    /// Example:
    /// `do ... while (true)` is `DoWhile(..., True)`
    DoWhile(Box<Stmt>, Expr),

    /// A for-loop.
    /// Example:
    /// `for (x; y; z) ...` is `For(x, y, z, ...)`
    For(Either<Decl, Expr>, Expr, Expr, Box<Stmt>),

    /// A break statement.
    Break,

    /// A continue statement.
    Continue,

    /// A return statement.
    /// Example:
    /// `return x` is `Return(x)`
    Return(Option<Expr>),

    /// A nested block.
    /// Example:
    /// `{ ... }` is `Vec<Statement>([...])`
    Block(Vec<Stmt>),
}

pub fn box_stmt(input: &mut &str) -> PResult<Box<Stmt>> {
    todo!()
}

pub fn vec_stmt(input: &mut &str) -> PResult<Vec<Stmt>> {
    todo!()
}

pub fn atom_stmt(input: &mut &str) -> PResult<Stmt> {
    alt((
        "break".value(Stmt::Break),
        "continue".value(Stmt::Continue),
        decl.map(Stmt::Decl),
        terminated(expr, pad0(';')).map(Stmt::Expr),
        ("if", paren(expr), box_stmt, opt((pad0("else"), box_stmt)))
            .map(|(_, cond, pass, fail)| Stmt::If(cond, pass, fail.map(|(_, s)| s))),
        preceded(("return", space1), opt(expr)).map(Stmt::Return),
        curly(vec_stmt).map(Stmt::Block),
    ))
    .parse_next(input)
}
