use super::*;

/// A declaration.
/// Example: `int x = 4;`
#[derive(Clone)]
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
