//! A simple example

use std::fs;

use particle::define_lexer;
use particle::lexer::LexerState;
use particle::span::Span;

#[derive(Debug)]
enum TokenKind {
    Whitespace,
    Punctuation(String),
    Integer(i32),
    Float(f64),
    Identifier(String),
    Comment(String),
    Unknown,
}

#[derive(Debug)]
struct Token {
    span: Span,
    kind: TokenKind,
}

fn main() {
    let lexer = define_lexer!(Token =
        discard "[ \n\r\t]+",
        "[1-9][0-9]*"                                   => |s, span| Token { span,
            kind: TokenKind::Integer(s.parse().unwrap()),
        },
        "[1-9][0-9]*(\\.[0-9]+)?([eE][+\\-]?[0-9]+)?"   => |s, span| Token { span,
            kind: TokenKind::Float(s.parse().unwrap()),
        },
        "\\+|-|\\*|/|\\(|\\)"                           => |s, span| Token { span,
            kind: TokenKind::Punctuation(String::from(s)),
        },
        "[a-zA-Z][_a-zA-Z0-9]*"                         => |s, span| Token { span,
            kind: TokenKind::Identifier(String::from(s)),
        },
        "/\\*[^\\*]*\\*+([^/\\*][^\\*]*\\*+)*/"         => |s, span| Token { span ,
            kind: TokenKind::Comment(String::from(s)),
        },
        "//[^\n]*"                                      => |s, span| Token { span ,
            kind: TokenKind::Comment(String::from(s)),
        },
        "[^ \n\r\t]+"                                            => |_, span| Token { span,
            kind: TokenKind::Unknown,
        }
    );

    let contents = fs::read_to_string("benches/large_file.hpp").expect("IO Error!");
    let mut state = LexerState::from(contents.chars());
    let mut cnt = 0usize;
    while !state.eof() {
        if let Ok(token) = lexer.next_token(&mut state) {
            cnt += 1;
        } else {
            break;
        }
    }
    eprintln!("{}", cnt);
}
