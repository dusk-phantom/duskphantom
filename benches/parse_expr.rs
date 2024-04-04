use compiler::frontend::*;
use criterion::{criterion_group, criterion_main, Criterion};

#[allow(unused)]
fn bench_all(c: &mut Criterion) {
    let code = r#"
void move(int n, char pos1, char pos3)
{
    //打印移动的过程
    // 1代表上面最小的盘子
    // 2代表中间位置的盘子
    // 3代表下面最大的盘子
    printf("盘子%d: 从 %c柱 移动到 %c柱\n", n, pos1, pos3);
 
}
 
void Hanoi(int n, char pos1, char pos2, char pos3)
{
    //如果是1个盘子，直接从起始柱A移动到目标柱C
    if (n == 1) 
    {
        move(n, pos1, pos3);
    }
    else
    {
        //如果盘子大于1个，需要把n-1个盘子，从起始柱pos1，通过目标柱pos3，移动到中转柱pos2
        Hanoi(n-1, pos1, pos3, pos2); 
 
        //此时pos1上的n-1个盘子全部移动pos2上去了，那么可以直接把pos1上剩下的1个盘子，直接移动到pos3上
        move(n, pos1, pos3);
 
        //把pos2剩下的n-1个盘子，通过中转位置pos1，移动到目标位置pos3
        Hanoi(n-1, pos2, pos1, pos3);
    }
}
 
int main()
{
    //盘子个数
    int n = 3;
 
    //起始柱A
    char pos1 = 'A';
 
    //中转柱B
    char pos2 = 'B';
 
    //目标柱C
    char pos3 = 'C';
 
    printf("移动%d个盘子的步骤如下↓\n", n);
 
    //汉诺塔函数
    Hanoi(n, pos1, pos2, pos3);

    return 0;
}
"#;
    c.bench_function("parse_all", |b| b.iter(|| parse(code)));
}

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
        b.iter(|| expr.parse("(1==(1==(1==(1==1))))"))
    });
    c.bench_function("parse_16_rrec", |b| {
        b.iter(|| {
            expr.parse(
                "(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==1))))))))))))))))",
            )
        })
    });
    c.bench_function("parse_64_rrec", |b| {
            b.iter(|| expr.parse("(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==(1==1))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))))"))
        });
}

#[allow(unused)]
fn bench_lrec(c: &mut Criterion) {
    c.bench_function("parse_4_lrec", |b| {
        b.iter(|| expr.parse("((((1==1)==1)==1)==1)"))
    });
    c.bench_function("parse_16_lrec", |b| {
        b.iter(|| {
            expr.parse(
                "((((((((((((((((1==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)",
            )
        })
    });
    c.bench_function("parse_64_lrec", |b| {
            b.iter(|| expr.parse("((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((((1==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)==1)"))
        });
}

#[allow(unused)]
fn bench_eq(c: &mut Criterion) {
    c.bench_function("parse_4_eq", |b| b.iter(|| expr.parse("1==1==1==1==1")));
    c.bench_function("parse_16_eq", |b| {
        b.iter(|| expr.parse("1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1"))
    });
    c.bench_function("parse_64_eq", |b| {
            b.iter(|| expr.parse("1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1==1"))
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
        bench_all,
        // bench_select,
        // bench_rrec,
        // bench_lrec,
        // bench_eq,
        // bench_add,
        // bench_prefix,
}
criterion_main!(benches);
