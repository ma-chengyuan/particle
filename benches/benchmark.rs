use criterion::Criterion;
use criterion::criterion_group;
use criterion::criterion_main;

use particle::automatons::DFA;
use particle::define_lexer;
use particle::lexer::LexerState;
use particle::regex;
use particle::span::Span;

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
    c.bench_function("All possible", |b| {
        b.iter(|| {
            let nfa = regex::compile_regex(r#"\"([^\\\"]|\\.)*\""#).unwrap();
            let _dfa = DFA::from(nfa).minimize();
        });
    });
}

fn bench_lexer(c: &mut Criterion) {
    #[derive(Debug)]
    enum TokenKind {
        Number(f64),
        Str(String),
        Punctuation(String),
        Bool(bool),
        Null,
    }

    #[derive(Debug)]
    struct Token {
        span: Span,
        kind: TokenKind,
    }

    impl Token {
        fn from(span: Span, kind: TokenKind) -> Self {
            Token { span, kind }
        }
    }

    let lexer = define_lexer!(Token =
        discard "[ \n\r\t]+",
        "\"([^\"\\\\]|\\\\([\"\\/bfnrt]|u[0-9a-f][0-9a-f][0-9a-f][0-9a-f]))*\"" =>
            |s, span| Token::from(span, TokenKind::Str(String::from(s))),
        "(-?(0|[1-9][0-9]*)(\\.[0-9]+)?([eE][+\\-]?[0-9]+)?" =>
            |s, span| Token::from(span, TokenKind::Number(s.parse().unwrap())),
        "[{}\\[\\],:]" =>
            |s, span| Token::from(span, TokenKind::Punctuation(String::from(s))),
        "true|false" =>
            |s, span| Token::from(span, TokenKind::Bool(s.parse().unwrap())),
        "null"=>
            |_, span| Token::from(span, TokenKind::Null)
    );

    c.bench_function("Lexer", |b| {
        b.iter(|| {
            use std::fs;
            let contents = fs::read_to_string("benches/large_json.json").unwrap();
            let mut state = LexerState::from(contents.chars());
            let mut cnt = 0usize;
            while !state.eof() {
                if let Ok(_) = lexer.next_token(&mut state) {
                    cnt += 1;
                } else {
                    break;
                }
            }
        })
    });
}

criterion_group!(
    benches,
    // bench_regex_to_nfa,
    // bench_nfa_to_dfa,
    // bench_dfa_minimize,
    // bench_all,
    bench_lexer
);
criterion_main!(benches);
