use hexf_parse::parse_hexf32;
use winnow::error::{ErrMode, ErrorKind};

use super::*;

/// Parser of a word that begins with letter,
/// and continues with letters or numbers.
/// For example, `int`, `x114ee`
pub fn word(input: &mut &str) -> PResult<String> {
    let head = one_of(('A'..='Z', 'a'..='z', '_')).parse_next(input)?;
    let rest = take_while(0.., ('A'..='Z', 'a'..='z', '0'..='9', '_')).parse_next(input)?;
    Ok(format!("{}{}", head, rest))
}

/// List of all keywords.
const KEYWORDS: [&str; 20] = [
    "void", "int", "float", "string", "char", "bool", "struct", "enum", "union", "false", "true",
    "sizeof", "break", "continue", "return", "if", "else", "do", "while", "for",
];

/// Parser of an identifier, a word which is not a keyword.
pub fn ident(input: &mut &str) -> PResult<String> {
    word.verify(|x| !KEYWORDS.contains(&x)).parse_next(input)
}

/// Match decimal or hexadecimal numbers.
pub fn match_numbers<'a>(input: &mut &'a str, is_hex: bool, min_count: usize) -> PResult<&'a str> {
    if is_hex {
        take_while(min_count.., ('0'..='9', 'a'..='f', 'A'..='F')).parse_next(input)
    } else {
        take_while(min_count.., '0'..='9').parse_next(input)
    }
}

/// Parser of a usize.
pub fn usize(input: &mut &str) -> PResult<usize> {
    let is_oct_or_hex = opt("0").parse_next(input)?.is_some();
    let is_hex = opt(alt(("x", "X"))).parse_next(input)?.is_some();
    let radix = if is_oct_or_hex {
        8
    } else if is_hex {
        16
    } else {
        10
    };

    // Octal number matches 8 and 9, but will error anyways
    // Empty number throws recoverable error
    let number = match_numbers(input, is_hex, 1)?;
    usize::from_str_radix(number, radix)
        .map_err(|_| ErrMode::from_error_kind(input, ErrorKind::Verify).cut())
}

/// Parser of a constant number.
pub fn constant_number(input: &mut &str) -> PResult<Expr> {
    let hex_prefix = opt(alt(("0x", "0X"))).parse_next(input)?.unwrap_or("");
    let is_hex = !hex_prefix.is_empty();
    let exponent_charset: (&str, &str) = if is_hex { ("p", "P") } else { ("e", "E") };
    let before_point = match_numbers(input, is_hex, 0)?;
    let point = opt(".").parse_next(input)?.unwrap_or("");
    let after_point = match_numbers(input, is_hex, 0)?;
    let exponent_indicator = opt(alt(exponent_charset)).parse_next(input)?.unwrap_or("");
    if point.is_empty() && exponent_indicator.is_empty() {
        // Number is empty, throw recoverable error
        if before_point.is_empty() {
            return Err(ErrMode::from_error_kind(input, ErrorKind::Verify));
        }

        // If number exists, but no point and exponent, parse `before_point` as int instead
        let radix = if is_hex {
            16
        } else if before_point.starts_with('0') {
            8
        } else {
            10
        };
        return i32::from_str_radix(before_point, radix)
            .map_err(|_| ErrMode::from_error_kind(input, ErrorKind::Verify).cut())
            .map(Expr::Int32);
    }

    // Read exponent value only if there is exponent indicator ("e" | "E" | "p" | "P")
    let number = if exponent_indicator.is_empty() {
        format!("{}{}{}{}", hex_prefix, before_point, point, after_point)
    } else {
        let sign = opt(alt(("+", "-"))).parse_next(input)?.unwrap_or("");
        let exponent = match_numbers(input, false, 1).map_err(|e| e.cut())?;
        format!(
            "{}{}{}{}{}{}{}",
            hex_prefix, before_point, point, after_point, exponent_indicator, sign, exponent
        )
    };

    // Parse the number as float
    if is_hex {
        parse_hexf32(&number, false)
            .map_err(|_| ErrMode::from_error_kind(input, ErrorKind::Verify).cut())
            .map(Expr::Float32)
    } else {
        number
            .parse()
            .map_err(|_| ErrMode::from_error_kind(input, ErrorKind::Verify).cut())
            .map(Expr::Float32)
    }
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

/// Parser of blank.
pub fn blank(input: &mut &str) -> PResult<()> {
    (multispace0, alt((line_comment, block_comment, empty)))
        .value(())
        .parse_next(input)
}

/// Parser of blank beginning with line comment.
pub fn line_comment(input: &mut &str) -> PResult<()> {
    ("//", take_until(0.., '\n'), blank)
        .value(())
        .parse_next(input)
}

/// Parser of blank beginning with block comment.
pub fn block_comment(input: &mut &str) -> PResult<()> {
    ("/*", take_until(0.., "*/"), blank)
        .value(())
        .parse_next(input)
}

/// Parser of something wrapped in `()`.
pub fn paren<'s, Output, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<&'s str, Output, ContextError>
where
    InnerParser: Parser<&'s str, Output, ContextError>,
{
    trace("paren", move |input: &mut &'s str| {
        let _ = token("(").parse_next(input)?;
        let output = parser.parse_next(input)?;
        let _ = token(")").parse_next(input)?;
        Ok(output)
    })
}

