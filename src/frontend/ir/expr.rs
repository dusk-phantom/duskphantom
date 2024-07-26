use super::*;

/// A term that can be evaluated.
/// Example: `f("224")`
#[derive(Clone, PartialEq, Debug)]
pub enum Expr {
    /// A single variable.
    /// Example: `x`
    Var(String),

    /// An array, union or struct.
    /// Example: `{ 1, 2, 3 }`
    Array(Vec<Expr>),

    /// Array indexing.
    /// Example: `x[8]`
    Index(Box<Expr>, Box<Expr>),

    /// A single 32-bit integer.
    /// Example: `8`
    Int(i32),

    /// A single-precision floating-point number.
    /// Example: `3.6`
    Float(f32),

    /// A string literal.
    /// Example: `"good"`
    String(String),

    /// A boolean literal.
    /// Example: `false`
    Bool(bool),

    /// A function call.
    /// Example: `f(x, y)`
    Call(Box<Expr>, Vec<Expr>),

    /// Application of unary operator.
    /// Example: `!false`, `x++`
    Unary(UnaryOp, Box<Expr>),

    /// Application of binary operator.
    /// Example: `a + b`
    Binary(Box<Expr>, Vec<(BinaryOp, Expr)>),
}
