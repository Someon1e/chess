use super::square::Square;
use std::fmt;

#[derive(Copy, Clone)]
pub struct BitBoard(u64);

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in (0..8).rev() {
                if !self.get(&Square::from_coords(rank, file)) {
                    write!(f, "0")?
                } else {
                    write!(f, "1")?
                }
                if file != 0 {
                    write!(f, " ")?
                }
            }
            writeln!(f)?
        }
        Ok(())
    }
}

impl BitBoard {
    pub fn empty() -> Self {
        BitBoard(0)
    }
    pub fn from_square(square: &Square) -> Self {
        BitBoard(1 << square.index())
    }
    pub fn set(&mut self, square: &Square) {
        self.0 |= square.bitboard().0
    }
    pub fn get(&self, square: &Square) -> bool {
        self.0 & square.bitboard().0 != 0
    }
}
