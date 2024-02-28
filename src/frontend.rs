use crate::errors::FrontendError;

/// The full program.
/// A excutable program is a set of modules with an entry module.
/// For now, only one module is supported, so the only module is entry.
pub struct Program {
    /// The module of the program.
    /// Currently only one module is supported.
    pub module: Module,
}

/// A module is a single file.
/// Only declaration can appear at top level.
pub type Module = Vec<Decl>;

/// A declaration.
/// Example: `int x = 4;`
pub enum Decl {
    /// A declaration of a variable, optionally with assignment.
    /// Example:
    /// `int x` is `Var(Int32, x, None)`
    /// `int x = 4` is `Var(Int32, x, Some(Int32(4)))`
    Var(Type, String, Option<Expr>),

    /// A declaration of a function, optionally with implementation.
    /// Example:
    /// `void f(int x)` is `Func(Void, [(Some("x"), Int32)] "f", None)`
    /// `void f() { ... }` is `Func(Void, [], "f", Some(...))`
    Func(Type, Vec<(Option<String>, Type)>, String, Option<Vec<Stmt>>),

    /// A declaration of an enum.
    /// Example:
    /// `enum fruit { x, y = 114 }` is
    /// `Enum("fruit", vec![("x", None), ("y", 114)])`
    Enum(String, Vec<(String, Option<i32>)>),

    /// A declaration of an union.
    /// Example:
    /// `union numbers { int i; float f; }` is
    /// `Union("numbers", vec![(Int32, "i"), (Float32, "f")])`
    Union(String, Vec<(Type, String)>),

    /// A declaration of a struct.
    /// Example:
    /// `struct numbers { int i; float f; }` is
    /// `Struct("numbers", vec![(Int32, "i"), (Float32, "f")])`
    Struct(String, Vec<(Type, String)>),
}

/// A statement.
/// Statements can not appear at top level.
/// Example: `continue`
pub enum Stmt {
    /// A declaration as statement.
    /// Example:
    /// `int x;` is `Decl(Var(Int32, "x"))`
    Decl(Decl),

    /// An expression as statement.
    /// Example:
    /// `x++` is `Expression(UnaryOperator(...))`
    Expr(Expr),

    /// An operation that assign RHS to LHS.
    /// Example:
    /// `x = 4` is `Assign(x, Int32(4))`
    Assign(Expr, Expr),

    /// A conditional branch.
    /// If the third argument is empty, it means there's no else block.
    /// Example:
    /// `if (x == 4) { ... } else { ... }` is `If(Binary(...), [...], [...])`
    If(Expr, Vec<Stmt>, Vec<Stmt>),

    /// A while-loop.
    /// Example:
    /// `while (true) { ... }` is `While(True, [...])`
    While(Expr, Vec<Stmt>),

    /// A do-while-loop.
    /// Example:
    /// `do{...}while(true)` is `DoWhile([...], True)`
    DoWhile(Expr, Vec<Stmt>),

    /// A for-loop.
    /// Example:
    /// `for (x; y; z) { ... }` is `For(x, y, z, [...])`
    For(Box<Decl>, Expr, Expr, Vec<Stmt>),

    /// A break statement.
    Break,

    /// A continue statement.
    Continue,

    /// A return statement.
    /// Example:
    /// `return x` is `Return(x)`
    Return(Expr),

    /// A nested block.
    /// Example:
    /// `{ ... }` is `Vec<Statement>([...])`
    Block(Vec<Stmt>),
}

/// A term that can be evaluated.
/// Example: `f("224")`
pub enum Expr {
    /// A single variable.
    /// Example: `x`
    Var(String),

    /// An array, union or struct.
    /// Example: `{ 1, 2, 3 }`
    Pack(Vec<Expr>),

    /// A named instantiation of an union or struct.
    /// Example: `{ x: 1, y: 2 }` or `{ .x = 1 }`
    Map(Vec<(String, Expr)>),

    /// Array indexing.
    /// Example: `x[8]`
    Index(Box<Expr>, Box<Expr>),

    /// Field of a value.
    /// Example: `point.x`
    Field(Box<Expr>, String),

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
    /// Example: `f(x)`
    Call(Box<Expr>, Vec<Expr>),

    /// Application of unary operator.
    /// Example: `!false`
    Unary(UnaryOp, Box<Expr>),

    /// Application of binary operator.
    /// Example: `a + b`
    Binary(BinaryOp, Box<Expr>, Box<Expr>),

    /// Application of conditional operator.
    /// Example: `cond ? a : b`
    Conditional(Box<Expr>, Box<Expr>, Box<Expr>),
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

    /// Array of given type.
    /// Example:
    /// `int x[4]` is `Array(Int32, [4])`
    Array(Box<Type>, Vec<i32>),

    /// Function pointer to given type.
    /// Example:
    /// `void (*x)(int)` is `Function(Void,[Int32])`
    Function(Box<Type>, Vec<Type>),

    /// Enum of given name.
    /// Example:
    /// `enum fruits` is `Enum("fruits")`
    Enum(String),

    /// Union of given name.
    /// Example:
    /// `union numbers` is `Union("numbers")`
    Union(String),

    /// Struct of given name.
    /// Example:
    /// `struct numbers` is `Struct("numbers")`
    Struct(String),
}

/// Unary operator type.
/// Unlike action, target of unary operator does not need to be a left value.
/// Example: `!`, `~`
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

/// Bianry operator type.
/// Example: `+`, `-`
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
