use super::*;

/// Unary operator type.
/// Unlike action, target of unary operator does not need to be a left value.
/// Example: `!`, `~`
#[derive(Clone)]
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
        '!'.value(UnaryOp::Not),
        '~'.value(UnaryOp::Inv),
        '-'.value(UnaryOp::Neg),
        "++".value(UnaryOp::Inc),
        "--".value(UnaryOp::Dec),
        '*'.value(UnaryOp::Ind),
        '&'.value(UnaryOp::Addr),
        paren(atom_type).map(|ty| UnaryOp::Cast(ty)),
        "sizeof".value(UnaryOp::Sizeof),
    ))
    .parse_next(input)
}

/// Bianry operator type.
/// Example: `+`, `-`
#[derive(Clone)]
pub enum BinaryOp {
    /// =
    Assign,
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
        '*'.value(BinaryOp::Mul),
        '/'.value(BinaryOp::Div),
        '%'.value(BinaryOp::Mod),
    ))
    .parse_next(input)
}

/// Level 1 operators, left to right
pub fn binary_op_lv1(input: &mut &str) -> PResult<BinaryOp> {
    alt(('+'.value(BinaryOp::Add), '-'.value(BinaryOp::Sub))).parse_next(input)
}

/// Level 2 operators, left to right
pub fn binary_op_lv2(input: &mut &str) -> PResult<BinaryOp> {
    alt((">>".value(BinaryOp::Shr), "<<".value(BinaryOp::Shl))).parse_next(input)
}

/// Level 3 operators, left to right
pub fn binary_op_lv3(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        '>'.value(BinaryOp::Gt),
        '<'.value(BinaryOp::Lt),
        ">=".value(BinaryOp::Ge),
        "<=".value(BinaryOp::Le),
    ))
    .parse_next(input)
}

/// Level 4 operators, left to right
pub fn binary_op_lv4(input: &mut &str) -> PResult<BinaryOp> {
    alt(("==".value(BinaryOp::Eq), "!=".value(BinaryOp::Ne))).parse_next(input)
}

/// Level 5 operators, left to right
pub fn binary_op_lv5(input: &mut &str) -> PResult<BinaryOp> {
    '&'.value(BinaryOp::And).parse_next(input)
}

/// Level 6 operators, left to right
pub fn binary_op_lv6(input: &mut &str) -> PResult<BinaryOp> {
    '^'.value(BinaryOp::Xor).parse_next(input)
}

/// Level 7 operators, left to right
pub fn binary_op_lv7(input: &mut &str) -> PResult<BinaryOp> {
    '|'.value(BinaryOp::Or).parse_next(input)
}

/// Level 8 operators, left to right
pub fn binary_op_lv8(input: &mut &str) -> PResult<BinaryOp> {
    "&&".value(BinaryOp::All).parse_next(input)
}

/// Level 9 operators, left to right
pub fn binary_op_lv9(input: &mut &str) -> PResult<BinaryOp> {
    "||".value(BinaryOp::Any).parse_next(input)
}

/// Level 11 operators, RIGHT TO LEFT
pub fn binary_op_lv11(input: &mut &str) -> PResult<BinaryOp> {
    '='.value(BinaryOp::Assign).parse_next(input)
}
