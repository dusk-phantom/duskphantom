/// Unary operator type.
/// Unlike action, target of unary operator does not need to be a left value.
/// Example: `!`, `~`
#[derive(Clone, PartialEq, Debug)]
pub enum UnaryOp {
    /// `!`
    Not,
    /// `-`
    Neg,
    /// `+`
    Pos,
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
