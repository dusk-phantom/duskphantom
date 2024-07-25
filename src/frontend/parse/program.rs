use crate::errors::FrontendError;

use super::*;

pub fn parse(src: &str) -> Result<Program, FrontendError> {
    preceded(blank, repeat(0.., decl))
        .map(Program::new)
        .parse(src)
        .map_err(|err| FrontendError::ParseError(err.to_string()))
}
