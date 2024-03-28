use super::bit_board::BitBoard;
use core::fmt;

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
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
    RIGHT_OFFSET,
    UP_OFFSET,
    LEFT_OFFSET,
    DOWN_OFFSET,
    UP_OFFSET + LEFT_OFFSET,
    UP_OFFSET + RIGHT_OFFSET,
    DOWN_OFFSET + LEFT_OFFSET,
    DOWN_OFFSET + RIGHT_OFFSET,
];

impl Square {
    /// Returns the square from the index
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_index(0), Square::from_notation("a1"));
    /// assert_eq!(Square::from_index(1), Square::from_notation("b1"));
    /// assert_eq!(Square::from_index(63), Square::from_notation("h8"));
    /// ```
    #[must_use]
    pub const fn from_index(index: i8) -> Self {
        Self(index)
    }

    /// Returns the square from rank number and file number
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_coords(0, 0), Square::from_notation("a1"));
    /// assert_eq!(Square::from_coords(1, 0), Square::from_notation("a2"));
    /// assert_eq!(Square::from_coords(0, 1), Square::from_notation("b1"));
    /// ```
    #[must_use]
    pub const fn from_coords(rank: i8, file: i8) -> Self {
        Self(rank * 8 + file)
    }

    /// Ranks up from white's perspective
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").up(2), Square::from_notation("a3"));
    /// ```
    #[must_use]
    pub const fn up(&self, number: i8) -> Self {
        self.offset(UP_OFFSET * number)
    }

    /// Ranks down from white's perspective
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a3").down(2), Square::from_notation("a1"));
    /// ```
    #[must_use]
    pub const fn down(&self, number: i8) -> Self {
        self.offset(DOWN_OFFSET * number)
    }

    /// Files left from white's perspective
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("c1").left(2), Square::from_notation("a1"));
    /// ```
    #[must_use]
    pub const fn left(&self, number: i8) -> Self {
        self.offset(LEFT_OFFSET * number)
    }

    /// Files right from white's perspective
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").right(2), Square::from_notation("c1"));
    /// ```
    #[must_use]
    pub const fn right(&self, number: i8) -> Self {
        self.offset(RIGHT_OFFSET * number)
    }

    #[must_use]
    pub const fn offset(&self, offset: i8) -> Self {
        Self(self.index() + offset)
    }

    /// Returns 63 >= square >= 0
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// for square_index in 0..63 {
    ///     assert!(Square::from_index(square_index).within_bounds())
    /// }
    /// assert!(!Square::from_index(-1).within_bounds());
    /// assert!(!Square::from_index(65).within_bounds());
    /// ```
    #[must_use]
    pub const fn within_bounds(&self) -> bool {
        self.index() >= 0 && self.index() < 64
    }

    #[must_use]
    pub const fn index(&self) -> i8 {
        self.0
    }
    #[must_use]
    pub const fn usize(&self) -> usize {
        self.0 as usize
    }

    /// Returns the square if viewed on flipped board
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").flip(), Square::from_notation("a8"));
    /// assert_eq!(Square::from_notation("e4").flip(), Square::from_notation("e5"));
    /// ```
    #[must_use]
    pub const fn flip(&self) -> Self {
        Self(self.index() ^ 56)
    }

    /// Returns the square if viewed on flipped board
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").flip(), Square::from_notation("a8"));
    /// assert_eq!(Square::from_notation("e4").flip(), Square::from_notation("e5"));
    /// ```
    #[must_use]
    pub const fn file(&self) -> i8 {
        self.index() & 0b111
    }

    /// Returns the rank of the square
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").rank(), 0);
    /// assert_eq!(Square::from_notation("e4").rank(), 3);
    /// ```
    #[must_use]
    pub const fn rank(&self) -> i8 {
        self.index() >> 3
    }

    /// Returns the notation of the square
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").to_notation(), "a1");
    /// ```
    #[must_use]
    pub fn to_notation(self) -> String {
        let file = self.file();
        let rank = self.rank();
        let file_char = (b'a' + file as u8) as char;
        let rank_number = rank + 1;
        format!("{file_char}{rank_number}")
    }

    /// Returns the square from the algebraic notation
    /// Rank (or row) 1 is the end of the board where white begins; black begins at rank 8.
    /// The files (or columns) are lettered from white's left to right.
    ///
    /// # Examples
    ///
    /// ```
    /// use engine::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1"), Square::from_index(0));
    /// ```
    #[must_use]
    pub fn from_notation(notation: &str) -> Self {
        let file = notation.as_bytes().first().expect("Invalid notation") - b'a';
        let rank = notation
            .chars()
            .nth(1)
            .expect("Invalid notation")
            .to_digit(10)
            .expect("Invalid notation")
            - 1;
        Self::from_coords(rank as i8, file as i8)
    }

    /// # Examples
    ///
    /// ```
    /// use engine::board::{square::Square, bit_board::BitBoard};
    ///
    /// let square = Square::from_notation("a1");
    /// assert!(square.bit_board() == BitBoard::from_square(&square));
    /// ```
    #[must_use]
    pub const fn bit_board(&self) -> BitBoard {
        BitBoard::from_square(self)
    }
}
