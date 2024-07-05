use winnow::{
    ascii::take_escaped,
    combinator::{alt, opt, repeat},
    token::take_until,
    PResult, Parser,
};

pub struct TestResult {
    pub path: String,
    pub line: usize,
    pub col: usize,
    pub left: String,
    pub right: String,
}

pub fn parse(input: &mut &str) -> PResult<Vec<TestResult>> {
    repeat(0.., opt(test_result))
        .map(|results: Vec<Option<TestResult>>| results.into_iter().flatten().collect())
        .parse_next(input)
}

pub fn test_result(input: &mut &str) -> PResult<TestResult> {
    (
        "thread '",
        take_until(0.., '\'').map(|path: &str| path.to_string()),
        " panicked at ",
        take_until(0.., ':').map(|line: &str| line.parse().unwrap()),
        ":",
        take_until(0.., ':').map(|col: &str| col.parse().unwrap()),
        take_until(0.., "left: \""),
        "left: \"",
        take_until(0.., '"').map(|left: &str| left.to_string()),
        "\"\n  right: \"",
        take_until(0.., '"').map(|right: &str| right.to_string()),
    )
        .map(
            |(_, path, _, line, _, col, _, _, left, _, right)| TestResult {
                path,
                line,
                col,
                left,
                right,
            },
        )
        .parse_next(input)
}
