//! Automaton stuffs.
//!
//! DFAs and NFAs.

extern crate multimap;
extern crate rustc_hash;
extern crate utf8_ranges;

use indexmap::IndexSet;
use multimap::MultiMap;
use rustc_hash::{FxHashMap, FxHashSet};
use std::cmp;
use std::collections::BTreeSet;
use std::fmt::*;
use std::ops::{BitAnd, BitOr};

/// Type of transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Transition {
    Input(u8),
    Epsilon,
}

pub type StateId = usize;
pub type BranchId = usize;
pub type StateSet = BTreeSet<StateId>;
// Default branch number for final states whose branch number is not explicitly specified
const DEFAULT_BRANCH_ID: BranchId = 0;

/// Nondeterministic Finite Automaton.
///
/// The inside implementation of the automaton is based on `u8`,
/// therefore a character transition may be represented as **multiple edges** in
/// the NFA depending on its UTF-8 encoding.
#[derive(Clone)]
pub struct NFA {
    pub initial_state: StateId,
    pub final_states: FxHashMap<StateId, BranchId>,
    pub transitions: MultiMap<(StateId, Transition), StateId>,
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
        ret.final_states.insert(last, DEFAULT_BRANCH_ID);
        ret
    }
}

impl From<char> for NFA {
    /// Constructs the NFA from a single char.
    fn from(ch: char) -> Self {
        let mut ret = NFA::new();
        let mut buf = [0; 4];
        let mut last = 0;

        for &b in ch.encode_utf8(&mut buf).as_bytes() {
            ret.transitions
                .insert((last, Transition::Input(b)), last + 1);
            last += 1;
        }
        ret.final_states.insert(last, DEFAULT_BRANCH_ID);
        ret
    }
}

