use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use particle::automatons::{DFA, NFA};
use particle::regex;

fn bench_regex_to_nfa(c: &mut Criterion) {
    c.bench_function("Regex To NFA", |b| {
        b.iter(|| {
            let _nfa = regex::compile_regex(r#"\"([^\\\"]|\\.)*\""#).unwrap();
        })
    });
}

fn bench_nfa_to_dfa(c: &mut Criterion) {
    let nfa = regex::compile_regex(r#"\"([^\\\"]|\\.)*\""#).unwrap();
    c.bench_function("NFA to DFA", |b| {
        b.iter(|| {
            let _dfa = DFA::from(nfa.clone());
        });
    });
}

fn bench_dfa_minimize(c: &mut Criterion) {
    let nfa = regex::compile_regex(r#"\"([^\\\"]|\\.)*\""#).unwrap();
    let dfa = DFA::from(nfa);
    c.bench_function("DFA Minimize", |b| {
        b.iter(|| {
            let _dfa_m = dfa.clone().minimize();
        });
    });
}

fn bench_all(c: &mut Criterion) {
    c.bench_function("", |b| {
        b.iter(|| {
            let nfa = regex::compile_regex(r#"\"([^\\\"]|\\.)*\""#).unwrap();
            let _dfa = DFA::from(nfa).minimize();
        });
    });
}

criterion_group!(
    benches,
    bench_regex_to_nfa,
    bench_nfa_to_dfa,
    bench_dfa_minimize,
    bench_all
);
criterion_main!(benches);
