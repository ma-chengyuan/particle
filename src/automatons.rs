//! Automaton stuffs.
//!
//! DFAs and NFAs.

extern crate multimap;
extern crate rustc_hash;
extern crate utf8_ranges;

use multimap::MultiMap;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp;
use std::collections::BTreeSet;
use std::ops::{BitAnd, BitOr};
use utf8_ranges::Utf8Sequences;

/// Type of transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Transition {
    Input(u8),
    Epsilon,
}

pub type StateId = usize;
pub type BranchId = usize;
type StateSet = BTreeSet<StateId>;
const DEFAULT_BRANCH_NUMBER: BranchId = 0;

/// Nondeterministic Finite Automaton.
///
/// The inside implementation of the automaton is based on `u8`,
/// therefore a character transition may be represented as **multiple edges** in
/// the NFA depending on its UTF-8 encoding.
///
/// # Example
/// Consider the regex to match C Strings (`\"([^\\\"]|\\.)*\"`):
/// ```
/// let quote = NFA::from('"');
/// let non_escape = NFA::from('"').or(&NFA::from('\\')).not();
/// let escape = NFA::from('\\').and(&NFA::from(('\u{0000}', '\u{ffff}')));
/// let string = quote
///     .and(&escape.or(&non_escape).zero_or_more())
///     .and(&quote);
/// ```
#[derive(Debug, Clone)]
pub struct NFA {
    initial_state: StateId,
    final_states: FxHashMap<StateId, BranchId>,
    transitions: MultiMap<(StateId, Transition), StateId>,
}

impl From<&str> for NFA {
    /// Constructs the NFA from a string.
    fn from(s: &str) -> Self {
        let mut ret = NFA::new();
        let mut last = 0;
        for b in s.as_bytes() {
            ret.transitions
                .insert((last, Transition::Input(*b)), last + 1);
            last += 1;
        }
        ret.final_states.insert(last, DEFAULT_BRANCH_NUMBER);
        ret
    }
}

impl From<char> for NFA {
    /// Constructs the NFA from a single char.
    fn from(ch: char) -> Self {
        let mut ret = NFA::new();
        let mut buf = [0; 4];
        let mut last = 0;

        for b in ch.encode_utf8(&mut buf).as_bytes() {
            ret.transitions
                .insert((last, Transition::Input(*b)), last + 1);
            last += 1;
        }
        ret.final_states.insert(last, DEFAULT_BRANCH_NUMBER);
        ret
    }
}

impl From<(char, char)> for NFA {
    /// Constructs the NFA from a range of chars.
    fn from(interval: (char, char)) -> Self {
        let mut ret = NFA::new();
        let mut next_id = 1;
        for seq in Utf8Sequences::new(interval.0, interval.1) {
            let mut last = 0;
            for r in seq.into_iter() {
                for b in r.start..=r.end {
                    ret.transitions
                        .insert((last, Transition::Input(b)), next_id);
                }
                last = next_id;
                next_id += 1;
            }
            ret.final_states.insert(last, DEFAULT_BRANCH_NUMBER);
        }
        ret
    }
}

impl From<DFA> for NFA {
    /// Converts a DFA back to NFA, all branch informations WILL BE LOST.
    fn from(dfa: DFA) -> Self {
        let mut ret = NFA::new();
        ret.initial_state = dfa.initial_state;
        ret.final_states
            .extend(dfa.final_states.keys().map(|x| (*x, DEFAULT_BRANCH_NUMBER)));
        ret.transitions.extend(
            dfa.transitions
                .iter()
                .map(|((from, b), to)| ((*from, Transition::Input(*b)), *to)),
        );
        ret
    }
}

impl BitAnd for NFA {
    type Output = NFA;
    fn bitand(self, rhs: NFA) -> NFA {
        let mut ret = self;
        let bias = ret.max_state_id() + 1;
        let old_final_states = ret.final_states;
        ret.final_states = rhs
            .final_states
            .iter()
            .map(|(x, br)| (x + bias, *br))
            .collect();
        ret.transitions.extend(
            old_final_states
                .iter()
                .map(|(x, _)| ((*x, Transition::Epsilon), rhs.initial_state + bias)),
        );
        ret.transitions.extend(
            rhs.transitions
                .iter()
                .map(|((from, trans), to)| ((from + bias, *trans), to + bias)),
        );
        ret
    }
}

impl BitOr for NFA {
    type Output = NFA;
    fn bitor(self, rhs: NFA) -> NFA {
        let mut ret = self;
        let bias = ret.max_state_id() + 1;
        ret.final_states
            .extend(rhs.final_states.iter().map(|(x, br)| (x + bias, *br)));
        ret.transitions.extend(
            rhs.transitions
                .iter()
                .map(|((from, trans), to)| ((from + bias, *trans), to + bias)),
        );
        let old_initial = ret.initial_state;
        ret.initial_state = bias + rhs.max_state_id() + 1;
        ret.transitions
            .insert((ret.initial_state, Transition::Epsilon), old_initial);
        ret.transitions.insert(
            (ret.initial_state, Transition::Epsilon),
            rhs.initial_state + bias,
        );
        ret
    }
}

