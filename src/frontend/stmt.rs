use super::*;

/// A statement.
/// Statements can not appear at top level.
/// Example: `continue`
#[derive(Clone, PartialEq, Debug)]
pub enum Stmt {
    /// A statement of nothing.
    /// Example: `;`
    Nothing,

    /// A declaration as statement.
    /// Example:
    /// `int x;` is `Decl(Var(Int32, "x"))`
    Decl(Decl),

    /// An expression as statement.
    /// Example:
    /// `y = x++;` is `Expr(Var("y"), UnaryOperator(...))`
    Expr(Option<Expr>, Expr),

    /// A conditional branch.
    /// If the third argument is None, it means there's no else block.
    /// Example:
    /// `if (x == 4) ... else ...` is `If(Binary(...), ..., ...)`
    If(Expr, Box<Stmt>, Box<Stmt>),

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
    (expr, cut_err(token(";")))
        .map(|(e, _)| e)
        .parse_next(input)
}

/// Decl or Expr.
pub fn decl_or_expr(input: &mut &str) -> PResult<Either<Decl, Expr>> {
    alt((decl.map(Either::Left), expr_sc.map(Either::Right))).parse_next(input)
}

pub fn stmt(input: &mut &str) -> PResult<Stmt> {
    let disp = dispatch! { peek(any);
        'b' => (token("break"), cut_err(token(";"))).value(Stmt::Break),
        'c' => (token("continue"), cut_err(token(";"))).value(Stmt::Continue),
        'i' => (token("if"), cut_err((paren(expr), box_stmt, opt((token("else"), box_stmt)))))
            .map(|(_, (cond, pass, fail))| Stmt::If(cond, pass, fail.map_or(Stmt::Block(vec![]).into(), |(_, s)| s))),
        'w' => (token("while"), cut_err((paren(expr), box_stmt))).map(|(_, (cond, body))| Stmt::While(cond, body)),
        'd' => (token("do"), cut_err((box_stmt, token("while"), paren(expr), token(";"))))
            .map(|(_, (body, _, cond, _))| Stmt::DoWhile(body, cond)),
        'f' => (token("for"), cut_err((paren((decl_or_expr, expr_sc, expr)), box_stmt)))
            .map(|(_, ((a, b, c), s))| Stmt::For(a, b, c, s)),
        'r' => (token("return"), cut_err((opt(expr), token(";"))))
            .map(|(_, (e, _))| Stmt::Return(e)),
        '{' => curly(cut_err(vec_stmt)).map(Stmt::Block),
        _ => fail
    };
    alt((
        disp,
        decl.map(Stmt::Decl),
        (opt(terminated(expr, token("="))), expr_sc).map(|(lval, expr)| Stmt::Expr(lval, expr)),
        token(";").value(Stmt::Nothing),
    ))
    .parse_next(input)
}
