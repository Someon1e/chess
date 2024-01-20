#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Piece(usize);

// Less than 6 = white
pub const WHITE_PAWN: Piece = Piece(0);
pub const WHITE_KNIGHT: Piece = Piece(1);
pub const WHITE_BISHOP: Piece = Piece(2);
pub const WHITE_ROOK: Piece = Piece(3);
pub const WHITE_QUEEN: Piece = Piece(4);
pub const WHITE_KING: Piece = Piece(5);
// More than 6 = black
pub const BLACK_PAWN: Piece = Piece(6);
pub const BLACK_KNIGHT: Piece = Piece(7);
pub const BLACK_BISHOP: Piece = Piece(8);
pub const BLACK_ROOK: Piece = Piece(9);
pub const BLACK_QUEEN: Piece = Piece(10);
pub const BLACK_KING: Piece = Piece(11);

impl Piece {
    pub fn usize(&self) -> usize {
        self.0
    }
    pub fn to_fen_char(self) -> char {
        match self {
            WHITE_PAWN => 'P',
            WHITE_KNIGHT => 'N',
            WHITE_BISHOP => 'B',
            WHITE_ROOK => 'R',
            WHITE_QUEEN => 'Q',
            WHITE_KING => 'K',

            BLACK_PAWN => 'p',
            BLACK_KNIGHT => 'n',
            BLACK_BISHOP => 'b',
            BLACK_ROOK => 'r',
            BLACK_QUEEN => 'q',
            BLACK_KING => 'k',

            _ => unreachable!(),
        }
    }
}

impl From<usize> for Piece {
    fn from(value: usize) -> Self {
        Piece(value)
    }
}

pub fn from_fen_char(character: &char) -> Result<Piece, &str> {
    match character {
        'P' => Ok(WHITE_PAWN),
        'N' => Ok(WHITE_KNIGHT),
        'B' => Ok(WHITE_BISHOP),
        'R' => Ok(WHITE_ROOK),
        'Q' => Ok(WHITE_QUEEN),
        'K' => Ok(WHITE_KING),

        'p' => Ok(BLACK_PAWN),
        'n' => Ok(BLACK_KNIGHT),
        'b' => Ok(BLACK_BISHOP),
        'r' => Ok(BLACK_ROOK),
        'q' => Ok(BLACK_QUEEN),
        'k' => Ok(BLACK_KING),

        _ => Err("Invalid piece"),
    }
}