impl From<(char, char)> for NFA {
    /// Constructs the NFA from a char interval.
    fn from(interval: (char, char)) -> Self {
        use utf8_ranges::Utf8Sequences;

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
            ret.final_states.insert(last, DEFAULT_BRANCH_ID);
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
            .extend(dfa.final_states.keys().map(|&x| (x, DEFAULT_BRANCH_ID)));
        ret.transitions.extend(
            dfa.transitions
                .iter()
                .map(|(&(from, b), &to)| ((from, Transition::Input(b)), to)),
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
        // The final states of the result NFA is the (biased) final states of the second NFA.
        ret.final_states = rhs
            .final_states
            .iter()
            .map(|(x, &br)| (x + bias, br))
            .collect();
        // Connecting edges from the first NFA to the second NFA.
        ret.transitions.extend(
            old_final_states
                .iter()
                .map(|(&x, _)| ((x, Transition::Epsilon), rhs.initial_state + bias)),
        );
        // Add biased transition edges.
        ret.transitions.extend(
            rhs.transitions
                .iter_all()
                .flat_map(|((from, trans), to_vec)| {
                    to_vec
                        .iter()
                        .map(move |to| ((from + bias, *trans), to + bias))
                }),
        );
        ret
    }
}

impl BitOr for NFA {
    type Output = NFA;
    fn bitor(self, rhs: NFA) -> NFA {
        let mut ret = self;
        let bias = ret.max_state_id() + 1;
        // Final states from either NFAs are final states of the result NFA.
        ret.final_states
            .extend(rhs.final_states.iter().map(|(x, &br)| (x + bias, br)));
        // Add biased transition edges.
        ret.transitions.extend(
            rhs.transitions
                .iter_all()
                .flat_map(|((from, trans), to_vec)| {
                    to_vec
                        .iter()
                        .map(move |to| ((from + bias, *trans), to + bias))
                }),
        );
        let old_initial = ret.initial_state;
        // New initial state
        ret.initial_state = bias + rhs.max_state_id() + 1;
        // Connected the new initial state to two original initial states.
        ret.transitions
            .insert((ret.initial_state, Transition::Epsilon), old_initial);
        ret.transitions.insert(
            (ret.initial_state, Transition::Epsilon),
            rhs.initial_state + bias,
        );
        ret
    }
}

impl Default for NFA {
    fn default() -> Self {
        NFA::new()
    }
}

impl NFA {
    /// Constructs an empty NFA.
    pub fn new() -> NFA {
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
            .map(|(&(from, _), &to)| cmp::max(from, to))
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
    pub fn transition_set(&self, from: &StateSet, input: u8) -> StateSet {
        let mut ret = StateSet::new();
        for &u in from {
            if let Some(vs) = self.transitions.get_vec(&(u, Transition::Input(input))) {
                for &v in vs {
                    ret.append(&mut self.epsilon_closure(v));
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
            .map(|(&x, _)| ((x, Transition::Epsilon), ret.initial_state))
            .collect();
        ret.transitions.extend(new_transisions);
        ret.final_states.clear();
        ret.final_states
            .insert(ret.initial_state, DEFAULT_BRANCH_ID);
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
        let new_final = self.max_state_id() + 1;
        ret.final_states.clear();
        ret.final_states.insert(new_final, DEFAULT_BRANCH_ID);
        ret.transitions.extend(
            self.final_states
                .iter()
                .map(|(&x, _)| ((x, Transition::Epsilon), new_final)),
        );
        ret.transitions
            .insert((self.initial_state, Transition::Epsilon), new_final);
        ret
    }
}

/// Deterministic Finite Automaton.
#[derive(Clone)]
pub struct DFA {
    pub initial_state: StateId,
    pub final_states: FxHashMap<StateId, FxHashSet<BranchId>>,
    pub transitions: FxHashMap<(StateId, u8), BranchId>,
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

        // Record character transitions coming out of each state
        for &(u, tr) in nfa.transitions.keys() {
            if let Transition::Input(ch) = tr {
                edges_out.insert(u, ch);
            }
        }

        // The new initial state
        states.insert(initial_state, 0);
        while !stack.is_empty() {
            let state_now = stack.pop().unwrap();
            let idx = states[&state_now];
            // Character transitions coming out from all state in the state_now
            let mut edges_out_now: FxHashSet<u8> = FxHashSet::default();
            let mut branches = FxHashSet::default();
            for u in &state_now {
                if let Some(&br) = nfa.final_states.get(u) {
                    branches.insert(br);
                }
                if let Some(chs) = edges_out.get_vec(u) {
                    edges_out_now.extend(chs);
                }
            }
            // Mark the new DFA state as final if it contains orginal NFA final state
            if !branches.is_empty() {
                ret.final_states.insert(idx, branches);
            }
            for ch in edges_out_now {
                let to = nfa.transition_set(&state_now, ch);
                match states.get(&to) {
                    Some(&to_idx) => {
                        ret.transitions.insert((idx, ch), to_idx);
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

impl Default for DFA {
    fn default() -> Self {
        DFA::new()
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

    fn max_state_id(&self) -> StateId {
        self.transitions
            .iter()
            .map(|(&(from, _), &to)| cmp::max(from, to))
            .max()
            .unwrap_or(0)
    }

    pub fn minimize(self) -> DFA {
        let reachable_from: MultiMap<StateId, (u8, StateId)> = self
            .transitions
            .iter()
            .map(|(&(from, tr), &to)| (to, (tr, from)))
            .collect();
        let all_states: StateSet = (0..=self.max_state_id()).collect();
        let mut partitions: IndexSet<StateSet> = IndexSet::new();
        let mut distinguishers: IndexSet<StateSet> = IndexSet::new();
        let final_states: StateSet = all_states
            .iter()
            .filter(|x| self.final_states.contains_key(x))
            .cloned()
            .collect();
        partitions.insert(final_states.clone());
        partitions.insert(all_states.difference(&final_states).cloned().collect());
        distinguishers.insert(all_states.difference(&final_states).cloned().collect());
        distinguishers.insert(final_states);

        while !distinguishers.is_empty() {
            let a = distinguishers.pop().unwrap();
            let c: MultiMap<u8, StateId> = a
                .iter()
                .filter_map(|x| reachable_from.get_vec(x).map(|vec| vec.iter().cloned()))
                .flatten()
                .collect();
            for (_, x) in c.iter_all() {
                let x: StateSet = x.iter().cloned().collect();
                let mut new_partitions: IndexSet<StateSet> = IndexSet::new();
                while !partitions.is_empty() {
                    let y = partitions.pop().unwrap();
                    let intersection: StateSet = y.intersection(&x).cloned().collect();
                    let difference: StateSet = y.difference(&x).cloned().collect();
                    if !intersection.is_empty() && !difference.is_empty() {
                        // println!("Found: {:?} {:?} {:?}", y, intersection, difference);
                        if distinguishers.contains(&y) {
                            distinguishers.remove(&y);
                            distinguishers.insert(intersection.clone());
                            distinguishers.insert(difference.clone());
                        } else {
                            distinguishers.insert(if intersection.len() <= difference.len() {
                                intersection.clone()
                            } else {
                                difference.clone()
                            });
                        }
                        new_partitions.insert(intersection);
                        new_partitions.insert(difference);
                    } else {
                        new_partitions.insert(y);
                    }
                }
                partitions = new_partitions;
            }
        }
        let labeled: FxHashMap<StateSet, StateId> = partitions.iter().cloned().zip(0..).collect();
        let map: FxHashMap<StateId, StateId> = partitions
            .iter()
            .flat_map(|p| {
                let id = labeled[p];
                p.iter().map(move |&x| (x, id))
            })
            .collect();
        DFA {
            initial_state: map[&self.initial_state],
            final_states: partitions
                .iter()
                .filter_map(|p| {
                    let union: FxHashSet<BranchId> = p
                        .iter()
                        .filter_map(|x| self.final_states.get(x).map(|s| s.iter().cloned()))
                        .flatten()
                        .collect();
                    if union.is_empty() {
                        None
                    } else {
                        Some((labeled[p], union))
                    }
                })
                .collect(),
            transitions: self
                .transitions
                .iter()
                .map(|((from, tr), to)| ((map[from], *tr), map[to]))
                .collect(),
        }
    }
}

/// Minimizes a vector of `u8` ot its string description
/// For example, [1, 2, 3, 4, 5, 9, 11, 12, 13] -> "[1-5], 9, [11,13]"
fn vec_to_string(mut vec: Vec<u8>) -> String {
    if vec.is_empty() {
        return String::new();
    }
    let mut ret = String::new();
    let mut begin: Option<u8> = None;
    let mut last: Option<u8> = None;
    vec.sort();
    vec.dedup();
    for v in vec {
        if begin.is_none() {
            begin = Some(v);
        }
        if let Some(l) = last {
            if v > l + 1 {
                // If we have last then we definitely have begin, unwrap safely
                if l == begin.unwrap() {
                    ret.push_str(&format!("{}, ", l));
                } else {
                    ret.push_str(&format!("[{},{}],", begin.unwrap(), l));
                }
                begin = Some(v);
            }
        }
        last = Some(v);
    }
    if last.unwrap() == begin.unwrap() {
        ret.push_str(&format!("{}", last.unwrap()));
    } else {
        ret.push_str(&format!("[{},{}]", begin.unwrap(), last.unwrap()));
    }
    ret
}

impl Debug for NFA {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let transitions: MultiMap<StateId, (Transition, StateId)> = self
            .transitions
            .iter_all()
            .flat_map(|(&(from, tr), to_vec)| to_vec.iter().map(move |&to| (from, (tr, to))))
            .collect();
        write!(f, "digraph NFA {{")?;
        if f.alternate() {
            writeln!(f)?;
        }
        for v in 0..=self.max_state_id() {
            if f.alternate() {
                write!(f, "\t")?;
            }
            write!(
                f,
                "N{0}[label=\"{0}\", shape={1}];",
                v,
                if self.final_states.contains_key(&v) {
                    "doublecircle"
                } else {
                    "circle"
                }
            )?;
            if f.alternate() {
                writeln!(f)?;
            }
        }
        for u in 0..=self.max_state_id() {
            if let Some(vec) = transitions.get_vec(&u) {
                let transitions_here: MultiMap<StateId, Transition> =
                    vec.iter().map(|&(tr, to)| (to, tr)).collect();
                for (v, tr) in transitions_here.iter_all() {
                    let char_transitions: Vec<u8> = tr
                        .iter()
                        .filter_map(|&tr| {
                            if let Transition::Input(ch) = tr {
                                Some(ch)
                            } else {
                                None
                            }
                        })
                        .collect();
                    if f.alternate() {
                        write!(f, "\t")?;
                    }
                    if !char_transitions.is_empty() {
                        write!(
                            f,
                            "N{} -> N{}[label=\"{}\"];",
                            u,
                            v,
                            vec_to_string(char_transitions)
                        )?;
                    } else {
                        write!(f, "N{} -> N{}[label=\"ε\"];", u, v)?;
                    }
                    if f.alternate() {
                        writeln!(f)?;
                    }
                }
            }
        }
        write!(f, "}}")
    }
}

impl Debug for DFA {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let transitions: MultiMap<StateId, (u8, StateId)> = self
            .transitions
            .iter()
            .map(|(&(from, tr), &to)| (from, (tr, to)))
            .collect();
        write!(f, "digraph DFA {{")?;
        if f.alternate() {
            writeln!(f)?;
        }
        for v in 0..=self.max_state_id() {
            if f.alternate() {
                write!(f, "\t")?;
            }
            write!(
                f,
                "N{0}[label=\"{0}\", shape={1}];",
                v,
                if self.final_states.contains_key(&v) {
                    "doublecircle"
                } else {
                    "circle"
                }
            )?;
            if f.alternate() {
                writeln!(f)?;
            }
        }
        for u in 0..=self.max_state_id() {
            if let Some(vec) = transitions.get_vec(&u) {
                let transitions_here: MultiMap<StateId, u8> =
                    vec.iter().map(|&(tr, to)| (to, tr)).collect();
                for v in transitions_here.keys() {
                    let inputs = transitions_here.get_vec(v).unwrap().clone();
                    if f.alternate() {
                        write!(f, "\t")?;
                    }
                    write!(f, "N{} -> N{}[label=\"{}\"];", u, v, vec_to_string(inputs))?;
                    if f.alternate() {
                        writeln!(f)?;
                    }
                }
            }
        }
        write!(f, "}}")
    }
}
