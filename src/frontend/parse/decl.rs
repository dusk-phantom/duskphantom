use winnow::error::{ErrMode, ErrorKind};

use super::*;

pub fn make_const(decl: Decl) -> Decl {
    match decl {
        Decl::Var(ty, id, expr) => Decl::Const(ty, id, expr),
        _ => decl,
    }
}

pub fn decl(input: &mut &str) -> PResult<Decl> {
    // Attempt to match a macro.
    if input.starts_with('#') {
        return alt((
            (token("#include"), opt(take_until(0.., '\n')), blank).value(Decl::Stack(vec![])),
            (token("#define"), pad(ident), expr)
                .map(|(_, id, expr)| Decl::Const(Type::Int, id, Some(expr))),
        ))
        .parse_next(input);
    }

    // Match const token.
    let is_const = opt(token("const")).parse_next(input)?.is_some();

    // Consume extern token.
    opt(token("extern")).parse_next(input)?;

    // Parse type.
    let left_type = atom_type.parse_next(input)?;

    // Parse lval and optional assignment expression.
    let mut decls: Vec<Decl> = separated(
        1..,
        |input: &mut &str| assignment(input, left_type.clone()),
        token(","),
    )
    .parse_next(input)?;

    // Require semicolon if the last declaration is not function implementation
    if let Some(Decl::Func(_, _, Some(_))) = decls.last() {
        // Do nothing
    } else {
        token(";").parse_next(input)?;
    }

    // Make constant if necessary
    if is_const {
        decls = decls.into_iter().map(make_const).collect();
    }

    // Return declaration according to count
    match decls.len() {
        1 => Ok(decls.pop().unwrap()),
        _ => Ok(Decl::Stack(decls)),
    }
}

pub fn assignment(input: &mut &str, left_type: Type) -> PResult<Decl> {
    let left_val = lval.parse_next(input)?;
    let typed_ident = acc_lval(left_type, left_val);
    let Some(id) = typed_ident.id else {
        return Err(ErrMode::from_error_kind(input, ErrorKind::Verify).cut());
    };

    // Parse optional assignment.
    if let Some(expr) = opt(preceded(token("="), expr)).parse_next(input)? {
        return Ok(Decl::Var(typed_ident.ty, id, Some(expr)));
    };

    // Parse optional function implementation.
    if let Some(body) = opt(curly(vec_stmt)).parse_next(input)? {
        return Ok(Decl::Func(
            typed_ident.ty,
            id,
            Some(Box::new(Stmt::Block(body))),
        ));
    };

    // Return declaration according to real type
    match typed_ident.ty {
        Type::Function(_, _) => Ok(Decl::Func(typed_ident.ty, id, None)),
        _ => Ok(Decl::Var(typed_ident.ty, id, None)),
    }
}