impl NFA {
    /// Constructs an empty NFA.
    fn new() -> NFA {
        NFA {
            initial_state: 0,
            final_states: FxHashMap::default(),
            transitions: MultiMap::new(),
        }
    }

    /// Max state id of the NFA, used for biasing when merging one NFA into the other.
    fn max_state_id(&self) -> StateId {
        self.transitions
            .iter()
            .map(|((from, _), to)| cmp::max(*from, *to))
            .max()
            .unwrap_or(0)
    }

    /// Calculates the epsilon closure of a state.
    fn epsilon_closure(&self, s: StateId) -> StateSet {
        let mut ret = StateSet::new();
        let mut stack = vec![s];
        ret.insert(s);
        while !stack.is_empty() {
            let u = stack.pop().unwrap();
            if let Some(vs) = self.transitions.get_vec(&(u, Transition::Epsilon)) {
                for v in vs {
                    if !ret.contains(v) {
                        ret.insert(*v);
                        stack.push(*v);
                    }
                }
            }
        }
        ret
    }

    /// Calculates the transition set of a stateset with given input.
    fn transition_set(&self, from: &StateSet, input: u8) -> StateSet {
        let mut ret = StateSet::new();
        for u in from {
            if let Some(vs) = self.transitions.get_vec(&(*u, Transition::Input(input))) {
                for v in vs {
                    ret.append(&mut self.epsilon_closure(*v));
                }
            }
        }
        ret
    }

    /// Sets the branch id for all final states currently in the NFA.
    ///
    /// This should only be called right before you convert the NFA into DFA,
    /// without any further operation after the call except conversion, or the
    /// branch information will be a total mess!
    pub fn set_branch(&mut self, branch: BranchId) {
        for br in self.final_states.values_mut() {
            *br = branch;
        }
    }

    /// Repeats `self` by >=0 times (`*` in regex).
    pub fn zero_or_more(self) -> NFA {
        let mut ret = self;
        let new_transisions: FxHashMap<(StateId, Transition), StateId> = ret
            .final_states
            .iter()
            .map(|(x, _)| ((*x, Transition::Epsilon), ret.initial_state))
            .collect();
        ret.transitions.extend(new_transisions);
        ret.final_states.clear();
        ret.final_states
            .insert(ret.initial_state, DEFAULT_BRANCH_NUMBER);
        ret
    }

    /// Repeats `self` by >=1 times (`+` in regex).
    pub fn one_or_more(self) -> NFA {
        let temp = self.clone().zero_or_more();
        self & temp
    }

    /// Makes `self` optional (0/1 times).
    pub fn optional(&self) -> NFA {
        let mut ret = self.clone();
        ret.transitions.extend(
            self.final_states
                .iter()
                .map(|(x, _)| ((self.initial_state, Transition::Epsilon), *x)),
        );
        ret
    }
}

/// Deterministic Finite Automaton.
#[derive(Debug, Clone)]
pub struct DFA {
    initial_state: StateId,
    final_states: FxHashMap<StateId, FxHashSet<BranchId>>,
    transitions: FxHashMap<(StateId, u8), BranchId>,
}

impl From<NFA> for DFA {
    /// Constructs the DFA from a NFA using subset construction.
    fn from(nfa: NFA) -> Self {
        let mut ret = DFA::new();
        let mut states = FxHashMap::default();
        let initial_state = nfa.epsilon_closure(nfa.initial_state);
        let mut stack = vec![initial_state.clone()];
        let mut next_idx = 1;
        let mut edges_out = MultiMap::new();

        for (u, tr) in nfa.transitions.keys() {
            if let Transition::Input(ch) = tr {
                edges_out.insert(*u, *ch);
            }
        }

        states.insert(initial_state, 0);
        while !stack.is_empty() {
            let state_now = stack.pop().unwrap();
            let idx = states[&state_now];
            let mut edges_out_here: FxHashSet<u8> = FxHashSet::default();
            let mut branches = FxHashSet::default();
            for u in &state_now {
                if let Some(br) = nfa.final_states.get(u) {
                    branches.insert(*br);
                }
                if let Some(chs) = edges_out.get_vec(u) {
                    edges_out_here.extend(chs);
                }
            }
            if !branches.is_empty() {
                ret.final_states.insert(idx, branches);
            }
            for ch in edges_out_here {
                let to = nfa.transition_set(&state_now, ch);
                match states.get(&to) {
                    Some(to_idx) => {
                        ret.transitions.insert((idx, ch), *to_idx);
                    }
                    None => {
                        stack.push(to.clone());
                        states.insert(to, next_idx);
                        ret.transitions.insert((idx, ch), next_idx);
                        next_idx += 1;
                    }
                }
            }
        }
        ret
    }
}

impl DFA {
    fn new() -> DFA {
        DFA {
            initial_state: 0,
            final_states: FxHashMap::default(),
            transitions: FxHashMap::default(),
        }
    }
}