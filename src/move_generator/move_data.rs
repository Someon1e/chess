use crate::board::{piece::Piece, square::Square};
use std::fmt;

pub struct Move {
    piece: Piece,
    from: Square,
    to: Square,
    capture: Option<Piece>
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.piece.to_fen_char(), self.from, self.to)
    }
}

impl Move {
    pub fn new(piece: Piece, from: Square, to: Square, capture: Option<Piece>) -> Self {
        Self { piece, from, to, capture }
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
    pub fn capture(&self) -> Option<Piece> {
        self.capture
    }
}
