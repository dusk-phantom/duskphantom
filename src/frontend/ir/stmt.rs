use rayon::iter::Either;

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
    /// `int x;` is `Decl(Var(Int, "x"))`
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
