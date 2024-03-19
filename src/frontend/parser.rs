use super::*;

pub fn ident0(input: &mut &str) -> PResult<Option<String>> {
    // TODO
    Ok(Some(String::from("")))
}

pub fn ident1(input: &mut &str) -> PResult<String> {
    // TODO
    Ok(String::from(""))
}

pub fn number(input: &mut &str) -> PResult<i32> {
    // TODO
    Ok(51419)
}

pub fn paren<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("paren", move |input: &mut Input| {
        let _ = '('.parse_next(input)?;
        let _ = space0(input)?;
        let output = parser.parse_next(input)?;
        let _ = space0(input)?;
        let _ = ')'.parse_next(input)?;
        Ok(output)
    })
}

pub fn bracket<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("bracket", move |input: &mut Input| {
        let _ = '['.parse_next(input)?;
        let _ = space0(input)?;
        let output = parser.parse_next(input)?;
        let _ = space0(input)?;
        let _ = ']'.parse_next(input)?;
        Ok(output)
    })
}

pub fn curly<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("curly", move |input: &mut Input| {
        let _ = '{'.parse_next(input)?;
        let _ = space0(input)?;
        let output = parser.parse_next(input)?;
        let _ = space0(input)?;
        let _ = '}'.parse_next(input)?;
        Ok(output)
    })
}

pub fn pad0<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("pad", move |input: &mut Input| {
        let _ = space0(input)?;
        let output = parser.parse_next(input)?;
        let _ = space0(input)?;
        Ok(output)
    })
}

pub fn pre0<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("pad", move |input: &mut Input| {
        let _ = space0(input)?;
        let output = parser.parse_next(input)?;
        Ok(output)
    })
}

pub fn lrec<I, OI, OR, E, PI, PR, F>(init: PI, rest: PR, mut comb: F) -> impl Parser<I, OI, E>
where
    I: Stream + StreamIsPartial + Compare<char>,
    E: ParserError<I>,
    F: FnMut(Box<OI>, OR) -> OI,
    PI: Parser<I, OI, E>,
    PR: Parser<I, Vec<OR>, E>,
{
    (init, rest).map(move |(base, vec): (_, Vec<OR>)| {
        let mut res = base;
        for ix in vec {
            res = comb(Box::new(res), ix);
        }
        res
    })
}
