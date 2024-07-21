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

    /// A named instantiation of an union or struct.
    /// Example: `{ x: 1, y: 2 }` or `{ .x = 1 }`
    Map(Vec<MapEntry>),

    /// Array indexing.
    /// Example: `x[8]`
    Index(Box<Expr>, Box<Expr>),

    /// Field of a value.
    /// Example: `point.x`
    Field(Box<Expr>, String),

    /// Field of a pointed value.
    /// Example: `point->x`
    Select(Box<Expr>, String),

    /// A single 32-bit integer.
    /// Example: `8`
    Int(i32),

    /// A single-precision floating-point number.
    /// Example: `3.6`
    Float(f32),

    /// A string literal.
    /// Example: `"good"`
    String(String),

    /// A character literal.
    /// Example: `'g'`
    Char(char),

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

    /// Application of conditional operator.
    /// Example: `cond ? a : b`
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
}
