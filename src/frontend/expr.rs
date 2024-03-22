use crate::gen_lrec_binary;

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

/// Parse a vector of Expr.
pub fn vec_expr(input: &mut &str) -> PResult<Vec<Expr>> {
    separated(0.., expr, p(',')).parse_next(input)
}

/// Parse a box of Expr.
/// Like an `expr`, but returns the boxed version.
pub fn box_expr(input: &mut &str) -> PResult<Box<Expr>> {
    expr.map(Box::new).parse_next(input)
}

/// Parse unary expressions separated by `*`, `/` and etc.
pub fn unary(input: &mut &str) -> PResult<Expr> {
    let atom = alt((
        p(ident).map(Expr::Var),
        curly(separated(0.., expr, p(','))).map(Expr::Pack),
        curly(separated(0.., map_entry, p(','))).map(Expr::Map),
        p(float).map(Expr::Float32),
        p(int).map(Expr::Int32),
        p(string_lit).map(Expr::String),
        p(char_lit).map(Expr::Char),
        k("false").value(Expr::Bool(false)),
        k("true").value(Expr::Bool(true)),
        paren(expr),
    ));

    // Access: `head[a].b(c)->d`.
    // Tail parsers return mutation on `head`.
    // Closures should be wrapped in `BoxF` for equal sizes.
    // Wrapping all closures with `BoxF` can also fix type inference problems,
    // because all closures have unique types, making `alt` report errors.
    let access_tail = alt((
        bracket(box_expr).map(|x| BoxF::new(|acc| Expr::Index(acc, x))),
        preceded(p('.'), p(ident)).map(|x| BoxF::new(|acc| Expr::Field(acc, x))),
        paren(vec_expr).map(|x| BoxF::new(|acc| Expr::Call(acc, x))),
        preceded(p("->"), p(ident)).map(|x| BoxF::new(|acc| Expr::Select(acc, x))),
    ));
    let access = lrec(atom, repeat(0.., access_tail));

    // Unary operator.
    let unary_init = unary_op.map(|op| BoxF::new(|acc| Expr::Unary(op, acc)));
    rrec(repeat(0.., unary_init), access).parse_next(input)
}

// Generate parser for each level of expressions,
// featuring binary operators.
gen_lrec_binary!(binary_lv0, binary_op_lv0, unary);
gen_lrec_binary!(binary_lv1, binary_op_lv1, binary_lv0);
gen_lrec_binary!(binary_lv2, binary_op_lv2, binary_lv1);
gen_lrec_binary!(binary_lv3, binary_op_lv3, binary_lv2);
gen_lrec_binary!(binary_lv4, binary_op_lv4, binary_lv3);
gen_lrec_binary!(binary_lv5, binary_op_lv5, binary_lv4);
gen_lrec_binary!(binary_lv6, binary_op_lv6, binary_lv5);
gen_lrec_binary!(binary_lv7, binary_op_lv7, binary_lv6);
gen_lrec_binary!(binary_lv8, binary_op_lv8, binary_lv7);
gen_lrec_binary!(binary_lv9, binary_op_lv9, binary_lv8);

/// Parse a conditional expression.
pub fn conditional(input: &mut &str) -> PResult<Expr> {
    // The first expression is memoized, so when there's no condition,
    // there will not be re-parsing.
    let cond = binary_lv9.parse_next(input)?;
    match (p('?'), conditional, p(':'), conditional).parse_next(input) {
        Ok((_, pass, _, fail)) => Ok(Expr::Conditional(
            Box::new(cond),
            Box::new(pass),
            Box::new(fail),
        )),
        Err(_) => Ok(cond),
    }
}

// Generate parser for assignment operators,
// which have the least associativity.
pub fn expr(input: &mut &str) -> PResult<Expr> {
    let lhs = conditional.parse_next(input)?;
    match (p(binary_op_lv10), expr).parse_next(input) {
        Ok((op, rhs)) => Ok(Expr::Binary(op, Box::new(lhs), Box::new(rhs))),
        Err(_) => Ok(lhs),
    }
}

// Unit tests
#[cfg(test)]
pub mod tests_expr {
    use super::*;

    #[test]
    fn test_minimal() {
        let code = "80";
        match int.parse(code) {
            Ok(result) => assert_eq!(result, 80),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_unary() {
        let code = "622.4";
        match unary.parse(code) {
            Ok(result) => assert_eq!(result, Expr::Float32(622.4)),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_int() {
        let code = "117";
        match expr.parse(code) {
            Ok(result) => assert_eq!(result, Expr::Int32(117)),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_plus() {
        let code = "1+1";
        match expr.parse(code) {
            Ok(result) => assert_eq!(
                result,
                Expr::Binary(
                    BinaryOp::Add,
                    Box::new(Expr::Int32(1)),
                    Box::new(Expr::Int32(1))
                )
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_space() {
        let code = "1  +  1";
        match expr.parse(code) {
            Ok(result) => assert_eq!(
                result,
                Expr::Binary(
                    BinaryOp::Add,
                    Box::new(Expr::Int32(1)),
                    Box::new(Expr::Int32(1))
                )
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_precedence() {
        let code = "1 + 1 * 2 - 3";
        match expr.parse(code) {
            Ok(result) => assert_eq!(
                result,
                Expr::Binary(
                    BinaryOp::Sub,
                    Box::new(Expr::Binary(
                        BinaryOp::Add,
                        Box::new(Expr::Int32(1)),
                        Box::new(Expr::Binary(
                            BinaryOp::Mul,
                            Box::new(Expr::Int32(1)),
                            Box::new(Expr::Int32(2))
                        ))
                    )),
                    Box::new(Expr::Int32(3)),
                )
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_consistency() {
        let code = "xy + 85.2 .  x -> y =!- -! 6=7  % 1 ? 1 ? 4 : 5 : 1 ? 4 : 1";
        let another = "(xy+ (( (85.2) .x) ->y)) =(! -(-!6)=(7 %1?(1?4:5):(1?4:1)))";
        match (expr.parse(code), expr.parse(another)) {
            (Ok(result), Ok(answer)) => assert_eq!(result, answer,),
            (Err(err), _) => panic!("failed to parse {}: {}", code, err),
            (_, Err(err)) => panic!("failed to parse {}: {}", another, err),
        }
    }
}
