use super::bit_board::BitBoard;
use std::fmt;

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
    pub fn from_index(index: i8) -> Self {
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

    pub fn index(&self) -> i8 {
        self.0
    }

    pub fn file(&self) -> i8 {
        self.index() % 8
    }
    pub fn rank(&self) -> i8 {
        self.index() / 8
    }

    pub fn to_notation(self) -> String {
        let file = self.file();
        let rank = self.rank();
        let file_char = (b'a' + file as u8) as char;
        let rank_number = (rank + 1).to_string();
        format!("{}{}", file_char, rank_number)
    }
    pub fn from_notation(notation: &str) -> Square {
        let file = notation.bytes().nth(0).expect("Invalid notation") - b'a';
        let rank = notation
            .chars().nth(1)
            .expect("Invalid notation")
            .to_digit(10)
            .expect("Invalid notation")
            - 1;
        Square::from_coords(rank as i8, file as i8)
    }
    pub fn bitboard(&self) -> BitBoard {
        BitBoard::from_square(self)
    }
}
