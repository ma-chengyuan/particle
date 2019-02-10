use particle::define_lexer;
use particle::lexer::LexerState;
use particle::span::Span;

#[derive(Debug)]
enum TokenKind {
    Punctuation(String),
    Integer(i32),
    Float(f64),
    Identifier(String),
}

#[derive(Debug)]
struct Token {
    span: Span,
    kind: TokenKind,
}

fn main() {
    let lexer = define_lexer!(Token =
        discard "[ \n\r\t]",
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
        }
    );

    let mut state = LexerState::from(
        "(412 + 321.654) / 768.432 * 34e-1 - sin(30)".chars()
    );
    while !state.eof() {
        match lexer.next_token(&mut state) {
            Ok(token) => eprintln!("{:?}", token),
            Err(msg) => {
                eprintln!("Error at {:?}: {}", state.location, msg);
                break;
            }
        }
    }
}
