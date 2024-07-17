#[macro_export]
macro_rules! gen_lrec_binary {
    ($fn_name:ident, $op_name:ident, $base:ident) => {
        pub fn $fn_name(input: &mut &str) -> PResult<Expr> {
            let head = $base.parse_next(input)?;
            let tail: Vec<(BinaryOp, Expr)> = repeat(0.., ($op_name, $base)).parse_next(input)?;
            Ok(if (tail.is_empty()) {
                head
            } else {
                Expr::Binary(Box::new(head), tail)
            })
        }
    };
}
