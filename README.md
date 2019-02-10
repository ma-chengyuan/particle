# Particle - A practice in writing parser gens

A hobby project as I learn compiler theory as well as rust.

## Planned Features

Currently there exists two major form of parser gens:

1. Parser gens like Yacc(or Bison), ANTLR and javacc needs developers to write an external syntax description file which will then be converted into source code by these generators. These generators are indeed fancy and powerful, but its relatively hard to customize.
2. Parser combinators are good, intuitive and simple, they do not require developer to write external files, instead, syntax rules are encoded in the source code directly. That being said, since parser combinators are "combinators", each of the combinator cannot really have a big picture of the source being parsed, therefore error handling can be hard.

Particle aims to be the hybrid of two forms listed above, to be exact, it is designed to be:

1. No external description files. Just put your syntax rules in the source code.
2. Fast. This is why I choose Rust as the language instead of Java or Go.
3. One-stop. Particle will cover both lexers and parsers. If possible, auto bindings to AST elements will be available too.
   
## Currently implemented Features

1. Parse from simple regular expressions (no captures) to NFAs.
2. Subset construction algorithm to convert NFAs to DFAs.
3. Hopcroft algorithm for NFA minimization.
4. Lexer construction

## Example

This annotated example shows a simple lexer parsing expression:

```rust
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
    // of rust! Then the only thing you may feel uncomfortable is writing //s!

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
```

Running this example yields:
```
Punctuation("(")
Integer(412)
Punctuation("+")
Float(321.654)
Punctuation(")")
Punctuation("/")
Float(768.432)
Punctuation("*")
Float(3.4)
Punctuation("-")
Identifier("sin")
Punctuation("(")
Integer(30)
Punctuation(")")
```

If we change the input string to an erroneous one like `"(412 + 321.65乱4) / 768.43入2 * 34e-1 - sin(30)"`:
```
Punctuation("(")
Integer(412)
Punctuation("+")
Float(321.654)
Error at Location { line: 1, col: 13 }: Empty input or input cannot be accepted by DFA
```

As you may see particle is different from other lexer gens in that it is not that "battery included"...
You still need to define your own token types, and write functions that do conversions.
But it offers great flexibility -- the lexer is only responsible for identifying where the token is,
its up to you to decide how to deal with them.

The `define_lexer` macro is still implemented in a somehow dumb way, and you can see some boilerplate code
in after => s, this should be improved after rust allows partial hygiene bending in macros... 

## TODO List
1. ~~Wrapping DFA into Lexer~~
2. ~~Better interface to construct NFA~~
3. Optimizing DFA performance
4. ~~Char class in NFA implementation(Too Lazy)~~
5. LL Parser gen
6. ~~DFA Minimization~~