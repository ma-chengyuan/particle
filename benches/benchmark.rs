#[macro_use]
extern crate criterion;
extern crate particle;
extern crate rand;
extern crate rustc_hash;

use criterion::Criterion;
use particle::automatons::{DFA, NFA};
use particle::regex;

fn bench_regex_to_nfa(c: &mut Criterion) {
    c.bench_function("Regex To NFA", |b| {
        b.iter(|| {
            let nfa = regex::compile_regex(r#"\"([^\\\"]|\\.)*\""#);
        })
    });
}

fn bench_nfa_to_dfa(c: &mut Criterion) {
    let nfa = regex::compile_regex(r#"\"([^\\\"]|\\.)*\""#);
    c.bench_function("NFA to DFA", |b| {
        b.iter(|| {
            let dfa = DFA::from(nfa.clone());
        });
    });
}

fn bench_dfa_minimize(c: &mut Criterion) {
    let nfa = regex::compile_regex(r#"\"([^\\\"]|\\.)*\""#);
    let dfa = DFA::from(nfa);
    c.bench_function("DFA Minimize", |b| {
        b.iter(|| {
            let dfa_m = dfa.clone().minimize();
        });
    });
}
criterion_group!(
    benches,
    bench_regex_to_nfa,
    bench_nfa_to_dfa,
    bench_dfa_minimize
);
criterion_main!(benches);
