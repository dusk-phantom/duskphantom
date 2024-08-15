use compiler::backend::irs::*;
use criterion::black_box;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

use rustc_hash::FxHashSet;
use std::collections::HashSet;

static N: usize = 1000;
static MAX_ID: u32 = 1000;

fn prepare_regs(n: usize) -> Vec<Reg> {
    let mut regs: Vec<Reg> = vec![];
    for _ in 0..n {
        let is_usual: bool = rand::random();
        let id: u32 = rand::random::<u32>() as u32 % MAX_ID;
        regs.push(Reg::new(id, is_usual));
    }
    regs
}

#[allow(unused)]
fn bench_insert(c: &mut Criterion) {
    let regs = prepare_regs(N);
    let insert1 = || {
        let mut rg1 = FxHashSet::default();
        for reg in &regs {
            rg1.insert(black_box(reg));
        }
    };
    let insert2 = || {
        let mut rg2 = HashSet::new();
        for reg in &regs {
            rg2.insert(black_box(*reg));
        }
    };
    let mut group = c.benchmark_group("Insert");
    group.bench_function(BenchmarkId::new("reg_set", 0), |b| b.iter(insert1));
    group.bench_function(BenchmarkId::new("hash_set", 1), |b| b.iter(insert2));
    group.finish();
}

#[allow(unused)]
fn bench_contains(c: &mut Criterion) {
    let regs = prepare_regs(N);
    let mut rg = FxHashSet::default();
    for reg in &regs {
        rg.insert(reg);
    }
    let mut rg2 = HashSet::new();
    for reg in &regs {
        rg2.insert(*reg);
    }
    let contains = || {
        for reg in &regs {
            rg.contains(black_box(reg));
        }
    };
    let contains2 = || {
        for reg in &regs {
            rg2.contains(black_box(reg));
        }
    };
    let mut group = c.benchmark_group("Contains");
    group.bench_function(BenchmarkId::new("reg_set", 0), |b| b.iter(contains));
    group.bench_function(BenchmarkId::new("hash_set", 1), |b| b.iter(contains2));
    group.finish();
}

#[allow(unused)]
fn bench_merge(c: &mut Criterion) {
    let mut rg1 = FxHashSet::default();
    let mut rg2 = FxHashSet::default();

    let mut rg3: HashSet<Reg> = HashSet::new();
    let mut rg4: HashSet<Reg> = HashSet::new();
    for reg in prepare_regs(N) {
        rg1.insert(reg);
        rg3.insert(reg);
    }
    for reg in prepare_regs(N) {
        rg2.insert(reg);
        rg4.insert(reg);
    }
    let merge = || {
        let mut rg: FxHashSet<Reg> = FxHashSet::default();
        rg.extend(rg2.iter().cloned());
        rg.extend(rg2.iter().cloned());
    };
    let merge2 = || {
        let mut rg: HashSet<Reg> = HashSet::new();
        rg.extend(rg3.iter().cloned());
        rg.extend(rg4.iter().cloned());
    };

    let mut group = c.benchmark_group("Merge");
    group.bench_function(BenchmarkId::new("reg_set", 0), |b| b.iter(merge));
    group.bench_function(BenchmarkId::new("hash_set", 1), |b| b.iter(merge2));
    group.finish();
}

#[allow(unused)]
fn bench_clone(c: &mut Criterion) {
    let mut rg1 = FxHashSet::default();
    let mut rg2 = HashSet::new();
    for reg in prepare_regs(N) {
        rg1.insert(reg);
        rg2.insert(reg);
    }
    let clone = || {
        black_box(rg1.clone());
    };
    let clone2 = || {
        black_box(rg2.clone());
    };
    let mut group = c.benchmark_group("Clone");
    group.bench_function(BenchmarkId::new("reg_set", 0), |b| b.iter(clone));
    group.bench_function(BenchmarkId::new("hash_set", 1), |b| b.iter(clone2));
    group.finish();
}

#[allow(unused)]
fn bench_clone_then_retain(c: &mut Criterion) {
    let mut rg1 = FxHashSet::default();
    let mut rg2 = FxHashSet::default();
    let mut rg3 = HashSet::new();
    let mut rg4 = HashSet::new();
    for reg in prepare_regs(N) {
        rg1.insert(reg);
        rg3.insert(reg);
    }
    for reg in prepare_regs(N) {
        rg2.insert(reg);
        rg4.insert(reg);
    }
    let rg1 = black_box(rg1);
    let rg2 = black_box(&rg2);
    let rg3 = black_box(rg3);
    let rg4 = black_box(&rg4);
    let retain = || {
        let mut rg = rg1.clone();
        rg.retain(|reg| rg2.contains(reg));
        black_box(rg);
    };
    let minus = || {
        let mut rg = rg1.clone();
        rg.retain(|r| !rg2.contains(r));
        black_box(rg);
    };
    let retain2 = || {
        let mut rg = rg3.clone();
        rg.retain(|reg| rg4.contains(reg));
        black_box(rg);
    };

    let mut group = c.benchmark_group("CloneThenRetain");
    group.bench_function(BenchmarkId::new("reg_set_retain", 0), |b| b.iter(retain));
    group.bench_function(BenchmarkId::new("reg_set_minus_another", 1), |b| {
        b.iter(minus)
    });
    group.bench_function(BenchmarkId::new("hash_set", 2), |b| b.iter(retain2));
    group.finish();
}

#[allow(unused)]
fn bench_remove(c: &mut Criterion) {
    let regs = prepare_regs(N);
    let mut rg1 = FxHashSet::default();
    let mut rg2 = HashSet::new();
    for reg in regs.iter() {
        rg1.insert(*reg);
        rg2.insert(*reg);
    }
    let regs = black_box(regs);
    let rg1 = black_box(rg1);
    let rg2 = black_box(rg2);
    let remove = || {
        let mut rg = rg1.clone();
        for reg in &regs {
            rg.remove(reg);
        }
    };
    let remove2 = || {
        let mut rg = rg2.clone();
        for reg in &regs {
            rg.remove(reg);
        }
    };
    let mut group = c.benchmark_group("Remove");
    group.bench_function(BenchmarkId::new("reg_set", 0), |b| b.iter(remove));
    group.bench_function(BenchmarkId::new("hash_set", 1), |b| b.iter(remove2));
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default();
    targets =
        bench_contains,
        bench_insert,
        bench_merge,
        bench_remove,
        bench_clone,
        bench_clone_then_retain,
}
criterion_main!(benches);
