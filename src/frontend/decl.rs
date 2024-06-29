use winnow::error::{ErrMode, ErrorKind};

use super::*;

/// A declaration.
/// Example: `int x = 4;`
#[derive(Clone, PartialEq, Debug)]
pub enum Decl {
    /// A declaration of a constant, optionally with assignment.
    /// Example:
    /// `const int x;` is `Const(Int32, x, None)`
    /// `const int x = 4;` is `Const(Int32, x, Some(Int32(4)))`
    Const(Type, String, Option<Expr>),

    /// A declaration of a variable, optionally with assignment.
    /// Example:
    /// `int x;` is `Var(Int32, x, None)`
    /// `int x = 4;` is `Var(Int32, x, Some(Int32(4)))`
    Var(Type, String, Option<Expr>),

    /// Stacked declarations.
    /// Example:
    /// `int x = 1, y = 2;` is `Stack([Var(Int32, x, Some(Int32(1))), Var(Int32, y, Some(Int32(2)))])`
    Stack(Vec<Decl>),

    /// A declaration of a function, optionally with implementation.
    /// Example:
    /// `void f(int x)` is `Func(Void, "f", [(Int32, (Some("x"))], None)`
    /// `void f() { ... }` is `Func(Void, "f", [], Some(...))`
    Func(Type, String, Option<Box<Stmt>>),

    /// A declaration of an enum.
    /// Example:
    /// `enum fruit { x, y = 114 }` is
    /// `Enum("fruit", vec![("x", None), ("y", 114)])`
    Enum(String, Vec<(String, Option<i32>)>),

    /// A declaration of an union.
    /// Example:
    /// `union numbers { int i; float f; }` is
    /// `Union("numbers", vec![(Int32, "i"), (Float32, "f")])`
    Union(String, Vec<TypedIdent>),

    /// A declaration of a struct.
    /// Example:
    /// `struct numbers { int i; float f; }` is
    /// `Struct("numbers", vec![(Int32, "i"), (Float32, "f")])`
    Struct(String, Vec<TypedIdent>),
}

pub fn make_const(decl: Decl) -> Decl {
    match decl {
        Decl::Var(ty, id, expr) => Decl::Const(ty, id, expr),
        _ => decl,
    }
}

pub fn decl(input: &mut &str) -> PResult<Decl> {
    // Match const token.
    let is_const = opt(token("const")).parse_next(input)?.is_some();

    // Consume extern token.
    opt(token("extern")).parse_next(input)?;

    // Parse type.
    let left_type = atom_type.parse_next(input)?;

    // Parse lval and optional assignment expression.
    let mut decls: Vec<Decl> = separated(
        1..,
        |input: &mut &str| assignment(input, left_type.clone()),
        token(","),
    )
    .parse_next(input)?;

    // Require semicolon if the last declaration is not function implementation
    if let Some(Decl::Func(_, _, Some(_))) = decls.last() {
        // Do nothing
    } else {
        token(";").parse_next(input)?;
    }

    // Make constant if necessary
    if is_const {
        decls = decls.into_iter().map(make_const).collect();
    }

    // Return declaration according to count
    match decls.len() {
        1 => Ok(decls.pop().unwrap()),
        _ => Ok(Decl::Stack(decls)),
    }
}

pub fn assignment(input: &mut &str, left_type: Type) -> PResult<Decl> {
    let left_val = lval.parse_next(input)?;
    let typed_ident = acc_lval(left_type, left_val);
    let Some(id) = typed_ident.id else {
        return Err(ErrMode::from_error_kind(input, ErrorKind::Verify).cut());
    };

    // Parse optional assignment.
    if let Some(expr) = opt(preceded(token("="), expr)).parse_next(input)? {
        return Ok(Decl::Var(typed_ident.ty, id, Some(expr)));
    };

    // Parse optional function implementation.
    if let Some(body) = opt(curly(vec_stmt)).parse_next(input)? {
        return Ok(Decl::Func(
            typed_ident.ty,
            id,
            Some(Box::new(Stmt::Block(body))),
        ));
    };

    // Return declaration according to real type
    match typed_ident.ty {
        Type::Function(_, _) => Ok(Decl::Func(typed_ident.ty, id, None)),
        _ => Ok(Decl::Var(typed_ident.ty, id, None)),
    }
}
