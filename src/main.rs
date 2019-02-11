//! A simple example

use std::fs;

use particle::define_lexer;
use particle::lexer::LexerState;
use particle::span::Span;

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

fn main() {
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

    let contents = fs::read_to_string("benches/large_json.json").unwrap();
    let mut state = LexerState::from(contents.chars());
    let mut cnt = 0usize;
    while !state.eof() {
        if let Ok(token) = lexer.next_token(&mut state) {
            println!("{:?}", token.kind);
            cnt += 1;
        } else {
            eprintln!("Error!");
            break;
        }
    }
    eprintln!("{}", cnt);
}
