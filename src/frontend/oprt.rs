use super::*;

/// Unary operator type.
/// Unlike action, target of unary operator does not need to be a left value.
/// Example: `!`, `~`
#[derive(Clone, PartialEq, Debug)]
pub enum UnaryOp {
    /// `!`
    Not,
    /// `~`
    Inv,
    /// `-`
    Neg,
    /// `++`
    Inc,
    /// `--`
    Dec,
    /// Indirection operator, `*`
    Ind,
    /// Address operator, `&`
    Addr,
    /// Type cast, `(int)`
    Cast(Type),
    /// Size-of, `sizeof`
    Sizeof,
}

pub fn unary_op(input: &mut &str) -> PResult<UnaryOp> {
    dispatch! { peek(any);
        '!' => pad('!').value(UnaryOp::Not),
        '~' => pad('~').value(UnaryOp::Inv),
        '-' => alt((
            pad("--").value(UnaryOp::Dec),
            pad('-').value(UnaryOp::Neg),
        )),
        '+' => pad("++").value(UnaryOp::Inc),
        '*' => pad('*').value(UnaryOp::Ind),
        '&' => pad('&').value(UnaryOp::Addr),
        's' => keyword("sizeof").value(UnaryOp::Sizeof),
        '(' => paren(single_type)
            .map(|ty| UnaryOp::Cast(ty)),
        _ => fail,
    }
    .parse_next(input)
}

/// Bianry operator type.
/// Example: `+`, `-`
#[derive(Clone, PartialEq, Debug)]
pub enum BinaryOp {
    /// =
    Assign,
    /// +=
    AssignAdd,
    /// -=
    AssignSub,
    /// *=
    AssignMul,
    /// /=
    AssignDiv,
    /// %=
    AssignMod,
    /// >>=
    AssignShr,
    /// <<=
    AssignShl,
    /// &=
    AssignAnd,
    /// |=
    AssignOr,
    /// ^=
    AssignXor,
    /// +
    Add,
    /// -
    Sub,
    /// *
    Mul,
    /// /
    Div,
    /// %
    Mod,
    /// >>
    Shr,
    /// <<
    Shl,
    /// &
    And,
    /// |
    Or,
    /// ^
    Xor,
    /// >
    Gt,
    /// <
    Lt,
    /// >=
    Ge,
    /// <=
    Le,
    /// ==
    Eq,
    /// !=
    Ne,
    /// &&
    All,
    /// ||
    Any,
}

/// Level 0 operators, left to right
pub fn binary_op_lv0(input: &mut &str) -> PResult<BinaryOp> {
    dispatch! { peek(any);
        '*' => pad('*').value(BinaryOp::Mul),
        '/' => pad('/').value(BinaryOp::Div),
        '%' => pad('%').value(BinaryOp::Mod),
        _ => fail,
    }
    .parse_next(input)
}

/// Level 1 operators, left to right
pub fn binary_op_lv1(input: &mut &str) -> PResult<BinaryOp> {
    alt((pad('+').value(BinaryOp::Add), pad('-').value(BinaryOp::Sub))).parse_next(input)
}

/// Level 2 operators, left to right
pub fn binary_op_lv2(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        pad(">>").value(BinaryOp::Shr),
        pad("<<").value(BinaryOp::Shl),
    ))
    .parse_next(input)
}

/// Level 3 operators, left to right
pub fn binary_op_lv3(input: &mut &str) -> PResult<BinaryOp> {
    dispatch! { peek(any);
        '>' => alt((
            pad(">=").value(BinaryOp::Ge),
            pad('>').value(BinaryOp::Gt),
        )),
        '<' => alt((
            pad("<=").value(BinaryOp::Le),
            pad('<').value(BinaryOp::Lt),
        )),
        _ => fail,
    }
    .parse_next(input)
}

/// Level 4 operators, left to right
pub fn binary_op_lv4(input: &mut &str) -> PResult<BinaryOp> {
    alt((pad("==").value(BinaryOp::Eq), pad("!=").value(BinaryOp::Ne))).parse_next(input)
}

/// Level 5 operators, left to right
pub fn binary_op_lv5(input: &mut &str) -> PResult<BinaryOp> {
    pad('&').value(BinaryOp::And).parse_next(input)
}

/// Level 6 operators, left to right
pub fn binary_op_lv6(input: &mut &str) -> PResult<BinaryOp> {
    pad('^').value(BinaryOp::Xor).parse_next(input)
}

/// Level 7 operators, left to right
pub fn binary_op_lv7(input: &mut &str) -> PResult<BinaryOp> {
    pad('|').value(BinaryOp::Or).parse_next(input)
}

/// Level 8 operators, left to right
pub fn binary_op_lv8(input: &mut &str) -> PResult<BinaryOp> {
    pad("&&").value(BinaryOp::All).parse_next(input)
}

/// Level 9 operators, left to right
pub fn binary_op_lv9(input: &mut &str) -> PResult<BinaryOp> {
    pad("||").value(BinaryOp::Any).parse_next(input)
}

/// Level 10 operators, RIGHT TO LEFT
pub fn binary_op_lv10(input: &mut &str) -> PResult<BinaryOp> {
    dispatch! { peek(any);
        '=' => pad('=').value(BinaryOp::Assign),
        '+' => pad("+=").value(BinaryOp::AssignAdd),
        '-' => pad("-=").value(BinaryOp::AssignSub),
        '*' => pad("*=").value(BinaryOp::AssignMul),
        '/' => pad("/=").value(BinaryOp::AssignDiv),
        '%' => pad("%=").value(BinaryOp::AssignMod),
        '>' => pad(">>=").value(BinaryOp::AssignShr),
        '<' => pad("<<=").value(BinaryOp::AssignShl),
        '&' => pad("&=").value(BinaryOp::AssignAnd),
        '|' => pad("|=").value(BinaryOp::AssignOr),
        '^' => pad("^=").value(BinaryOp::AssignXor),
        _ => fail,
    }
    .parse_next(input)
}
