use super::*;

/// A record of map assignment.
/// Example: `x: 1`
#[derive(Clone)]
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
