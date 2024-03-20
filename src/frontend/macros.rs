#[macro_export]
macro_rules! gen_lrec_binary {
    ($fn_name:ident, $op_name:ident, $base:ident) => {
        // TODO memoize first
        pub fn $fn_name(input: &mut &str) -> PResult<Expr> {
            let p0 = (pad0($op_name), $base);
            let p1 = p0.map(|(op, x)| BoxF::new(|acc| Expr::Binary(op, acc, Box::new(x))));
            lrec($base, repeat(0.., p1)).parse_next(input)
        }
    };
}

#[macro_export]
macro_rules! gen_rrec_binary {
    ($fn_name:ident, $op_name:ident, $base:ident) => {
        // TODO memoize first
        pub fn $fn_name(input: &mut &str) -> PResult<Expr> {
            let p0 = ($base, pad0($op_name));
            let p1 = p0.map(|(x, op)| BoxF::new(|acc| Expr::Binary(op, acc, Box::new(x))));
            rrec(repeat(0.., p1), $base).parse_next(input)
        }
    };
}
