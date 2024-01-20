#[derive(Copy, Clone)]
pub enum Piece {
    WhitePawn,
    WhiteKnight,
    WhiteBishop,
    WhiteRook,
    WhiteQueen,
    WhiteKing,

    BlackPawn,
    BlackKnight,
    BlackBishop,
    BlackRook,
    BlackQueen,
    BlackKing,
}

impl Piece {
    pub const LIST: [Self; 12] = [
        Self::WhitePawn,
        Self::WhiteKnight,
        Self::WhiteBishop,
        Self::WhiteRook,
        Self::WhiteQueen,
        Self::WhiteKing,
        Self::BlackPawn,
        Self::BlackKnight,
        Self::BlackBishop,
        Self::BlackRook,
        Self::BlackQueen,
        Self::BlackKing,
    ];
    pub fn to_fen_char(self) -> char {
        match self {
            Self::WhitePawn => 'P',
            Self::WhiteKnight => 'N',
            Self::WhiteBishop => 'B',
            Self::WhiteRook => 'R',
            Self::WhiteQueen => 'Q',
            Self::WhiteKing => 'K',

            Self::BlackPawn => 'p',
            Self::BlackKnight => 'n',
            Self::BlackBishop => 'b',
            Self::BlackRook => 'r',
            Self::BlackQueen => 'q',
            Self::BlackKing => 'k',
        }
    }
}

pub fn from_fen_char(character: &char) -> Result<Piece, &str> {
    match character {
        'P' => Ok(Piece::WhitePawn),
        'N' => Ok(Piece::WhiteKnight),
        'B' => Ok(Piece::WhiteBishop),
        'R' => Ok(Piece::WhiteRook),
        'Q' => Ok(Piece::WhiteQueen),
        'K' => Ok(Piece::WhiteKing),

        'p' => Ok(Piece::BlackPawn),
        'n' => Ok(Piece::BlackKnight),
        'b' => Ok(Piece::BlackBishop),
        'r' => Ok(Piece::BlackRook),
        'q' => Ok(Piece::BlackQueen),
        'k' => Ok(Piece::BlackKing),

        _ => Err("Invalid piece"),
    }
}
