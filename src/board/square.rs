use super::bit_board::BitBoard;
use core::fmt;

#[derive(PartialEq, Copy, Clone, Debug)]
pub struct Square(i8);

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_notation())
    }
}

pub type Direction = i8;

const UP_OFFSET: Direction = 8;
const DOWN_OFFSET: Direction = -8;
const LEFT_OFFSET: Direction = -1;
const RIGHT_OFFSET: Direction = 1;
pub const DIRECTIONS: [Direction; 8] = [
    UP_OFFSET,
    DOWN_OFFSET,
    LEFT_OFFSET,
    RIGHT_OFFSET,
    UP_OFFSET + LEFT_OFFSET,
    DOWN_OFFSET + RIGHT_OFFSET,
    UP_OFFSET + RIGHT_OFFSET,
    DOWN_OFFSET + LEFT_OFFSET,
];

impl Square {
    pub const fn from_index(index: i8) -> Self {
        Self(index)
    }
    pub const fn from_coords(rank: i8, file: i8) -> Self {
        Self(rank * 8 + file)
    }

    pub fn up(&self, number: i8) -> Self {
        self.offset(UP_OFFSET * number)
    }
    pub fn down(&self, number: i8) -> Self {
        self.offset(DOWN_OFFSET * number)
    }

    pub fn left(&self, number: i8) -> Self {
        self.offset(LEFT_OFFSET * number)
    }
    pub fn right(&self, number: i8) -> Self {
        self.offset(RIGHT_OFFSET * number)
    }

    pub fn within_bounds(&self) -> bool {
        self.index() >= 0 && self.index() < 64
    }

    pub fn offset(&self, offset: i8) -> Self {
        Self(self.index() + offset)
    }

    pub const fn index(&self) -> i8 {
        self.0
    }

    pub fn flip(&self) -> Self {
        Self(self.index() ^ 56)
    }

    pub const fn file(&self) -> i8 {
        self.index() & 0b111
    }
    pub const fn rank(&self) -> i8 {
        self.index() >> 3
    }

    pub fn to_notation(self) -> String {
        let file = self.file();
        let rank = self.rank();
        let file_char = (b'a' + file as u8) as char;
        let rank_number = rank + 1;
        format!("{}{}", file_char, rank_number)
    }

    pub fn from_notation(notation: &str) -> Square {
        let file = notation.as_bytes().first().expect("Invalid notation") - b'a';
        let rank = notation
            .chars()
            .nth(1)
            .expect("Invalid notation")
            .to_digit(10)
            .expect("Invalid notation")
            - 1;
        Square::from_coords(rank as i8, file as i8)
    }
    pub fn bit_board(&self) -> BitBoard {
        BitBoard::from_square(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::board::{bit_board::BitBoard, square::Square};

    #[test]
    fn test_coordinates() {
        let a1 = Square::from_coords(0, 0);
        assert!(a1.to_notation() == "a1");
        let b1 = a1.right(1);
        assert!(b1.to_notation() == "b1");
        let b2 = b1.up(1);
        assert!(b2.to_notation() == "b2");
        let also_b2 = Square::from_index(b2.index());
        assert!(also_b2.to_notation() == "b2");
        assert!(also_b2.index() == 9);

        let bit_board = also_b2.bit_board();

        let mut same_bit_board = BitBoard::EMPTY;
        same_bit_board.set(&also_b2);

        assert!(bit_board.to_string() == same_bit_board.to_string())
    }
}
