//! A simple example

use particle::define_lexer;
use particle::lexer::LexerState;
use particle::span::Span;

/// Four kinds of tokens
#[derive(Debug)]
enum TokenKind {
    Punctuation(String),
    Integer(i32),
    Float(f64),
    Identifier(String),
}

/// A Token object with locational information (span)
#[derive(Debug)]
struct Token {
    span: Span,
    kind: TokenKind,
}

fn main() {
    // Define our lexer
    let lexer = define_lexer!(Token = // `Token` denotes the type of token this lexer is going to return
        // Discard white spaces
        discard "[ \n\r\t]+",
        // Integers
        // The expression after => is a function that takes the token string as well as the span
        // and returns the result (of type specified above, or Token in this case).
        "[1-9][0-9]*"                                   => |s, span| Token { span,
            kind: TokenKind::Integer(s.parse().unwrap()),
        },
        // Floats with exponents
        // Have you noticed that the regex above also matches integers, which may lead to ambiguity?
        // Such ambiguity is solved by preferring rules that are defined first
        // So you should somehow put identifier rules at last...
        "[1-9][0-9]*(\\.[0-9]+)?([eE][+\\-]?[0-9]+)?"   => |s, span| Token { span,
            kind: TokenKind::Float(s.parse().unwrap()),
        },
        // Punctuations
        "\\+|-|\\*|/|\\(|\\)"                           => |s, span| Token { span,
            kind: TokenKind::Punctuation(String::from(s)),
        },
        // Identifiers
        "[a-zA-Z][_a-zA-Z0-9]*"                         => |s, span| Token { span,
            kind: TokenKind::Identifier(String::from(s)),
        }
    );
    // Notice that when writing regular expressions down we did not use raw string literals, which is a common
    // practice, this is because I am simply too lazy to handle all types of escape characters -- just use that
    // of rust! Then the only thing you may feel uncomfortable is writing \\s!

    // We use a lexer state to store the context information
    // A LexerState can be constructed from any char iterators, the simplest being calling .chars() of a string
    let mut state = LexerState::from(
        "(412 + 321.654) / 768.432 * 34e-1 - sin(30)".chars()
    );
    // Proceed until EOF
    while !state.eof() {
        match lexer.next_token(&mut state) {
            Ok(token) => eprintln!("{:?}", token.kind),
            Err(msg) => {
                eprintln!("Error at {:?}: {}", state.location, msg);
                break;
            }
        }
    }
}
