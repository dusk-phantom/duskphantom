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
    /// `+`
    Pos,
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
        '!' => token("!").value(UnaryOp::Not),
        '~' => token("~").value(UnaryOp::Inv),
        '-' => alt((
            token("--").value(UnaryOp::Dec),
            token("-").value(UnaryOp::Neg),
        )),
        '+' => alt((
            token("++").value(UnaryOp::Inc),
            token("+").value(UnaryOp::Pos),
        )),
        '*' => token("*").value(UnaryOp::Ind),
        '&' => token("&").value(UnaryOp::Addr),
        's' => token("sizeof").value(UnaryOp::Sizeof),
        '(' => paren(single_type).map(UnaryOp::Cast),
        _ => fail,
    }
    .parse_next(input)
}

/// Binry operator type.
/// Example: `+`, `-`
#[derive(Clone, PartialEq, Debug)]
pub enum BinaryOp {
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
    BitAnd,
    /// |
    BitOr,
    /// ^
    BitXor,
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
    And,
    /// ||
    Or,
}

/// Level 0 operators, left to right
pub fn binary_op_lv0(input: &mut &str) -> PResult<BinaryOp> {
    dispatch! { peek(any);
        '*' => token("*").value(BinaryOp::Mul),
        '/' => token("/").value(BinaryOp::Div),
        '%' => token("%").value(BinaryOp::Mod),
        _ => fail,
    }
    .parse_next(input)
}

/// Level 1 operators, left to right
pub fn binary_op_lv1(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        token("+").value(BinaryOp::Add),
        token("-").value(BinaryOp::Sub),
    ))
    .parse_next(input)
}

/// Level 2 operators, left to right
pub fn binary_op_lv2(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        token(">>").value(BinaryOp::Shr),
        token("<<").value(BinaryOp::Shl),
    ))
    .parse_next(input)
}

/// Level 3 operators, left to right
pub fn binary_op_lv3(input: &mut &str) -> PResult<BinaryOp> {
    dispatch! { peek(any);
        '>' => alt((
            token(">=").value(BinaryOp::Ge),
            token(">").value(BinaryOp::Gt),
        )),
        '<' => alt((
            token("<=").value(BinaryOp::Le),
            token("<").value(BinaryOp::Lt),
        )),
        _ => fail,
    }
    .parse_next(input)
}

/// Level 4 operators, left to right
pub fn binary_op_lv4(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        token("==").value(BinaryOp::Eq),
        token("!=").value(BinaryOp::Ne),
    ))
    .parse_next(input)
}

/// Level 5 operators, left to right
pub fn binary_op_lv5(input: &mut &str) -> PResult<BinaryOp> {
    token("&").value(BinaryOp::BitAnd).parse_next(input)
}

/// Level 6 operators, left to right
pub fn binary_op_lv6(input: &mut &str) -> PResult<BinaryOp> {
    token("^").value(BinaryOp::BitXor).parse_next(input)
}

/// Level 7 operators, left to right
pub fn binary_op_lv7(input: &mut &str) -> PResult<BinaryOp> {
    token("|").value(BinaryOp::BitOr).parse_next(input)
}

/// Level 8 operators, left to right
pub fn binary_op_lv8(input: &mut &str) -> PResult<BinaryOp> {
    token("&&").value(BinaryOp::And).parse_next(input)
}

/// Level 9 operators, left to right
pub fn binary_op_lv9(input: &mut &str) -> PResult<BinaryOp> {
    token("||").value(BinaryOp::Or).parse_next(input)
}
