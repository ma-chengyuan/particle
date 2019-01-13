pub mod automatons;
pub mod lexer;
pub mod regex;

use crate::automatons::{DFA, NFA};

fn main() {
    let nfa = regex::compile_regex(r#"\"([^\\\"]|\\.)*\""#);
    println!("{:#?}", nfa);
    let dfa = DFA::from(nfa);
    println!("{:#?}", dfa);
    let dfa = dfa.minimize();
    println!("{:#?}", dfa);
    println!("{}", dfa.initial_state);
}
