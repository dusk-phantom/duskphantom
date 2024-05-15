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
    Array(Box<Type>, usize),

    /// Function to given type.
    /// Example:
    /// `void (*x)(int)` is `Pointer(Function(Void, [Int32]))`
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

pub fn atom_type(input: &mut &str) -> PResult<Type> {
    alt((
        token("void").value(Type::Void),
        token("int").value(Type::Int32),
        token("float").value(Type::Float32),
        token("string").value(Type::String),
        token("char").value(Type::Char),
        token("bool").value(Type::Boolean),
        (token("enum"), ident).map(|(_, ty)| Type::Enum(ty)),
        (token("union"), ident).map(|(_, ty)| Type::Union(ty)),
        (token("struct"), ident).map(|(_, ty)| Type::Struct(ty)),
    ))
    .parse_next(input)
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
    Index(Box<LVal>, usize),

    /// A function call.
    /// Example: `f(x, y)`
    Call(Box<LVal>, Vec<TypedIdent>),

    /// Application of indirection.
    /// Example: `*x`, `x[]`
    Pointer(Box<LVal>),
}

/// Parser of an left value.
pub fn lval(input: &mut &str) -> PResult<LVal> {
    let atom = alt((
        pad(ident).map(LVal::Var),
        paren(lval),
        empty.value(LVal::Nothing),
    ));
    let access_tail = alt((
        bracket(opt(pad(usize))).map(|x| {
            // Empty bracket is pointer, non-empty bracket is index.
            BoxF::new(move |acc| match x {
                Some(ix) => LVal::Index(acc, ix),
                None => LVal::Pointer(acc),
            })
        }),
        paren(vec_typed).map(|x| BoxF::new(|acc| LVal::Call(acc, x))),
    ));
    let access = lrec(atom, repeat(0.., access_tail));
    let unary_init = token("*").map(|_op| BoxF::new(LVal::Pointer));
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
pub fn acc_lval(ty: Type, usage: LVal) -> TypedIdent {
    match usage {
        LVal::Nothing => TypedIdent::new(ty, None),
        LVal::Var(id) => TypedIdent::new(ty, Some(id)),
        LVal::Index(inner, ix) => acc_lval(Type::Array(Box::new(ty), ix), *inner),
        LVal::Call(inner, arg) => acc_lval(Type::Function(Box::new(ty), arg), *inner),
        LVal::Pointer(inner) => acc_lval(Type::Pointer(Box::new(ty)), *inner),
    }
}

/// Parser of a TypedIdent.
pub fn typed_ident(input: &mut &str) -> PResult<TypedIdent> {
    let ty = atom_type.parse_next(input)?;
    let us = lval.parse_next(input)?;
    Ok(acc_lval(ty, us))
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
pub fn vec_typed(input: &mut &str) -> PResult<Vec<TypedIdent>> {
    separated(0.., typed_ident, token(",")).parse_next(input)
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
    fn test_space() {
        // Pointer to a function.
        let code = "int  (  *  )  (  int  u  )";
        match single_type.parse(code) {
            Ok(result) => assert_eq!(
                result,
                Type::Pointer(Box::new(Type::Function(
                    Box::new(Type::Int32),
                    vec![TypedIdent::new(Type::Int32, Some("u".to_string()))],
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
                    vec![TypedIdent::new(Type::Int32, None)],
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
                    vec![TypedIdent::new(Type::Int32, None)],
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
                    vec![TypedIdent::new(Type::Int32, Some("u".to_string()))],
                )
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_usage() {
        let code = "*(int)";
        match lval.parse(code) {
            Ok(result) => assert_eq!(
                result,
                LVal::Pointer(Box::new(LVal::Call(
                    Box::new(LVal::Nothing),
                    vec![TypedIdent::new(Type::Int32, None)],
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
                    vec![TypedIdent::new(Type::Int32, None)],
                )
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }

    #[test]
    fn test_array_pointer() {
        let code = "int x[][4]";
        match single_type.parse(code) {
            Ok(result) => assert_eq!(
                result,
                Type::Pointer(Box::new(Type::Array(Box::new(Type::Int32), 4)))
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
                            vec![TypedIdent::new(Type::Int32, None)],
                        )))),
                        vec![TypedIdent::new(
                            Type::Pointer(Box::new(Type::Function(
                                Box::new(Type::Int32),
                                vec![TypedIdent::new(Type::Int32, None)],
                            ))),
                            None
                        )],
                    ))),
                    Some("app".to_string()),
                ),
            ),
            Err(err) => panic!("failed to parse {}: {}", code, err),
        }
    }
}