/// Parser of something wrapped in `[]`.
pub fn bracket<'s, Output, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<&'s str, Output, ContextError>
where
    InnerParser: Parser<&'s str, Output, ContextError>,
{
    trace("bracket", move |input: &mut &'s str| {
        let _ = token("[").parse_next(input)?;
        let output = parser.parse_next(input)?;
        let _ = token("]").parse_next(input)?;
        Ok(output)
    })
}

/// Parser of something wrapped in `{}`.
pub fn curly<'s, Output, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<&'s str, Output, ContextError>
where
    InnerParser: Parser<&'s str, Output, ContextError>,
{
    trace("curly", move |input: &mut &'s str| {
        let _ = token("{").parse_next(input)?;
        let output = parser.parse_next(input)?;
        let _ = token("}").parse_next(input)?;
        Ok(output)
    })
}

/// Parser of a token.
pub fn token<'s>(mut parser: &'static str) -> impl Parser<&'s str, &'s str, ContextError> {
    // Get the first character and length of the token
    let head = parser.chars().next().unwrap();
    let len = parser.chars().count();
    trace("token", move |input: &mut &'s str| {
        let output = parser.parse_next(input)?;

        // The next character after a token can not connect with the token
        if head.is_alphanum() {
            let _ = alt((peek(any), empty.value(' ')))
                .verify(|x: &char| !x.is_alphanum() && *x != '_')
                .parse_next(input)?;
        }

        // The next character of '>' can not be '>' or '='
        if head == '>' && len == 1 {
            let _ = alt((peek(any), empty.value(' ')))
                .verify(|x: &char| *x != '>' && *x != '=')
                .parse_next(input)?;
        }

        // The next character of '<' can not be '<' or '='
        if head == '<' && len == 1 {
            let _ = alt((peek(any), empty.value(' ')))
                .verify(|x: &char| *x != '<' && *x != '=')
                .parse_next(input)?;
        }

        // The next character of '=' can not be '='
        if head == '=' && len == 1 {
            let _ = alt((peek(any), empty.value(' ')))
                .verify(|x: &char| *x != '=')
                .parse_next(input)?;
        }

        // The next character of '!' can not be '='
        if head == '!' && len == 1 {
            let _ = alt((peek(any), empty.value(' ')))
                .verify(|x: &char| *x != '=')
                .parse_next(input)?;
        }

        // The next character of '&' can not be '&'
        if head == '&' && len == 1 {
            let _ = alt((peek(any), empty.value(' ')))
                .verify(|x: &char| *x != '&')
                .parse_next(input)?;
        }

        // The next character of '|' can not be '|'
        if head == '|' && len == 1 {
            let _ = alt((peek(any), empty.value(' ')))
                .verify(|x: &char| *x != '|')
                .parse_next(input)?;
        }

        // Consume some optional blanks
        blank(input)?;
        Ok(output)
    })
}

/// Parser of something ending with zero or more spaces.
pub fn pad<'s, Output, InnerParser>(
    mut parser: InnerParser,
) -> impl Parser<&'s str, Output, ContextError>
where
    InnerParser: Parser<&'s str, Output, ContextError>,
{
    trace("pad", move |input: &mut &'s str| {
        let output = parser.parse_next(input)?;
        blank(input)?;
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
/// Writing mutual-recursive function for each operator is too verbose,
/// and for left-recurse currying is required, bringing lifetime problems,
/// so I made a procedual version for simplicity.
///
/// Note that `lrec` has built-in memoization, so parsing would be O(n).
/// Benchmark results show that lrec / rrec has slightly better performance
/// than hand-written recursive version.
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
