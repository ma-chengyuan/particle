pub mod automatons;
pub mod lexer;
pub mod regex;

use crate::automatons::{DFA, NFA};

fn main() {
    let int_part = NFA::from(('1', '9')) & NFA::from(('0', '9')).zero_or_more();
    let float_part = NFA::from('.') & NFA::from(('0', '9')).one_or_more();
    let exp_part = (NFA::from('e') | NFA::from('E'))
        & (NFA::from('+') | NFA::from('-')).optional()
        & NFA::from(('1', '9'))
        & (NFA::from(('0', '9')).zero_or_more());
    let nfa = int_part & float_part.optional() & exp_part.optional();
    println!("{:#?}", nfa);
    println!("{:#?}", DFA::from(nfa));

    let nfa = regex::compile_regex(r#"[1-9][0-9]*(\.[0-9]+)?([eE](\+|-)?[1-9][0-9]*)?"#);
    println!("{:#?}", nfa);
    println!("{:#?}", DFA::from(nfa));
}
