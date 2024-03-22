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
    stmt.map(Box::new).parse_next(input)
}

pub fn vec_stmt(input: &mut &str) -> PResult<Vec<Stmt>> {
    repeat(0.., stmt).parse_next(input)
}

/// Expression with semicolon.
pub fn expr_sc(input: &mut &str) -> PResult<Expr> {
    (expr, p(';')).map(|(e, _)| e).parse_next(input)
}

/// Decl or Expr.
pub fn decl_or_expr(input: &mut &str) -> PResult<Either<Decl, Expr>> {
    alt((decl.map(Either::Left), expr_sc.map(Either::Right))).parse_next(input)
}

pub fn stmt(input: &mut &str) -> PResult<Stmt> {
    alt((
        k("break").value(Stmt::Break),
        k("continue").value(Stmt::Continue),
        decl.map(Stmt::Decl),
        expr_sc.map(Stmt::Expr),
        (k("if"), paren(expr), box_stmt, opt((k("else"), box_stmt)))
            .map(|(_, cond, pass, fail)| Stmt::If(cond, pass, fail.map(|(_, s)| s))),
        (k("while"), paren(expr), box_stmt).map(|(_, cond, body)| Stmt::While(cond, body)),
        (k("do"), box_stmt, k("while"), paren(expr))
            .map(|(_, body, _, cond)| Stmt::DoWhile(body, cond)),
        (k("for"), paren((decl_or_expr, expr_sc, expr)), box_stmt)
            .map(|(_, (a, b, c), s)| Stmt::For(a, b, c, s)),
        (k("return"), opt(expr)).map(|(_, e)| Stmt::Return(e)),
        curly(vec_stmt).map(Stmt::Block),
    ))
    .parse_next(input)
}
