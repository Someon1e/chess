// TODO: Less repetition here

/// A piece.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Piece {
    /// A white pawn.
    WhitePawn,

    /// A white knight.
    WhiteKnight,

    /// A white bishop.
    WhiteBishop,

    /// A white rook.
    WhiteRook,

    /// A white queen.
    WhiteQueen,

    /// A white king.
    WhiteKing,

    /// A black pawn.
    BlackPawn,

    /// A black knight.
    BlackKnight,

    /// A black bishop.
    BlackBishop,

    /// A black rook.
    BlackRook,

    /// A black queen.
    BlackQueen,

    /// A black king.
    BlackKing,
}

impl Piece {
    /// All pieces.
    pub const ALL_PIECES: [Self; 12] = [
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

    /// Every white piece.
    pub const WHITE_PIECES: [Self; 6] = [
        Self::WhitePawn,
        Self::WhiteKnight,
        Self::WhiteBishop,
        Self::WhiteRook,
        Self::WhiteQueen,
        Self::WhiteKing,
    ];

    /// Every black piece.
    pub const BLACK_PIECES: [Self; 6] = [
        Self::BlackPawn,
        Self::BlackKnight,
        Self::BlackBishop,
        Self::BlackRook,
        Self::BlackQueen,
        Self::BlackKing,
    ];

    /// Converts a piece into a FEN character.
    #[must_use]
    pub const fn to_fen_char(self) -> char {
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

    /// Tries to convert a FEN character into the piece.
    ///
    /// # Errors
    ///
    /// Will return `Err` if `character` is not `'p', 'n', 'b', 'r', 'q', 'k', 'P', 'N', 'B', 'R', 'Q', or 'K'`.
    pub const fn from_fen_char(character: &char) -> Result<Self, &str> {
        match character {
            'P' => Ok(Self::WhitePawn),
            'N' => Ok(Self::WhiteKnight),
            'B' => Ok(Self::WhiteBishop),
            'R' => Ok(Self::WhiteRook),
            'Q' => Ok(Self::WhiteQueen),
            'K' => Ok(Self::WhiteKing),

            'p' => Ok(Self::BlackPawn),
            'n' => Ok(Self::BlackKnight),
            'b' => Ok(Self::BlackBishop),
            'r' => Ok(Self::BlackRook),
            'q' => Ok(Self::BlackQueen),
            'k' => Ok(Self::BlackKing),

            _ => Err("Invalid piece"),
        }
    }
}
