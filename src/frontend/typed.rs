use super::*;

/// A type.
/// Example: *int
#[derive(Clone, PartialEq, Debug)]
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

    /// Function to given type.
    /// Example:
    /// `void (*x)(int)` is `Pointer(Function(Void, [Int32]))`
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

/// An identifier with usage of its type.
/// Example: `(*f)(int)` indicates that `f` should be used as `(*f)(some_int)`.
#[derive(Clone, PartialEq, Debug)]
pub enum IdentUsage {
    /// Nothing.
    /// Used when there's no target of usage.
    /// Example: `*(int)` has core `Nothing`.
    Nothing,

    /// A single variable.
    /// Example: `x`
    Var(String),

    /// Array indexing.
    /// Example: `x[8]`
    Index(Box<IdentUsage>, i32),

    /// A function call.
    /// Example: `f(x, y)`
    Call(Box<IdentUsage>, Vec<Type>),

    /// Application of indirection.
    /// Example: `*x`
    Pointer(Box<IdentUsage>),
}

/// Parser of an IdentUsage.
pub fn usage(input: &mut &str) -> PResult<IdentUsage> {
    let atom = alt((
        ident.map(IdentUsage::Var),
        paren(usage),
        empty.value(IdentUsage::Nothing),
    ));
    let access_tail = alt((
        pre0(bracket(int)).map(|x| BoxF::new(move |acc| IdentUsage::Index(acc, x))),
        pre0(paren(vec_type)).map(|x| BoxF::new(|acc| IdentUsage::Call(acc, x))),
    ));
    let access = lrec(atom, repeat(0.., access_tail));
    let unary_init = suf0('*').map(|_op| BoxF::new(|acc| IdentUsage::Pointer(acc)));
    rrec(repeat(0.., unary_init), access).parse_next(input)
}

/// A typed identifier.
/// `ty`: type
/// `id`: identifier name
/// Example: `int *x` is `{ ty: Pointer(Int32), id: "x" }`
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

/// Accumulate usage to type, so that it becomes a TypedIdent.
/// For example, `int (*x)(float)` becomes `{int (*)(float), x}`
pub fn acc_usage(ty: Type, usage: IdentUsage) -> TypedIdent {
    match usage {
        IdentUsage::Nothing => TypedIdent::new(ty, None),
        IdentUsage::Var(id) => TypedIdent::new(ty, Some(id)),
        IdentUsage::Index(inner, ix) => acc_usage(Type::Array(Box::new(ty), ix), *inner),
        IdentUsage::Call(inner, arg) => acc_usage(Type::Function(Box::new(ty), arg), *inner),
        IdentUsage::Pointer(inner) => acc_usage(Type::Pointer(Box::new(ty)), *inner),
    }
}

/// Parser of a TypedIdent.
pub fn typed_ident(input: &mut &str) -> PResult<TypedIdent> {
    let ty = atom_type.parse_next(input)?;
    let _ = space0.parse_next(input)?;
    let us = usage.parse_next(input)?;
    Ok(acc_usage(ty, us))
}

/// Parser of a single type.
pub fn single_type(input: &mut &str) -> PResult<Type> {
    typed_ident.map(|ti| ti.ty).parse_next(input)
}

/// Parser of a box of type.
pub fn box_type(input: &mut &str) -> PResult<Box<Type>> {
    single_type.map(Box::new).parse_next(input)
}

/// Parser of a vector of type.
pub fn vec_type(input: &mut &str) -> PResult<Vec<Type>> {
    separated(0.., single_type, pad0(',')).parse_next(input)
}

// Unit tests
#[cfg(test)]
pub mod tests_typed {
    use super::*;

    #[test]
    fn test_atom() {
        let code = "int";
        match atom_type.parse(code) {
            Ok(result) => assert_eq!(result, Type::Int32),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_argname_function_pointer() {
        // Pointer to a function.
        let code = "int (*)(int u)";
        match single_type.parse(code) {
            Ok(result) => assert_eq!(
                result,
                Type::Pointer(Box::new(Type::Function(
                    Box::new(Type::Int32),
                    vec![Type::Int32],
                )))
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_function_pointer() {
        let code = "int (*)(int)";
        match single_type.parse(code) {
            Ok(result) => assert_eq!(
                result,
                Type::Pointer(Box::new(Type::Function(
                    Box::new(Type::Int32),
                    vec![Type::Int32],
                )))
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_name_pointer_function() {
        // Function that returns a pointer.
        let code = "int *u(int)";
        match single_type.parse(code) {
            Ok(result) => assert_eq!(
                result,
                Type::Function(
                    Box::new(Type::Pointer(Box::new(Type::Int32))),
                    vec![Type::Int32],
                )
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_argname_pointer_function() {
        let code = "int *(int u)";
        match single_type.parse(code) {
            Ok(result) => assert_eq!(
                result,
                Type::Function(
                    Box::new(Type::Pointer(Box::new(Type::Int32))),
                    vec![Type::Int32],
                )
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_usage() {
        let code = "*(int)";
        match usage.parse(code) {
            Ok(result) => assert_eq!(
                result,
                IdentUsage::Pointer(Box::new(IdentUsage::Call(
                    Box::new(IdentUsage::Nothing),
                    vec![Type::Int32],
                )))
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_pointer_function() {
        let code = "int *(int)";
        match single_type.parse(code) {
            Ok(result) => assert_eq!(
                result,
                Type::Function(
                    Box::new(Type::Pointer(Box::new(Type::Int32))),
                    vec![Type::Int32],
                )
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_vec_minimal() {
        let code = "int";
        match vec_type.parse(code) {
            Ok(result) => assert_eq!(result, vec![Type::Int32,]),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_vec() {
        let code = "int *, int, int *";
        match vec_type.parse(code) {
            Ok(result) => assert_eq!(
                result,
                vec![
                    Type::Pointer(Box::new(Type::Int32)),
                    Type::Int32,
                    Type::Pointer(Box::new(Type::Int32)),
                ]
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_complicated() {
        // (*(*app)(f))(x) === f(x)
        // app: *(*(int -> int) -> *(int -> int))
        let code = "int (*(*app)(int (*)(int)))(int)";
        match typed_ident.parse(code) {
            Ok(result) => assert_eq!(
                result,
                TypedIdent::new(
                    Type::Pointer(Box::new(Type::Function(
                        Box::new(Type::Pointer(Box::new(Type::Function(
                            Box::new(Type::Int32),
                            vec![Type::Int32],
                        )))),
                        vec![Type::Pointer(Box::new(Type::Function(
                            Box::new(Type::Int32),
                            vec![Type::Int32],
                        )))],
                    ))),
                    Some("app".to_string()),
                ),
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }
}
