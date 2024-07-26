use crate::gen_lrec_binary;

use super::*;

/// Parse a vector of Expr.
pub fn vec_expr(input: &mut &str) -> PResult<Vec<Expr>> {
    separated(0.., expr, token(",")).parse_next(input)
}

/// Parse a box of Expr.
/// Like an `expr`, but returns the boxed version.
pub fn box_expr(input: &mut &str) -> PResult<Box<Expr>> {
    expr.map(Box::new).parse_next(input)
}

/// Parse prefix expressions like `!x->y.z`.
pub fn prefix(input: &mut &str) -> PResult<Expr> {
    let disp = dispatch! { peek(any);
        '{' => curly(separated(0.., expr, token(","))).map(Expr::Array),
        '.' | '0'..='9' => pad(constant_number),
        '"' => pad(string_lit).map(Expr::String),
        'f' => token("false").value(Expr::Int(0)),
        't' => token("true").value(Expr::Int(1)),
        '(' => paren(expr),
        _ => fail,
    };
    let atom = alt((disp, pad(ident).map(Expr::Var)));

    // Postfix: `head[a].b(c)->d`.
    // Tail parsers return mutation on `head`.
    // Closures should be wrapped in `BoxF` for equal sizes.
    // Wrapping all closures with `BoxF` can also fix type inference problems,
    // because all closures have unique types, making `alt` report errors.
    let postfix_tail = dispatch! { peek(any);
        '[' => bracket(box_expr).map(|x| BoxF::new(|acc| Expr::Index(acc, x))),
        '(' => paren(vec_expr).map(|x| BoxF::new(|acc| Expr::Call(acc, x))),
        _ => fail,
    };
    let postfix = lrec(atom, repeat(0.., postfix_tail));

    // Prefix unary operator.
    let prefix_init = unary_op.map(|op| BoxF::new(|acc| Expr::Unary(op, acc)));
    rrec(repeat(0.., prefix_init), postfix).parse_next(input)
}

// Generate parser for each level of expressions,
// featuring binary operators.
gen_lrec_binary!(binary_lv0, binary_op_lv0, prefix);
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
pub fn expr(input: &mut &str) -> PResult<Expr> {
    binary_lv9.parse_next(input)
}
