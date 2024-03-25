use super::*;

/// A declaration.
/// Example: `int x = 4;`
#[derive(Clone, PartialEq, Debug)]
pub enum Decl {
    /// A declaration of a variable, optionally with assignment.
    /// Example:
    /// `int x;` is `Var(Int32, x, None)`
    /// `int x = 4;` is `Var(Int32, x, Some(Int32(4)))`
    Var(Type, String, Option<Expr>),

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

pub fn decl(input: &mut &str) -> PResult<Decl> {
    // TODO declaration for enum / union / struct
    let (ty, id) = typed_ident
        .verify_map(|ti| ti.id.map(|id| (ti.ty, id)))
        .parse_next(input)?;

    // Assign a variable.
    if let Ok((_, (ex, _))) = (pad("="), cut_err((expr, pad(";")))).parse_next(input) {
        return Ok(Decl::Var(ty, id, Some(ex)));
    }

    // Implement a function.
    if let Ok(st) = curly(vec_stmt).parse_next(input) {
        return Ok(Decl::Func(ty, id, Some(Box::new(Stmt::Block(st)))));
    }

    // Pure declaration ends with `;`.
    let _ = cut_err(pad(";")).parse_next(input)?;
    match ty {
        Type::Function(_, _) => Ok(Decl::Func(ty, id, None)),
        _ => Ok(Decl::Var(ty, id, None)),
    }
}
