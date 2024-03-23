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
    (ident, pad(":"), expr)
        .map(|(id, _, expr)| MapEntry::new(id, expr))
        .parse_next(input)
}
