use anyhow::{anyhow, Context, Result};

use crate::context;

use super::*;

/// A type.
/// Example: *int
#[derive(Clone, PartialEq, Debug)]
pub enum Type {
    /// Nothing. Can only be function return type.
    Void,

    /// 32-bit integer.
    Int,

    /// 32-bit floating-point number.
    Float,

    /// String.
    String,

    /// Character.
    Char,

    /// Boolean.
    Bool,

    /// Pointer to given type.
    /// Example:
    /// `int *` is `Pointer(Int)`
    Pointer(Box<Type>),

    /// Array of given type.
    /// Example:
    /// `int x[4]` is `Array(Int, Int(4))`
    Array(Box<Type>, Box<Expr>),

    /// Function to given type.
    /// Example:
    /// `void (*x)(int)` is `Pointer(Function(Void, [Int]))`
    Function(Box<Type>, Vec<TypedIdent>),

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

/// Default initializers for types.
impl Type {
    pub fn default_initializer(&self) -> Result<Expr> {
        match self {
            Type::Int => Ok(Expr::Int(0)),
            Type::Float => Ok(Expr::Float(0.0)),
            Type::Bool => Ok(Expr::Bool(false)),
            Type::Array(_, _) => Ok(Expr::Zero(self.clone().into())),
            _ => Err(anyhow!("Cannot initialize type {:?}", self)).with_context(|| context!()),
        }
    }
}

/// A left value is an identifier with usage of its type.
/// If identifier is not null, it can be assigned to.
/// Example: `(*f)(int)` indicates that `f` should be used as `(*f)(some_int)`.
#[derive(Clone, PartialEq, Debug)]
pub enum LVal {
    /// Nothing.
    /// Used when there's no target of usage.
    /// Example: `*(int)` has core `Nothing`.
    Nothing,

    /// A single variable.
    /// Example: `x`
    Var(String),

    /// Array indexing.
    /// Example: `x[8]`
    Index(Box<LVal>, Box<Expr>),

    /// A function call.
    /// Example: `f(x, y)`
    Call(Box<LVal>, Vec<TypedIdent>),

    /// Application of indirection.
    /// Example: `*x`, `x[]`
    Pointer(Box<LVal>),
}

/// A typed identifier.
/// `ty`: type
/// `id`: identifier name
/// Example: `int *x` is `{ ty: Pointer(Int), id: "x" }`
#[derive(Clone, PartialEq, Debug)]
pub struct TypedIdent {
    pub ty: Type,
    pub id: Option<String>,
}

impl TypedIdent {
    pub fn new(ty: Type, id: Option<String>) -> Self {
        Self { ty, id }
    }
}
