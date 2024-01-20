use std::fmt;
use crate::board::square::Square;

pub struct Move {
    from: Square,
    to: Square
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.from, self.to)
    }
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Self {from, to}
    }
}