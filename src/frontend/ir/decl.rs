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
