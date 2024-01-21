use crate::board::{piece::Piece, square::Square};
use std::fmt;

pub struct Move {
    from: Square,
    to: Square,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.from, self.to)
    }
}

impl Move {
    pub fn new(from: Square, to: Square) -> Self {
        Self { from, to }
    }
    pub fn from(&self) -> Square {
        return self.from
    }
    pub fn to(&self) -> Square {
        return self.to
    }
}
