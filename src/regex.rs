//! Simple regular expression parsing.
//!
//! For simplicity, only a few key features of regex are supported:
//! 1. Grouping `()`
//! 2. Bracket `[...]` and `[^...]`
//! 3. Branching `()`
//! 4. Repetition `+` and `-` (`{m, n}` are not supported)
//! 5. Optional `?`
//! 6. ~~Escape characters~~
//!
//! This module contains only one HUGE function :)
//!
//! # Example
//!
//! ```rust
//! use particle::regex::compile_regex;
//! let nfa = compile_regex(r#"[1-9][0-9]*(\.[0-9]+)?([eE](\+|-)?[1-9][0-9]*)?"#).unwrap();
//! ```

use std::cell::RefCell;
use std::collections::BTreeMap;

use crate::automatons::NFA;

/// Compile a regex into NFA, using only one function
///
/// The regex now supports Grouping (), Brackets [], Branch | and Repetition (* and +)
#[allow(clippy::cyclomatic_complexity)]
pub fn compile_regex(regex: &str) -> Result<NFA, &'static str> {
    // We use a simple two stack approach
    // This stack stores parts of regex
    let stack: RefCell<Vec<NFA>> = RefCell::new(Vec::new());
    // This stack stores regex operators like concat / repetitions etc
    let op_stack: RefCell<Vec<RegexOp>> = RefCell::new(Vec::new());
    // Are we just after an operator like ( or | (So we don't need to push extra concat operator?)
    let after_op = RefCell::new(true);

    // Regex operators
    #[derive(Debug, Eq, PartialEq)]
    enum RegexOp {
        Concat,
        ZeroOrMore,
        OneOrMore,
        Optional,
        Branch,
        Group,
    }

    // Operator precedence
    impl RegexOp {
        fn precedence(&self) -> usize {
            match self {
                RegexOp::Group => 4,
                RegexOp::ZeroOrMore => 3,
                RegexOp::OneOrMore => 3,
                RegexOp::Optional => 3,
                RegexOp::Concat => 2,
                RegexOp::Branch => 1,
            }
        }
    }

    // Pops an operator out of the stack and do the corresponding operation on the operand
    let pop_op = |op: RegexOp| {
        // println!("Popped {:?}", op);
        let mut stack = stack.borrow_mut();
        match op {
            RegexOp::Concat => {
                let second = stack.pop().unwrap();
                let first = stack.pop().unwrap();
                stack.push(first & second);
            }
            RegexOp::ZeroOrMore => {
                let inner = stack.pop().unwrap();
                stack.push(inner.zero_or_more());
            }
            RegexOp::OneOrMore => {
                let inner = stack.pop().unwrap();
                stack.push(inner.one_or_more());
            }
            RegexOp::Optional => {
                let inner = stack.pop().unwrap();
                stack.push(inner.optional());
            }
            RegexOp::Branch => {
                let first = stack.pop().unwrap();
                let second = stack.pop().unwrap();
                stack.push(first | second);
            }
            _ => {}
        }
    };

    // Pushes an operator in to the stack
    let push_op = |op: RegexOp| {
        // println!("Pushed {:?}", op);
        let mut op_stack = op_stack.borrow_mut();
        let prec = op.precedence();
        while !op_stack.is_empty() && {
            let last = op_stack.last().unwrap();
            *last != RegexOp::Group && last.precedence() >= prec
        } {
            let o = op_stack.pop().unwrap();
            pop_op(o)
        }
        op_stack.push(op);
    };

    // Pushes an NFA onto the stack
    let push_nfa = |nfa: NFA| {
        if !*after_op.borrow() {
            push_op(RegexOp::Concat);
        } else {
            after_op.replace(false);
        }
        stack.borrow_mut().push(nfa);
    };

    #[derive(Debug, Clone, Copy)]
    struct CharEndpoint(u32, i8);

    // Are we in an escape (right after '\\')
    let mut escape = false;
    // Are we in the bracket?
    let mut bracket = false;
    // Is the bracket inverted? ([^......])
    let mut bracket_inverted = false;
    // The interval endpoints in a bracket
    let mut bracket_endpoints: BTreeMap<u32, i32> = BTreeMap::new();
    // Last single char in a bracket
    let mut bracket_char: Option<u32> = None;
    // When in bracket, are we after a '-'? (So ch closes the interval)?
    let mut bracket_after_hyphen = false;
    for ch in regex.chars() {
        if !bracket {
            match ch {
                '\\' if !escape => escape = true,
                // Bracket mode
                '[' if !escape => bracket = true,
                // Parenthesis
                '(' if !escape => {
                    if !*after_op.borrow() {
                        push_op(RegexOp::Concat);
                    }
                    push_op(RegexOp::Group);
                    after_op.replace(true);
                }
                '|' if !escape => {
                    if *after_op.borrow() {
                        return Err("Consecutive |s: empty branch?");
                    }
                    push_op(RegexOp::Branch);
                    after_op.replace(true);
                }
                '*' if !escape => {
                    if *after_op.borrow() {
                        return Err("| followed immediately by *: repeat nothing?");
                    }
                    push_op(RegexOp::ZeroOrMore);
                }
                '+' if !escape => {
                    if *after_op.borrow() {
                        return Err("| followed immediately by +: repeat nothing?");
                    }
                    push_op(RegexOp::OneOrMore);
                }
                '?' if !escape => {
                    if *after_op.borrow() {
                        return Err("| followed immediately by +: making nothing optional?");
                    }
                    push_op(RegexOp::Optional);
                }
                ')' if !escape => {
                    let mut op_stack = op_stack.borrow_mut();
                    while !op_stack.is_empty() && *op_stack.last().unwrap() != RegexOp::Group {
                        let o = op_stack.pop().unwrap();
                        pop_op(o);
                    }
                    op_stack.pop().expect("() Mismatch!");
                }
                '.' if !escape => {
                    push_nfa(NFA::from(('\u{0000}', '\u{ffff}')));
                }
                _ => {
                    push_nfa(NFA::from(ch));
                    if escape {
                        escape = false;
                    }
                }
            }
        } else {
            match ch {
                '\\' if !escape => escape = true,
                '^' if !escape => bracket_inverted = true,
                '-' if !escape => {
                    if bracket_after_hyphen {
                        return Err("Consecutive - found in brackets");
                    }
                    if let Some(val) = bracket_char {
                        let endpoint = bracket_endpoints.get_mut(&(val + 1)).unwrap();
                        *endpoint += 1;
                    } else {
                        return Err("Adding '-' in brackets following nothing");
                    }
                    bracket_after_hyphen = true;
                }
                ']' if !escape => {
                    if bracket_after_hyphen {
                        return Err("Unclosed bracket interval (] after -)");
                    }

                    let mut overlay = 0;
                    let mut begin: Option<u32> = if !bracket_inverted { None } else { Some(0) };
                    let mut nfa: Option<NFA> = None;
                    let mut last = 0;

                    for (i, j) in bracket_endpoints.iter() {
                        overlay += j;
                        last = *i;
                        // If the bracket is inverted, character is in interval if overlay == 0
                        let in_interval = (overlay > 0) ^ bracket_inverted;
                        // Mark the beginning of a interval
                        if begin.is_none() && in_interval {
                            begin = Some(*i);
                        }
                        // Mark the ending of the interval
                        if begin.is_some() && !in_interval {
                            let l = std::char::from_u32(begin.unwrap()).unwrap();
                            let r = std::char::from_u32(i - 1).unwrap();
                            let n = NFA::from((l, r));
                            nfa = Some(if let Some(prev) = nfa { prev | n } else { n });
                            begin = None;
                        }
                    }

                    if overlay > 0 {
                        return Err("Unbalanced intervals!");
                    }
                    if bracket_inverted {
                        bracket_inverted = false;
                        // Don't forget to push [last, 0xffff]
                        let l = std::char::from_u32(last).unwrap();
                        let r = std::char::from_u32(0xffff).unwrap();
                        let n = NFA::from((l, r));
                        nfa = Some(if let Some(prev) = nfa { prev | n } else { n });
                    }
                    if let Some(n) = nfa {
                        push_nfa(n);
                    } else {
                        return Err("NFA not constructed for bracket!");
                    }
                    bracket_endpoints.clear();
                    bracket_char = None;
                    bracket = false;
                }
                _ => {
                    let val = ch as u32;
                    if bracket_after_hyphen {
                        bracket_after_hyphen = false;
                        bracket_char = None;
                    } else {
                        if let Some(x) = bracket_endpoints.get_mut(&val) {
                            *x += 1;
                        } else {
                            bracket_endpoints.insert(val, 1);
                        }
                        bracket_char = Some(val);
                    }
                    if let Some(x) = bracket_endpoints.get_mut(&(val + 1)) {
                        *x -= 1;
                    } else {
                        bracket_endpoints.insert(val + 1, -1);
                    }
                    if escape {
                        escape = false;
                    }
                }
            }
        }
    }
    let mut op_stack = op_stack.borrow_mut();
    while !op_stack.is_empty() {
        let o = op_stack.pop().unwrap();
        pop_op(o);
    }
    let mut stack = stack.borrow_mut();
    stack.pop().ok_or("Empty regex.")
}
