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
    alt((
        p('!').value(UnaryOp::Not),
        p('~').value(UnaryOp::Inv),
        p('-').value(UnaryOp::Neg),
        p("++").value(UnaryOp::Inc),
        p("--").value(UnaryOp::Dec),
        p('*').value(UnaryOp::Ind),
        p('&').value(UnaryOp::Addr),
        paren(atom_type).map(|ty| UnaryOp::Cast(ty)),
        k("sizeof").value(UnaryOp::Sizeof),
    ))
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
    alt((
        p('*').value(BinaryOp::Mul),
        p('/').value(BinaryOp::Div),
        p('%').value(BinaryOp::Mod),
    ))
    .parse_next(input)
}

/// Level 1 operators, left to right
pub fn binary_op_lv1(input: &mut &str) -> PResult<BinaryOp> {
    alt((p('+').value(BinaryOp::Add), p('-').value(BinaryOp::Sub))).parse_next(input)
}

/// Level 2 operators, left to right
pub fn binary_op_lv2(input: &mut &str) -> PResult<BinaryOp> {
    alt((p(">>").value(BinaryOp::Shr), p("<<").value(BinaryOp::Shl))).parse_next(input)
}

/// Level 3 operators, left to right
pub fn binary_op_lv3(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        p('>').value(BinaryOp::Gt),
        p('<').value(BinaryOp::Lt),
        p(">=").value(BinaryOp::Ge),
        p("<=").value(BinaryOp::Le),
    ))
    .parse_next(input)
}

/// Level 4 operators, left to right
pub fn binary_op_lv4(input: &mut &str) -> PResult<BinaryOp> {
    alt((p("==").value(BinaryOp::Eq), p("!=").value(BinaryOp::Ne))).parse_next(input)
}

/// Level 5 operators, left to right
pub fn binary_op_lv5(input: &mut &str) -> PResult<BinaryOp> {
    p('&').value(BinaryOp::And).parse_next(input)
}

/// Level 6 operators, left to right
pub fn binary_op_lv6(input: &mut &str) -> PResult<BinaryOp> {
    p('^').value(BinaryOp::Xor).parse_next(input)
}

/// Level 7 operators, left to right
pub fn binary_op_lv7(input: &mut &str) -> PResult<BinaryOp> {
    p('|').value(BinaryOp::Or).parse_next(input)
}

/// Level 8 operators, left to right
pub fn binary_op_lv8(input: &mut &str) -> PResult<BinaryOp> {
    p("&&").value(BinaryOp::All).parse_next(input)
}

/// Level 9 operators, left to right
pub fn binary_op_lv9(input: &mut &str) -> PResult<BinaryOp> {
    p("||").value(BinaryOp::Any).parse_next(input)
}

/// Level 10 operators, RIGHT TO LEFT
pub fn binary_op_lv10(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        p('=').value(BinaryOp::Assign),
        p("+=").value(BinaryOp::AssignAdd),
        p("-=").value(BinaryOp::AssignSub),
        p("*=").value(BinaryOp::AssignMul),
        p("/=").value(BinaryOp::AssignDiv),
        p("%=").value(BinaryOp::AssignMod),
        p(">>=").value(BinaryOp::AssignShr),
        p("<<=").value(BinaryOp::AssignShl),
        p("&=").value(BinaryOp::AssignAnd),
        p("|=").value(BinaryOp::AssignOr),
        p("^=").value(BinaryOp::AssignXor),
    ))
    .parse_next(input)
}
