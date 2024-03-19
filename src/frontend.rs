use rayon::iter::Either;
use winnow::ascii::space0;
use winnow::ascii::space1;
use winnow::combinator::alt;
use winnow::combinator::separated;
use winnow::combinator::trace;
use winnow::error::ParserError;
use winnow::stream::AsChar;
use winnow::stream::Compare;
use winnow::stream::Stream;
use winnow::stream::StreamIsPartial;
use winnow::PResult;
use winnow::Parser;

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
    /// `void f(int x)` is `Func(Void, "f", [(Int32, (Some("x"))], None)`
    /// `void f() { ... }` is `Func(Void, "f", [], Some(...))`
    Func(Type, String, Vec<(Type, Option<String>)>, Option<Vec<Stmt>>),

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
    /// `if (x == 4) ... else ...` is `If(Binary(...), ..., ...)`
    If(Expr, Box<Stmt>, Box<Stmt>),

    /// A while-loop.
    /// Example:
    /// `while (true) ...` is `While(True, ...)`
    While(Expr, Box<Stmt>),

    /// A do-while-loop.
    /// Example:
    /// `do ... while (true)` is `DoWhile(..., True)`
    DoWhile(Box<Stmt>, Expr),

    /// A for-loop.
    /// Example:
    /// `for (x; y; z) ...` is `For(x, y, z, ...)`
    For(Either<Decl, Expr>, Expr, Expr, Box<Stmt>),

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

fn expr(input: &mut &str) -> PResult<Expr> {
    alt((
        ident1.map(Expr::Var),
        curly(separated(1.., expr, pad(','))).map(Expr::Pack),
    ))
    .parse_next(input)
}

/// A type.
/// Example: *int
#[derive(Clone)]
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
    /// `int *` is `Pointer(Int32)`
    Pointer(Box<Type>),

    /// Array of given type.
    /// Example:
    /// `int x[4]` is `Array(Int32, 4)`
    Array(Box<Type>, i32),

    /// Function pointer to given type.
    /// Example:
    /// `void (*x)(int)` is `Function(Void, [Int32])`
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

/// A typed identifier.
/// `ty`: type
/// `id`: identifier name
/// Example: `int *x` is `{ ty: Pointer(Int32), id: "x" }`
#[derive(Clone)]
pub struct TypedIdent {
    pub ty: Type,
    pub id: Option<String>,
}

impl TypedIdent {
    pub fn new(ty: Type, id: Option<String>) -> Self {
        Self { ty, id }
    }
}

fn atom_type(input: &mut &str) -> PResult<Type> {
    alt((
        "void".value(Type::Void),
        "int".value(Type::Int32),
        "float".value(Type::Float32),
        "string".value(Type::String),
        "char".value(Type::Char),
        "bool".value(Type::Boolean),
        ("enum", space1, ident1).map(|(_, _, ty)| Type::Enum(ty)),
        ("union", space1, ident1).map(|(_, _, ty)| Type::Union(ty)),
        ("struct", space1, ident1).map(|(_, _, ty)| Type::Struct(ty)),
    ))
    .parse_next(input)
}

fn typed(input: &mut &str) -> PResult<TypedIdent> {
    alt((
        (atom_type, space1, ident0).map(|(ty, _, id)| TypedIdent::new(ty, id)),
        (atom_type, space1, "*", space0, ident0)
            .map(|(ty, _, _, _, id)| TypedIdent::new(Type::Pointer(Box::new(ty)), id)),
        (atom_type, space1, ident0, space0, bracket(number))
            .map(|(ty, _, id, _, num)| TypedIdent::new(Type::Array(Box::new(ty), num), id)),
    ))
    .parse_next(input)
}

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

fn unary_op(input: &mut &str) -> PResult<UnaryOp> {
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

fn binary_op(input: &mut &str) -> PResult<BinaryOp> {
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

fn ident0(input: &mut &str) -> PResult<Option<String>> {
    // TODO
    Ok(Some(String::from("")))
}

fn ident1(input: &mut &str) -> PResult<String> {
    // TODO
    Ok(String::from(""))
}

fn number(input: &mut &str) -> PResult<i32> {
    // TODO
    Ok(51419)
}

fn bracket<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("bracket", move |input: &mut Input| {
        let _ = '['.parse_next(input)?;
        let _ = space0(input)?;
        let output = parser.parse_next(input)?;
        let _ = space0(input)?;
        let _ = ']'.parse_next(input)?;
        Ok(output)
    })
}

fn curly<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("curly", move |input: &mut Input| {
        let _ = '{'.parse_next(input)?;
        let _ = space0(input)?;
        let output = parser.parse_next(input)?;
        let _ = space0(input)?;
        let _ = '}'.parse_next(input)?;
        Ok(output)
    })
}

fn pad<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("pad", move |input: &mut Input| {
        let _ = space0(input)?;
        let output = parser.parse_next(input)?;
        let _ = space0(input)?;
        Ok(output)
    })
}

pub fn parse(_src: &str) -> Result<Program, FrontendError> {
    Err(FrontendError::ParseError)
}

#[allow(unused)]
pub fn optimize(program: &mut Program) {}
