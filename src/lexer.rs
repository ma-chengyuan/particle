extern crate rustc_hash;
use crate::automatons::{BranchId, StateId, DFA};
use rustc_hash::FxHashMap;

type TokenCallback<T> = Fn(&str) -> T;

struct Lexer<'a, T> {
    dfa: DFA,
    state: StateId,
    input: &'a str,
    callbacks: FxHashMap<BranchId, T>,
}
