//! Structs that holds locational information.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Location {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub from: Location,
    pub to: Location,
}

impl Location {
    pub fn new(line: usize, col: usize) -> Self {
        Location { line, col }
    }
}

impl Span {
    pub fn new(from: Location, to: Location) -> Self {
        Span { from, to }
    }
}
