# Particle - A practice in writing parser gens

A hobby project as I learn compiler theory as well as rust.

一时摸一时爽，一直摸一直爽！

## Planned Features

Currently there exists two major form of parser gens:

1. Parser gens like Yacc(or Bison), ANTLR and JavaCC needs developers to write an external syntax description file which will then be converted into source code by these generators. These generators are indeed fancy and powerful, but its relatively hard to customize.
2. Parser combinators are good, intuitive and simple, they do not require developer to write external files, instead, syntax rules are encoded in the source code directly. That being said, since parser combinators are "combinators", each of the combinator cannot really have a big picture of the source being parsed, therefore error handling can be hard.

Particle aims to be the hybrid of two forms listed above, to be exact, it is designed to be:

1. No external description files. Just put your syntax rules in the source code.
2. Fast. This is why I choose Rust as the language instead of Java or Go.
3. One-stop. Particle will cover both lexers and parsers. If possible, auto bindings to AST elements will be available too.
   
## Currently implemented Features

1. Parse from simple regular expressions (no captures) to NFAs.
2. Subset construction algorithm to convert NFAs to DFAs.
3. Hopcroft algorithm for DFA minimization.
4. Lexer construction 

## Example

This example shows a simple lexer parsing simple expressions:

```rust
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
```

Running this example for some random json yields:
```
Punctuation("{")
Str("\"age\"")
Punctuation(":")
Number(29.0)
Punctuation(",")
Str("\"name\"")
Punctuation(":")
Str("\"Glover Duran\"")
Punctuation(",")
Str("\"gender\"")
Punctuation(":")
Str("\"male\"")
Punctuation(",")
Str("\"company\"")
Punctuation(":")
Str("\"KINETICA\"")
Punctuation(",")
Str("\"phone\"")
Punctuation(":")
Str("\"+1 (953) 497-3410\"")
Punctuation(",")
Str("\"address\"")
...
```

As you may see particle is different from other lexer gens in that it is not that "battery included"...
You still need to define your own token types, and write functions that do conversions.
But it offers great flexibility -- the lexer is only responsible for identifying where the token is,
its up to you to decide how to deal with them.

The `define_lexer` macro is still implemented in a somehow dumb way, and you can see some boilerplate code
in after => s, this should be improved after rust allows partial hygiene bending in macros... 

## Performance
I did a rough benchmark on the speed of the lexer using [a json file](/benches/large_json.json).
The benchmark code can be found under `/benches`.

The benchmark is run on a i7-8550U and 16GB RAM.

The header file is 155kB and contains 13401 tokens according to the definition in the benchmark. The lexer is able
identify them all in an average of 8.041ms according to criterion. 
Therefore the estimated efficiency should be around **1,666,583 tokens/s** or **18.9 MB/s**. Which should be sufficient in most cases.

The benchmark here is still inaccurate, further improvement is needed.

## TODO List
1. ~~Wrapping DFA into Lexer~~
2. ~~Better interface to construct NFA~~
3. Optimizing DFA performance
4. ~~Char class in NFA implementation(Too Lazy)~~
5. LL Parser gen (or probably LR?)
6. ~~DFA Minimization~~
7. Wait for rust's macro (proc_macro and macro 2.0) to stabilize and expand DFA into real code in compilation