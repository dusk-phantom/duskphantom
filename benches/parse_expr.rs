use compiler::frontend::*;
use criterion::{criterion_group, criterion_main, Criterion};

#[allow(unused)]
fn bench_select(c: &mut Criterion) {
    c.bench_function("parse_4_select", |b| b.iter(|| expr.parse("1->x->x->x->x")));
    c.bench_function("parse_16_select", |b| {
        b.iter(|| expr.parse("1->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x"))
    });
    c.bench_function("parse_64_select", |b| {
            b.iter(|| expr.parse("1->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x->x"))
        });
}

#[allow(unused)]
fn bench_rrec(c: &mut Criterion) {
    c.bench_function("parse_4_rrec", |b| {
        b.iter(|| expr.parse("(1^=(1^=(1^=(1^=1))))"))
    });
    c.bench_function("parse_16_rrec", |b| {
        b.iter(|| {
            expr.parse(
                "(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=1))))))))))))))))",
            )
        })
    });
    c.bench_function("parse_64_rrec", |b| {
            b.iter(|| expr.parse("(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=(1^=1))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))"))
        });
}

#[allow(unused)]
fn bench_lrec(c: &mut Criterion) {
    c.bench_function("parse_4_lrec", |b| {
        b.iter(|| expr.parse("((((1^=1)^=1)^=1)^=1)"))
    });
    c.bench_function("parse_16_lrec", |b| {
        b.iter(|| {
            expr.parse(
                "((((((((((((((((1^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)",
            )
        })
    });
    c.bench_function("parse_64_lrec", |b| {
            b.iter(|| expr.parse("((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((1^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)^=1)"))
        });
}

#[allow(unused)]
fn bench_xoreq(c: &mut Criterion) {
    c.bench_function("parse_4_xoreq", |b| b.iter(|| expr.parse("1^=1^=1^=1^=1")));
    c.bench_function("parse_16_xoreq", |b| {
        b.iter(|| expr.parse("1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1"))
    });
    c.bench_function("parse_64_xoreq", |b| {
            b.iter(|| expr.parse("1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1^=1"))
        });
}

#[allow(unused)]
fn bench_add(c: &mut Criterion) {
    c.bench_function("parse_4_add", |b| b.iter(|| expr.parse("1+1+1+1+1")));
    c.bench_function("parse_16_add", |b| {
        b.iter(|| expr.parse("1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1"))
    });
    c.bench_function("parse_64_add", |b| {
            b.iter(|| expr.parse("1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1+1"))
        });
}

#[allow(unused)]
fn bench_prefix(c: &mut Criterion) {
    c.bench_function("parse_4_prefix", |b| b.iter(|| expr.parse("!!!!1")));
    c.bench_function("parse_16_prefix", |b| {
        b.iter(|| expr.parse("!!!!!!!!!!!!!!!!1"))
    });
    c.bench_function("parse_64_prefix", |b| {
        b.iter(|| expr.parse("!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!1"))
    });
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets =
        bench_select,
        bench_rrec,
        bench_lrec,
        bench_xoreq,
        bench_add,
        bench_prefix,
}
criterion_main!(benches);
