use crate::board::{piece::Piece, square::Square};
use std::fmt;

pub struct Move {
    piece: Piece,
    from: Square,
    to: Square,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.from, self.to)
    }
}

impl Move {
    pub fn new(piece: Piece, from: Square, to: Square) -> Self {
        Self { piece, from, to }
    }
    pub fn from(&self) -> Square {
        self.from
    }
    pub fn to(&self) -> Square {
        self.to
    }
    pub fn piece(&self) -> Piece {
        self.piece
    }
}
