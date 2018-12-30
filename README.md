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

### NFA and DFA

Example: Constructing a DFA matching C-Style strings with escapes (In regex: `\"([^\\\"]|\\.)*\"`):

```rust
let quote = NFA::from('"');
let non_escape = NFA::from('"').or(&NFA::from('\\')).not();
let escape = NFA::from('\\').and(&NFA::from(('\u{0000}', '\u{ffff}')));
let string = quote
    .and(&escape.or(&non_escape).zero_or_more())
    .and(&quote);
let dfa = DFA::from(string)
```
Still quite a lot of boilerplate codes, more work to do! 

## TODO List
1. Wrapping DFA into lexer
2. Better interface to contruct NFA
3. Optimizing DFA performance
4. Char class in NFA implementation
5. LL Parser gen