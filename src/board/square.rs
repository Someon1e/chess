use super::bit_board::BitBoard;
use std::fmt;

#[derive(PartialEq)]
pub struct Square(u8);

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_notation())
    }
}

impl Square {
    pub fn from_index(index: u8) -> Self {
        Self(index)
    }
    pub fn from_coords(rank: u8, file: u8) -> Self {
        Self(rank * 8 + file)
    }
    pub fn sub(&self, number: u8) -> Self {
        Self(self.0 - number)
    }
    pub fn index(&self) -> u8 {
        self.0
    }
    pub fn file(&self) -> u8 {
        self.0 % 8
    }
    pub fn rank(&self) -> u8 {
        self.0 / 8
    }
    pub fn to_notation(&self) -> String {
        let file = self.file();
        let rank = self.rank();
        let file_char = (b'a' + file) as char;
        let rank_number = (rank + 1).to_string();
        format!("{}{}", file_char, rank_number)
    }
    pub fn bitboard(&self) -> BitBoard {
        BitBoard::from_square(self)
    }
}
