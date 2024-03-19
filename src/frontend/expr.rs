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
    Pack(Vec<Expr>),

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
    Call(Box<Expr>, Vec<Expr>),

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

pub fn expr(input: &mut &str) -> PResult<Expr> {
    let atom = alt((
        ident1.map(Expr::Var),
        curly(separated(0.., expr, pad0(','))).map(Expr::Pack),
        curly(separated(0.., map_entry, pad0(','))).map(Expr::Map),
        paren(expr),
    ));
    let field = lrec(atom, repeat(0.., preceded(pad0('.'), ident1)), Expr::Field);
    let mut index = lrec(field, repeat(0.., pre0(bracket(expr))), |x, y| {
        Expr::Index(x, Box::new(y))
    });
    index.parse_next(input)
}
