use super::*;

/// A term that can be evaluated.
/// Example: `f("224")`
#[derive(Clone)]
pub enum Expr {
    /// A single variable.
    /// Example: `x`
    Var(String),

    /// An array, union or struct.
    /// Example: `{ 1, 2, 3 }`
    Pack(Vec<Box<Expr>>),

    /// A named instantiation of an union or struct.
    /// Example: `{ x: 1, y: 2 }` or `{ .x = 1 }`
    Map(Vec<MapEntry>),

    /// Array indexing.
    /// Example: `x[8]`
    Index(Box<Expr>, Box<Expr>),

    /// Field of a value.
    /// Example: `point.x`
    Field(Box<Expr>, String),

    /// A single 32-bit integer.
    /// Example: `8`
    Int32(i32),

    /// A single-precision floating-point number.
    /// Example: `3.6`
    Float32(f32),

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
    Call(Box<Expr>, Vec<Box<Expr>>),

    /// Application of unary operator.
    /// Example: `!false`, `x++`
    Unary(UnaryOp, Box<Expr>),

    /// Application of binary operator.
    /// Example: `a + b`
    Binary(BinaryOp, Box<Expr>, Box<Expr>),

    /// Application of conditional operator.
    /// Example: `cond ? a : b`
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
}

pub fn expr(input: &mut &str) -> PResult<Box<Expr>> {
    let atom = alt((ident1.map(Box::new(Expr::Var)), expr));
    alt((
        curly(separated(0.., expr, pad0(','))).map(|vec| Box::new(Expr::Pack(vec))),
        curly(separated(0.., map_entry, pad0(','))).map(|vec| Box::new(Expr::Map(vec))),
        paren(expr),
        atom.flat_map(|base| {
            repeat(0.., bracket(expr)).fold(|| base, |acc, item| Box::new(Expr::Index(acc, item)))
        }),
    ))
    .parse_next(input)
}
