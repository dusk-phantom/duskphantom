use super::*;

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

pub fn atom_type(input: &mut &str) -> PResult<Type> {
    alt((
        "void".value(Type::Void),
        "int".value(Type::Int32),
        "float".value(Type::Float32),
        "string".value(Type::String),
        "char".value(Type::Char),
        "bool".value(Type::Boolean),
        ("enum", space1, ident).map(|(_, _, ty)| Type::Enum(ty)),
        ("union", space1, ident).map(|(_, _, ty)| Type::Union(ty)),
        ("struct", space1, ident).map(|(_, _, ty)| Type::Struct(ty)),
    ))
    .parse_next(input)
}
