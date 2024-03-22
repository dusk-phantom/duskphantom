use super::*;

/// A record of map assignment.
/// Example: `x: 1`
#[derive(Clone, PartialEq, Debug)]
pub struct MapEntry {
    pub id: String,
    pub expr: Expr,
}

impl MapEntry {
    pub fn new(id: String, expr: Expr) -> Self {
        Self { id, expr }
    }
}

pub fn map_entry(input: &mut &str) -> PResult<MapEntry> {
    (ident, pad0(':'), expr)
        .map(|(id, _, expr)| MapEntry::new(id, expr))
        .parse_next(input)
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

pub fn accumulate_type(ty: Type, expr: Expr) -> TypedIdent {
    match expr {
        Expr::Var(id) => TypedIdent::new(ty, Some(id)),
        Expr::Pack(_) => todo!("pattern matching is not supported"),
        Expr::Map(_) => todo!("pattern matching is not supported"),
        Expr::Index(inner, ix) => {
            if let Expr::Int32(num) = *ix {
                accumulate_type(Type::Array(Box::new(ty), num), *inner)
            } else {
                todo!("dependent array type is not supported")
            }
        }
        Expr::Field(_, _) => todo!("row polymorphism is not supported"),
        Expr::Select(_, _) => todo!("row polymorphism is not supported"),
        Expr::Int32(_) => todo!("literal cannot be pattern-matched"),
        Expr::Float32(_) => todo!("literal cannot be pattern-matched"),
        Expr::String(_) => todo!("literal cannot be pattern-matched"),
        Expr::Char(_) => todo!("literal cannot be pattern-matched"),
        Expr::Bool(_) => todo!("literal cannot be pattern-matched"),
        Expr::Call(func, arg) => accumulate_type(Type::Function(ty, arg), *func),
        Expr::Unary(_, _) => todo!(),
        Expr::Binary(_, _, _) => todo!(),
        Expr::Conditional(_, _, _) => todo!(),
    }
}

pub fn typed_ident(input: &mut &str) -> PResult<TypedIdent> {
    let ty = atom_type.parse_next(input)?;
    let ex = expr.parse_next(input)?;
    Ok(accumulate_type(ty, ex))
}
