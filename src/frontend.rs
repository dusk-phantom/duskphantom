use crate::errors::FrontendError;

/// The full program.
pub struct Program {
    /// Blocks of the program.
    /// A block at top level can only be declaration.
    pub blocks: Vec<Declaration>,
}

/// A group of operations.
/// Only declarations can appear at top level.
/// Example: `x = 4;`
pub enum Block {
    /// A declaration.
    /// Example:
    /// `void f(int)` is `Declare(DeclareFunc(...))`
    Declare(Declaration),

    /// A statement.
    /// Example:
    /// `break` is `State(Break)`
    State(Statement),

    /// A nested block.
    /// Example:
    /// `{ ... }` is `Nest([...])`
    Nest(Vec<Block>),
}

/// A declaration.
/// Only declaration can appear at top level.
/// Example: `int x = 4;`
pub enum Declaration {
    /// A declaration of a variable.
    /// Example:
    /// `int x` is `DeclareVariable(Int32, x)`
    DeclareVariable(Type, LeftValue),

    /// A declaration with assignment.
    /// Example:
    /// `int x = 4` is `DefineVariable(Int32, x, Int32(4))`
    DefineVariable(Type, LeftValue, Expression),

    /// A function declaration.
    /// Example:
    /// `void f(int)` is `DeclareFunc(Void, [...], "f")`
    DeclareFunc(Type, Vec<Type>, String),

    /// A function implementation.
    /// Example:
    /// `void f() { ... }` is `ImplementFunc(Void, [], "f", ...)`
    ImplementFunc(Type, Vec<Param>, String, Expression),
}

/// A statement.
/// Statements can not appear at top level.
/// Example: `continue`
pub enum Statement {
    /// A single expression as a block.
    /// Example:
    /// `x++` is `Single(UnaryOperator(...))`
    Single(Expression),

    /// An operation that assign RHS to LHS.
    /// Example:
    /// `x = 4` is `Assign(x, Int32(4))`
    Assign(LeftValue, Expression),

    /// A conditional branch.
    /// If the third argument is empty, it means there's no else block.
    /// Example:
    /// `if (x == 4) { ... } else { ... }` is `If(Binary(...), [...], [...])`
    If(Expression, Vec<Block>, Vec<Block>),

    /// A while-loop.
    /// Example:
    /// `while (true) { ... }` is `While(True, [...])`
    While(Expression, Vec<Block>),

    /// A do-while-loop.
    /// Example:
    /// `while (true) { ... }` is `While(True, [...])`
    DoWhile(Expression, Vec<Block>),

    /// A for-loop.
    /// Example:
    /// `for (x; y; z) { ... }` is `For(x, y, z, [...])`
    For(Box<Block>, Expression, Expression, Vec<Block>),

    /// A break statement.
    Break,

    /// A continue statement.
    Continue,

    /// A return statement.
    /// Example:
    /// `return x` is `Return(x)`
    Return(Expression),
}

/// A term that can be evaluated.
/// Example: `f("224")`
pub enum Expression {
    /// A single variable.
    /// Example: `x`
    Variable(String),

    /// An array.
    /// Example: `{ 1, 2, 3 }`
    Array(Vec<Expression>),

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
    Boolean(bool),

    /// A function call.
    /// Example: `f(x)`
    Call(Box<Expression>, Vec<Expression>),

    /// Application of unary operator.
    /// Example: `!false`
    Unary(UnaryOperator, Box<Expression>),

    /// Application of action.
    /// Example: `x++`
    Action(Action, Box<LeftValue>),

    /// Application of binary operator.
    /// Example: `a + b`
    Binary(BinaryOperator, Box<Expression>, Box<Expression>),

    /// Application of conditional operator.
    /// Example: `cond ? a : b`
    Conditional(Box<Expression>, Box<Expression>, Box<Expression>),
}

/// A type.
/// Example: *int
pub enum Type {
    /// Nothing. Can only be function return type.
    Void,

    /// 32-bit integer.
    Int32,

    /// 32-bit floating-point number.
    Float32,

    /// String.
    String,

    /// Character.
    Char,

    /// Boolean.
    Boolean,

    /// Pointer to given type.
    /// Example:
    /// `*int` is `Pointer(Int32)`
    Pointer(Box<Type>),

    /// Function pointer to given type.
    /// Example:
    /// `void (*x)(int)` is `Function([Int32], Void)`
    Function(Vec<Type>, Box<Type>),
}

/// Parameter of function.
/// Example: `int x`
pub struct Param {
    /// Identifier of parameter.
    pub identifier: String,

    /// Type of parameter.
    pub param_type: Type,
}

/// Left value.
/// Example: `x, a[2]`
pub struct LeftValue {
    /// Name of left value.
    pub identifier: String,

    /// Array indexer of left value.
    pub indexer: Vec<Expression>,
}

/// Unary operator type.
/// Unlike action, target of unary operator does not need to be a left value.
/// Example: `!`, `~`
pub enum UnaryOperator {
    /// `!`
    Not,
    /// `~`
    Inv,
    /// `-`
    Neg,
}

/// Action type.
/// Unlike unary operator, target of action must be a left value.
/// Example: `++`, `--`
pub enum Action {
    /// `++`
    Inc,
    /// `--`
    Dec,
}

/// Bianry operator type.
/// Example: `+`, `-`
pub enum BinaryOperator {
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

pub fn parse(_src: &str) -> Result<Program, FrontendError> {
    Err(FrontendError::ParseError)
}

#[allow(unused)]
pub fn optimize(program: &mut Program) {}
