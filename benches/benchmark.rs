#[macro_use]
extern crate criterion;
extern crate particle;
extern crate rand;
extern crate rustc_hash;

use criterion::Criterion;
use particle::automatons::{DFA, NFA};

fn bench_vec_common(c: &mut Criterion) {
    let mut data = Vec::new();
    for _ in 0..10000 {
        data.push(rand::random::<u64>());
    }
    c.bench_function("Vec Copy", |b| {
        b.iter(|| {
            use rustc_hash::FxHashSet;
            let mut set = FxHashSet::default();
            for i in &data {
                set.insert(i);
            }
        })
    });
}

fn bench_vec_fold(c: &mut Criterion) {
    let mut data = Vec::new();
    for _ in 0..10000 {
        data.push(rand::random::<u64>());
    }
    c.bench_function("Vec Fold", |b| {
        b.iter(|| {
            use rustc_hash::FxHashSet;
            let set = data.iter().fold(FxHashSet::default(), |mut acc, x| {
                acc.insert(x);
                acc
            });
        })
    });
}

criterion_group!(benches, bench_vec_common, bench_vec_fold);
criterion_main!(benches);
