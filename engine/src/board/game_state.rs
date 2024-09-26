use crate::board::square::Square;

use super::{piece::Piece, zobrist::Zobrist};

macro_rules! define_castling_rights {
    ($getter:ident, $setter:ident, $unsetter:ident, $side:expr, $offset:expr) => {
        #[doc = " Returns `true` if there is a right to"]
        #[doc = $side]
        #[doc = " castling, otherwise `false`."]
        #[must_use]
        pub const fn $getter(&self) -> bool {
            self.0 & (1 << $offset) != 0
        }

        #[doc = "Sets "]
        #[doc = $side]
        #[doc = " castling right."]
        pub fn $setter(&mut self) {
            self.0 |= 1 << $offset;
        }

        #[doc = "Unsets "]
        #[doc = $side]
        #[doc = " castling right."]
        pub fn $unsetter(&mut self) {
            self.0 &= !(1 << $offset);
        }
    };
}

/// Encodes castling rights.
#[derive(Copy, Clone)]
pub struct CastlingRights(u8);
impl CastlingRights {
    const WHITE_KING_SIDE_OFFSET: u8 = 0;
    const WHITE_QUEEN_SIDE_OFFSET: u8 = 1;
    const BLACK_KING_SIDE_OFFSET: u8 = 2;
    const BLACK_QUEEN_SIDE_OFFSET: u8 = 3;

    define_castling_rights!(
        get_white_king_side,
        set_white_king_side,
        unset_white_king_side,
        "white kingside",
        Self::WHITE_KING_SIDE_OFFSET
    );

    define_castling_rights!(
        get_white_queen_side,
        set_white_queen_side,
        unset_white_queen_side,
        "white queenside",
        Self::WHITE_QUEEN_SIDE_OFFSET
    );
    define_castling_rights!(
        get_black_king_side,
        set_black_king_side,
        unset_black_king_side,
        "black kingside",
        Self::BLACK_KING_SIDE_OFFSET
    );
    define_castling_rights!(
        get_black_queen_side,
        set_black_queen_side,
        unset_black_queen_side,
        "black queenside",
        Self::BLACK_QUEEN_SIDE_OFFSET
    );

    /// Creates castling rights.
    #[must_use]
    #[allow(clippy::fn_params_excessive_bools)]
    pub fn new(
        white_can_castle_king_side: bool,
        white_can_castle_queen_side: bool,

        black_can_castle_king_side: bool,
        black_can_castle_queen_side: bool,
    ) -> Self {
        let mut data = 0;
        data |= u8::from(white_can_castle_king_side) << Self::WHITE_KING_SIDE_OFFSET;
        data |= u8::from(white_can_castle_queen_side) << Self::WHITE_QUEEN_SIDE_OFFSET;
        data |= u8::from(black_can_castle_king_side) << Self::BLACK_KING_SIDE_OFFSET;
        data |= u8::from(black_can_castle_queen_side) << Self::BLACK_QUEEN_SIDE_OFFSET;
        Self(data)
    }

    /// Parses castling rights from the section of a FEN string.
    #[must_use]
    pub fn from_fen_section(castling_rights: &str) -> Self {
        if castling_rights == "-" {
            Self::new(false, false, false, false)
        } else {
            Self::new(
                castling_rights.contains('K'),
                castling_rights.contains('Q'),
                castling_rights.contains('k'),
                castling_rights.contains('q'),
            )
        }
    }

    /// Returns whether no sides can castle.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        self.0 == 0
    }

    /// Returns the u8 internal representation.
    #[must_use]
    pub const fn internal_value(&self) -> u8 {
        self.0
    }
}

/// Game state.
#[derive(Copy, Clone)]
pub struct GameState {
    /// The square which can be captured by en passant.
    pub en_passant_square: Option<Square>,

    /// Castling rights.
    pub castling_rights: CastlingRights,

    /// Half move clock.
    pub half_move_clock: u32,

    /// The last captured piece.
    pub captured: Option<Piece>,

    /// Zobrist key.
    pub zobrist_key: Zobrist,
}
