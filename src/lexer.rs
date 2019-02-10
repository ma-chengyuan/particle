//! Lexers.
//!
//! # Example
//! See README! or `main.rs`

use std::iter::Peekable;

use rustc_hash::FxHashMap;

use crate::automatons::{BranchId, DFA, StateId};
use crate::span::{Location, Span};

/// A token handler enables custom conversions from the original strings
/// to user-defined token type enum. In this handler users can, for example:
///
/// * Decide whether to keep or discard span information
/// * Parse string to number if the token is a numeric literal
/// * Trim quotes around a string if the token is a string literal, and probably
///   deal with escape characters in it
/// * etc.
pub type TokenHandler<T> = Box<dyn Fn(&str, Span) -> T>;

/// The lexer type that parses some string and returns converted tokens of type `T`
///
/// This type is deliberately designed to not contain any "dynamic" context information,
/// the context is stored in the `LexerState<T>` class.
pub struct Lexer<T> {
    pub dfa: DFA,
    pub discarded_branch: BranchId,
    pub handlers: FxHashMap<BranchId, TokenHandler<T>>,
}

/// Holds the context
pub struct LexerState<T: Iterator<Item=char>> {
    pub chars: Peekable<T>,
    pub location: Location,
}

/// LexerState can be constructed from any character iterator
impl<T> From<T> for LexerState<T> where T: Iterator<Item=char> {
    fn from(s: T) -> Self {
        LexerState {
            chars: s.peekable(),
            location: Location::new(1, 0),
        }
    }
}

impl<T> LexerState<T> where T: Iterator<Item=char> {
    /// Whether we have reached EOF.
    pub fn eof(&mut self) -> bool {
        self.chars.peek().is_none()
    }

    /// Current character the state holds, panics with message "End of file" if already EOF.
    pub fn current(&mut self) -> &char {
        self.chars.peek().expect("End of file")
    }

    /// Move on to the next character
    pub fn next(&mut self) {
        if !self.eof() {
            let ch = self.chars.next().unwrap();
            if ch == '\n' {
                self.location.line += 1;
                self.location.col = 0;
            } else {
                self.location.col += 1;
            }
        }
    }
}

impl<T> Lexer<T> {
    pub fn next_token<I>(&self, state: &mut LexerState<I>) -> Result<T, &'static str>
        where I: Iterator<Item=char> {
        if state.eof() {
            return Err("End of file");
        }
        // Starting from the initial state of the DFA
        let mut dfa_state: StateId = self.dfa.initial_state;
        // Record we start matching the token
        let from = state.location;
        let mut to = from;
        // If matching is aborted, is the token currently accepted by any of the rules?
        let mut accepted = false;
        // Is the current char accepted by the DFA (transitions exists)
        let mut ch_accepted = true;
        // Matched token so far
        let mut token = String::new();
        // Match until no transition of a certain character can be found in the DFA
        while !state.eof() && ch_accepted {
            let ch = *state.current();
            let mut buf = [0u8; 4];
            let mut tmp_state = dfa_state;
            ch_accepted = true;
            // Encode a char to utf8 code points
            for &b in ch.encode_utf8(&mut buf).as_bytes() {
                // Try state transition from `tmp_state` with input `ch`
                if let Some(&next) = self.dfa.transitions.get(&(tmp_state, b)) {
                    tmp_state = next;
                } else {
                    // The DFA cannot accept this character
                    ch_accepted = false;
                    break;
                }
            }
            if ch_accepted {
                // Update state
                dfa_state = tmp_state;
                // Are we accepted now?
                accepted = self.dfa.final_states.contains_key(&dfa_state);
                to = state.location.clone();
                token.push(ch);
                state.next();
            }
        }
        if !accepted {
            Err("Empty input or input cannot be accepted by DFA")
        } else {
            let branch = self.dfa.final_states[&dfa_state].iter().min().unwrap();
            if let Some(handler) = self.handlers.get(branch) {
                Ok(handler(&token, Span::new(from, to)))
            } else {
                self.next_token(state) // If it is discarded?
            }
        }
    }
}

/// Macro that helps define a lexer
/// The usage is shown in README
#[macro_export] macro_rules! define_lexer {
    ($token_type:ty = $($re:expr => $handler:expr),+) => {{
        use particle::automatons::{NFA, DFA, BranchId};
        use particle::regex::compile_regex;
        use particle::lexer::{TokenHandler, Lexer};
        use rustc_hash::FxHashMap;

        let mut nfa = NFA::new();
        let mut next_branch:BranchId = 0;
        let mut handlers: FxHashMap<BranchId, TokenHandler<$token_type>> = FxHashMap::default();
        $(
            next_branch += 1;
            nfa = nfa | {
                let mut rule = compile_regex($re).unwrap();
                rule.set_branch(next_branch);
                handlers.insert(next_branch, Box::new($handler));
                rule
            };
        )*
        Lexer {
            dfa: DFA::from(nfa),
            discarded_branch: 32,
            handlers,
        }
    }};
    ($token_type:ty = discard $dis:expr, $($re:expr => $handler:expr),+) => {{
        use particle::automatons::{NFA, DFA, BranchId};
        use particle::regex::compile_regex;
        use particle::lexer::{TokenHandler, Lexer};
        use rustc_hash::FxHashMap;

        let mut nfa = NFA::new();
        let mut next_branch:BranchId = 0;
        let mut handlers: FxHashMap<BranchId, TokenHandler<$token_type>> = FxHashMap::default();
        nfa = nfa | {
            let mut discarded = compile_regex($dis).unwrap();
            discarded.set_branch(0);
            discarded
        };
        $(
            next_branch += 1;
            nfa = nfa | {
                let mut rule = compile_regex($re).unwrap();
                rule.set_branch(next_branch);
                handlers.insert(next_branch, Box::new($handler));
                rule
            };
        )*
        Lexer {
            dfa: DFA::from(nfa),
            discarded_branch: 0,
            handlers,
        }
    }};
}