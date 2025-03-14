use super::bit_board::BitBoard;
use core::fmt;

/// Squares.
#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub struct Square(i8);

impl fmt::Display for Square {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_notation())
    }
}

/// A direction.
pub type Direction = i8;

const UP_OFFSET: Direction = 8;
const DOWN_OFFSET: Direction = -8;
const LEFT_OFFSET: Direction = -1;
const RIGHT_OFFSET: Direction = 1;
/// All directions.
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
    /// Returns the square from the index.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_index(0), Square::from_notation("a1").unwrap());
    /// assert_eq!(Square::from_index(1), Square::from_notation("b1").unwrap());
    /// assert_eq!(Square::from_index(63), Square::from_notation("h8").unwrap());
    /// ```
    #[must_use]
    pub const fn from_index(index: i8) -> Self {
        Self(index)
    }

    /// Returns the square from rank number and file number.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_coords(0, 0), Square::from_notation("a1").unwrap());
    /// assert_eq!(Square::from_coords(1, 0), Square::from_notation("a2").unwrap());
    /// assert_eq!(Square::from_coords(0, 1), Square::from_notation("b1").unwrap());
    /// ```
    #[must_use]
    pub const fn from_coords(rank: i8, file: i8) -> Self {
        Self(rank * 8 + file)
    }

    /// Ranks up from white's perspective.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").unwrap().up(2), Square::from_notation("a3").unwrap());
    /// ```
    #[must_use]
    pub const fn up(&self, number: i8) -> Self {
        self.offset(UP_OFFSET * number)
    }

    /// Ranks down from white's perspective.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a3").unwrap().down(2), Square::from_notation("a1").unwrap());
    /// ```
    #[must_use]
    pub const fn down(&self, number: i8) -> Self {
        self.offset(DOWN_OFFSET * number)
    }

    /// Files left from white's perspective.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("c1").unwrap().left(2), Square::from_notation("a1").unwrap());
    /// ```
    #[must_use]
    pub const fn left(&self, number: i8) -> Self {
        self.offset(LEFT_OFFSET * number)
    }

    /// Files right from white's perspective.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").unwrap().right(2), Square::from_notation("c1").unwrap());
    /// ```
    #[must_use]
    pub const fn right(&self, number: i8) -> Self {
        self.offset(RIGHT_OFFSET * number)
    }

    /// Adds to index.
    #[must_use]
    pub const fn offset(&self, offset: i8) -> Self {
        Self(self.index() + offset)
    }

    /// Returns 63 >= square >= 0.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
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

    /// Returns the index of the square.
    #[must_use]
    pub const fn index(&self) -> i8 {
        self.0
    }

    /// Returns the index of the square as an usize.
    #[must_use]
    pub const fn usize(&self) -> usize {
        self.0 as usize
    }

    /// Returns the square if viewed on flipped board.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").unwrap().flip(), Square::from_notation("a8").unwrap());
    /// assert_eq!(Square::from_notation("e4").unwrap().flip(), Square::from_notation("e5").unwrap());
    /// ```
    #[must_use]
    pub const fn flip(&self) -> Self {
        Self(self.index() ^ 56)
    }

    /// Returns the square if viewed on flipped board.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").unwrap().flip(), Square::from_notation("a8").unwrap());
    /// assert_eq!(Square::from_notation("e4").unwrap().flip(), Square::from_notation("e5").unwrap());
    /// ```
    #[must_use]
    pub const fn file(&self) -> i8 {
        self.index() & 0b111
    }

    /// Returns the rank of the square.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").unwrap().rank(), 0);
    /// assert_eq!(Square::from_notation("e4").unwrap().rank(), 3);
    /// ```
    #[must_use]
    pub const fn rank(&self) -> i8 {
        self.index() >> 3
    }

    /// Returns the notation of the square.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").unwrap().to_notation(), "a1");
    /// ```
    #[must_use]
    pub fn to_notation(self) -> String {
        let file = self.file();
        let rank = self.rank();
        let file_char = (b'a' + file as u8) as char;
        let rank_number = rank + 1;
        format!("{file_char}{rank_number}")
    }

    /// Returns the square from the coordinate notation.
    /// Rank (or row) 1 is the end of the board where white begins; black begins at rank 8.
    /// The files (or columns) are lettered from white's left to right.
    ///
    /// # Examples
    ///
    /// ```
    /// use encrustant::board::square::Square;
    ///
    /// assert_eq!(Square::from_notation("a1").unwrap(), Square::from_index(0));
    /// ```
    #[must_use]
    pub fn from_notation(notation: &str) -> Result<Self, &str> {
        let file = match notation.as_bytes().first() {
            file @ Some(b'a'..=b'h') => file.unwrap() - b'a',
            _ => return Err("Invalid file"),
        };
        let rank = match notation.chars().nth(1) {
            rank @ Some('1'..='8') => rank.unwrap() as u8 - b'1',
            _ => return Err("Invalid rank"),
        };
        Ok(Self::from_coords(rank as i8, file as i8))
    }

    /// # Examples
    ///
    /// ```
    /// use encrustant::board::{square::Square, bit_board::BitBoard};
    ///
    /// let square = Square::from_notation("a1").unwrap();
    /// assert!(square.bit_board() == BitBoard::from_square(&square));
    /// ```
    #[must_use]
    pub const fn bit_board(&self) -> BitBoard {
        BitBoard::from_square(self)
    }
}
