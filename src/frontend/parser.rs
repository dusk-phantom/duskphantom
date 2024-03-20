use super::*;

pub fn ident(input: &mut &str) -> PResult<String> {
    let head = one_of(('A'..='Z', 'a'..='z', '_')).parse_next(input)?;
    let rest = take_while(0.., ('A'..='Z', 'a'..='z', '0'..='9', '_')).parse_next(input)?;
    Ok(format!("{}{}", head, rest))
}

/// Parser of an integer.
pub fn integer(input: &mut &str) -> PResult<i32> {
    take_while(1.., '0'..'9')
        .map(|s: &str| s.parse().unwrap())
        .parse_next(input)
}

/// Parser of a float.
pub fn float(input: &mut &str) -> PResult<f32> {
    let upper = take_while(1.., '0'..='9').parse_next(input)?;
    let _ = pad0('.').parse_next(input)?;
    let lower = take_while(1.., '0'..='9').parse_next(input)?;
    Ok(format!("{}.{}", upper, lower).parse().unwrap())
}

/// Parser of a string literal.
pub fn string_lit(input: &mut &str) -> PResult<String> {
    // TODO escape
    let _ = '"'.parse_next(input)?;
    let content = take_until(0.., '"').parse_next(input)?;
    let _ = '"'.parse_next(input)?;
    Ok(content.to_string())
}

/// Parser of a char literal.
pub fn char_lit(input: &mut &str) -> PResult<char> {
    // TODO escape
    let _ = '\''.parse_next(input)?;
    let content = any.parse_next(input)?;
    let _ = '\''.parse_next(input)?;
    Ok(content)
}

/// Parser of something wrapped in `()`.
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

/// Parser of something wrapped in `[]`.
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

/// Parser of something wrapped in `{}`.
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

/// Parser of something padded with zero or more spaces .
pub fn pad0<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("pad0", move |input: &mut Input| {
        let _ = space0(input)?;
        let output = parser.parse_next(input)?;
        let _ = space0(input)?;
        Ok(output)
    })
}

/// Parser of something starting with zero or more spaces.
pub fn pre0<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("pre0", move |input: &mut Input| {
        let _ = space0(input)?;
        let output = parser.parse_next(input)?;
        Ok(output)
    })
}

/// Parser of something ending with zero or more spaces.
pub fn suf0<Input, Output, Error, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<Input, Output, Error>
where
    Input: Stream + StreamIsPartial + Compare<char>,
    Error: ParserError<Input>,
    InnerParser: Parser<Input, Output, Error>,
    <Input as Stream>::Token: AsChar,
{
    trace("suf0", move |input: &mut Input| {
        let output = parser.parse_next(input)?;
        let _ = space0(input)?;
        Ok(output)
    })
}

/// Boxed one-time closure that converts Box<T> to T.
pub struct BoxF<T>(Box<dyn FnOnce(Box<T>) -> T>);

impl<T> BoxF<T> {
    pub fn new<F>(f: F) -> Self
    where
        F: FnOnce(Box<T>) -> T + 'static,
    {
        BoxF(Box::new(f))
    }

    pub fn apply(self, t: T) -> T {
        self.0(Box::new(t))
    }
}

/// Left recursion, function depends on tail parser.
///
/// Different from the usual mutual-recursive implementation,
/// this implementation generates all closures first,
/// and then apply the result continuously.
///
/// I'm not using `Repeat::fold` because Rust compiler
/// confuses about the ownership of its arguments.
///
/// Writing mutual-recursive function for each operator is too verbose,
/// and for left-recurse currying is required, bringing lifetime problems,
/// so I made a procedual version for simplicity.
///
/// `head`: parser of the FIRST element of left-recursive chain.
/// `tail`: parser of ALL later elements, returning MUTATION on `head`.
pub fn lrec<I, OH, E, PH, PT>(head: PH, tail: PT) -> impl Parser<I, OH, E>
where
    I: Stream + StreamIsPartial + Compare<char>,
    E: ParserError<I>,
    PH: Parser<I, OH, E>,
    PT: Parser<I, Vec<BoxF<OH>>, E>,
{
    (head, tail).map(move |(base, vec): (_, Vec<BoxF<OH>>)| {
        let mut res = base;
        for ix in vec {
            res = ix.apply(res);
        }
        res
    })
}

/// Right recursion, function depends on init parser.
pub fn rrec<I, OL, E, PL, PI>(init: PI, last: PL) -> impl Parser<I, OL, E>
where
    I: Stream + StreamIsPartial + Compare<char>,
    E: ParserError<I>,
    PI: Parser<I, Vec<BoxF<OL>>, E>,
    PL: Parser<I, OL, E>,
{
    (init, last).map(move |(vec, base): (Vec<BoxF<OL>>, _)| {
        let mut res = base;
        for ix in vec.into_iter().rev() {
            res = ix.apply(res);
        }
        res
    })
}
