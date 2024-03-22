use super::*;

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Int32(i32),
    Float32(f32),
    String(String),
    Char(char),
    Bool(bool),
    Type(Type),
    Ident(String),
    Not,
    Inv,
    Neg,
    Inc,
    Dec,
    Addr,
    Sizeof,
    Assign,
    AssignAdd,
    AssignSub,
    AssignMul,
    AssignDiv,
    AssignMod,
    AssignShr,
    AssignShl,
    AssignAnd,
    AssignOr,
    AssignXor,
    Add,
    Sub,
    Star,
    Div,
    Mod,
    Shr,
    Shl,
    And,
    Or,
    Xor,
    Gt,
    Lt,
    Ge,
    Le,
    Eq,
    Ne,
    All,
    Any,
    QuestionMark,
    Colon,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LCurly,
    RCurly,
    Semicolon,
}

pub fn lex(i: &mut &str) -> PResult<Vec<Token>> {
    preceded(multispace0, repeat(1.., terminated(token, multispace0))).parse_next(i)
}

fn token(i: &mut &str) -> PResult<Token> {
    dispatch! { peek(any);
        '0'..='9' => alt((float.map(Token::Float32), int.map(Token::Int32))),
        '.' => float.map(Token::Float32),
        '(' => '('.value(Token::LParen),
        ')' => ')'.value(Token::RParen),
        '*' => alt((
            "*=".value(Token::AssignMul),
            '*'.value(Token::Star),
        )),
        '/' => alt((
            "/=".value(Token::AssignDiv),
            '/'.value(Token::Div),
        )),
        '+' => alt((
            "+=".value(Token::AssignAdd),
            '+'.value(Token::Add),
        )),
        '-' => alt((
            "-=".value(Token::AssignSub),
            '-'.value(Token::Sub),
        )),
        '>' => alt((
            ">>=".value(Token::AssignShr),
            ">>".value(Token::Shr),
            ">=".value(Token::Ge),
            '>'.value(Token::Gt),
        )),
        '<' => alt((
            "<<=".value(Token::AssignShl),
            "<<".value(Token::Shl),
            "<=".value(Token::Le),
            '<'.value(Token::Lt),
        )),
        '=' => alt((
            "==".value(Token::Eq),
            '='.value(Token::Assign),
        )),
        '&' => alt((
            "&=".value(Token::AssignAnd),
            "&&".value(Token::All),
            '&'.value(Token::And),
        )),
        '^' => alt((
            "^=".value(Token::AssignXor),
            '^'.value(Token::Xor),
        )),
        '|' => alt((
            "|=".value(Token::AssignOr),
            "||".value(Token::Any),
            '|'.value(Token::Or),
        )),
        '!' => alt((
            "!=".value(Token::Ne),
            '!'.value(Token::Not),
        )),
        _ => fail,
    }
    .parse_next(i)
}
