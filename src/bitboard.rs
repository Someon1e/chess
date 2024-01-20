use std::fmt;

#[derive(Copy, Clone)]
pub struct BitBoard (u64);

impl fmt::Display for BitBoard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            for file in (0..8).rev() {
                if !self.get(rank * 8 + file) {
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
    pub fn set(&mut self, square: u8) {
        self.0 |= 1 << square
    }
    pub fn get(&self, square: u8) -> bool {
        self.0 & (1 << square) != 0
    }
}