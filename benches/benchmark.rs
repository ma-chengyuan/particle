#[macro_use]
extern crate criterion;
extern crate particle;
extern crate rand;
extern crate rustc_hash;

use criterion::Criterion;
use particle::automatons::{DFA, NFA};

fn bench_nfa_string(c: &mut Criterion) {
    c.bench_function("NFA String Regex", |b| {
        b.iter(|| {
            let quote = NFA::from('"');
            let non_escape = NFA::from('"').or(&NFA::from('\\')).not();
            let escape = NFA::from('\\').and(&NFA::from(('\u{0000}', '\u{ffff}')));
            let string = quote
                .and(&escape.or(&non_escape).zero_or_more())
                .and(&quote);
        })
    });
}

fn bench_dfa_string(c: &mut Criterion) {
    let quote = NFA::from('"');
    let non_escape = NFA::from('"').or(&NFA::from('\\')).not();
    let escape = NFA::from('\\').and(&NFA::from(('\u{0000}', '\u{ffff}')));
    let string = quote
        .and(&escape.or(&non_escape).zero_or_more())
        .and(&quote);
    c.bench_function("DFA String Regex", |b| {
        b.iter(|| {
            let dfa = DFA::from(&string);
        })
    });
}

fn bench_dfa_to_nfa(c: &mut Criterion) {
    let quote = NFA::from('"');
    let non_escape = NFA::from('"').or(&NFA::from('\\')).not();
    let escape = NFA::from('\\').and(&NFA::from(('\u{0000}', '\u{ffff}')));
    let string = quote
        .and(&escape.or(&non_escape).zero_or_more())
        .and(&quote);
    let dfa = DFA::from(&string);
    c.bench_function("DFA to NFA", |b| {
        b.iter(|| {
            let nfa = NFA::from(&dfa);
        })
    });
}

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

criterion_group!(
    benches,
    bench_nfa_string,
    bench_dfa_string,
    bench_dfa_to_nfa,
    bench_vec_common,
    bench_vec_fold
);
criterion_main!(benches);
