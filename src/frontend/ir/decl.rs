use super::*;

/// A declaration.
/// Example: `int x = 4;`
#[derive(Clone, PartialEq, Debug)]
pub enum Decl {
    /// A declaration of a constant, optionally with assignment.
    /// Example:
    /// `const int x;` is `Const(Int, x, None)`
    /// `const int x = 4;` is `Const(Int, x, Some(Int(4)))`
    Const(Type, String, Option<Expr>),

    /// A declaration of a variable, optionally with assignment.
    /// Example:
    /// `int x;` is `Var(Int, x, None)`
    /// `int x = 4;` is `Var(Int, x, Some(Int(4)))`
    Var(Type, String, Option<Expr>),

    /// Stacked declarations.
    /// Example:
    /// `int x = 1, y = 2;` is `Stack([Var(Int, x, Some(Int(1))), Var(Int, y, Some(Int(2)))])`
    Stack(Vec<Decl>),

    /// A declaration of a function, optionally with implementation.
    /// Example:
    /// `void f(int x)` is `Func(Void, "f", [(Int, (Some("x"))], None)`
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
    /// `Union("numbers", vec![(Int, "i"), (Float, "f")])`
    Union(String, Vec<TypedIdent>),

    /// A declaration of a struct.
    /// Example:
    /// `struct numbers { int i; float f; }` is
    /// `Struct("numbers", vec![(Int, "i"), (Float, "f")])`
    Struct(String, Vec<TypedIdent>),
}
