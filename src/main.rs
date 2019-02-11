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
        discard r#"[ \n\r\t]+"#,
        r#""([^"\\]|\\(["\\/bfnrt]|u[0-9a-f][0-9a-f][0-9a-f][0-9a-f]))*""# =>
            |s, span| Token::from(span, TokenKind::Str(String::from(s))),
        r#"-?(0|[1-9][0-9]*)(\.[0-9]+)?([eE][\+\-]?[0-9]+)?"# =>
            |s, span| Token::from(span, TokenKind::Number(s.parse().unwrap())),
        r#"[{}\[\],:]"# =>
            |s, span| Token::from(span, TokenKind::Punctuation(String::from(s))),
        r#"true|false"# =>
            |s, span| Token::from(span, TokenKind::Bool(s.parse().unwrap())),
        r#"null"# =>
            |_, span| Token::from(span, TokenKind::Null)
    );

    let contents = r#"
        {
            "age": 29,
            "name": "Glover Duran",
            "gender": "male",
            "company": "KINETICA",
            "phone": "+1 (953) 497-3410",
            "address": "368 Highland Place, Elbert, New Mexico, 262",
            "registered": "2016-11-11T09:22:14 -08:00",
            "latitude": -32.258953,
            "longitude": 28.625491,
            "tags": [
                "sint",
                "quis",
                "eu"
            ]
        }"#;
    let mut state = LexerState::from(contents.chars());
    while !state.eof() {
        match lexer.next_token(&mut state) {
            Ok(token) => println!("{:?}", token.kind),
            Err(msg) => {
                eprintln!("Error!");
                break;
            }
        }
    }
}
