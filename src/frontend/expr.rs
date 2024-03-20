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

    /// Field of a pointed value.
    /// Example: `point->x`
    Select(Box<Expr>, String),

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

pub fn vec_expr(input: &mut &str) -> PResult<Vec<Expr>> {
    separated(1.., expr, pad0(',')).parse_next(input)
}

pub fn box_expr(input: &mut &str) -> PResult<Box<Expr>> {
    expr.map(Box::new).parse_next(input)
}

pub fn atom(input: &mut &str) -> PResult<Expr> {
    alt((
        ident.map(Expr::Var),
        curly(separated(0.., expr, pad0(','))).map(Expr::Pack),
        curly(separated(0.., map_entry, pad0(','))).map(Expr::Map),
        integer.map(Expr::Int32),
        float.map(Expr::Float32),
        string_lit.map(Expr::String),
        char_lit.map(Expr::Char),
        "false".value(Expr::Bool(false)),
        "true".value(Expr::Bool(true)),
        paren(expr),
    ))
    .parse_next(input)
}

pub fn bind_rest(init: Expr, input: &mut &str) -> PResult<Expr> {
    alt((
        pre0(bracket(box_expr))
            .flat_map(|x| |input| bind_rest(Expr::Index(Box::new(init), x), input)),
        empty.value(init),
    ))
    .parse_next(input)
}

pub fn expr(input: &mut &str) -> PResult<Expr> {
    // Bind: `head[a].b(c)->d`.
    // Tail parsers return mutation on `head`.
    // Closures should be wrapped in `BoxF` for equal sizes.
    // Using `BoxF` can fix type inference as well.
    let bind_tail = alt((
        pre0(bracket(box_expr)).map(|x| BoxF::new(|acc| Expr::Index(acc, x))),
        preceded(pad0('.'), ident).map(|x| BoxF::new(|acc| Expr::Field(acc, x))),
        pre0(paren(vec_expr)).map(|x| BoxF::new(|acc| Expr::Call(acc, x))),
        preceded(pad0("->"), ident).map(|x| BoxF::new(|acc| Expr::Select(acc, x))),
    ));
    let bind = lrec(atom, repeat(0.., bind_tail));

    // Unary operator.
    let unary_init = unary_op.map(|op| BoxF::new(|acc| Expr::Unary(op, acc)));
    let unary = rrec(repeat(0.., unary_init), bind);

    // Level-0 binary operator.
    let binary_tail =
        (binary_op_lv0, unary).map(|(op, x)| BoxF::new(|acc| Expr::Binary(op, acc, Box::new(x))));
    let binary = lrec(unary, repeat(0.., binary_tail));
    panic!("unimplemented")
}
