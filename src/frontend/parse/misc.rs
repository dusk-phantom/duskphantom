use super::*;

pub fn map_entry(input: &mut &str) -> PResult<MapEntry> {
    (ident, token(":"), expr)
        .map(|(id, _, expr)| MapEntry::new(id, expr))
        .parse_next(input)
}
