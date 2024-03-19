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

pub fn binary_op(input: &mut &str) -> PResult<BinaryOp> {
    alt((
        '='.value(BinaryOp::Assign),
        '+'.value(BinaryOp::Add),
        '-'.value(BinaryOp::Sub),
        '/'.value(BinaryOp::Div),
        '%'.value(BinaryOp::Mod),
        ">>".value(BinaryOp::Shr),
        "<<".value(BinaryOp::Shl),
        '&'.value(BinaryOp::And),
        '|'.value(BinaryOp::Or),
        '^'.value(BinaryOp::Xor),
        '>'.value(BinaryOp::Gt),
        '<'.value(BinaryOp::Lt),
        ">=".value(BinaryOp::Ge),
        "<=".value(BinaryOp::Le),
        "==".value(BinaryOp::Eq),
        "!=".value(BinaryOp::Ne),
        "&&".value(BinaryOp::All),
        "||".value(BinaryOp::Any),
    ))
    .parse_next(input)
}
