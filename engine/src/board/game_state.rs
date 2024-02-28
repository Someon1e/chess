use crate::board::square::Square;

use super::{piece::Piece, zobrist::Zobrist};

macro_rules! define_castling_rights {
    ($getter:ident, $setter:ident, $unsetter:ident, $offset:expr) => {
        #[must_use]
        pub fn $getter(&self) -> bool {
            self.0 & (1 << $offset) != 0
        }

        pub fn $setter(&mut self) {
            self.0 |= 1 << $offset;
        }

        pub fn $unsetter(&mut self) {
            self.0 &= !(1 << $offset);
        }
    };
}
#[derive(Copy, Clone)]
pub struct CastlingRights(u8);
impl CastlingRights {
    define_castling_rights!(
        get_white_king_side,
        set_white_king_side,
        unset_white_king_side,
        0
    );
    define_castling_rights!(
        get_white_queen_side,
        set_white_queen_side,
        unset_white_queen_side,
        1
    );
    define_castling_rights!(
        get_black_king_side,
        set_black_king_side,
        unset_black_king_side,
        2
    );
    define_castling_rights!(
        get_black_queen_side,
        set_black_queen_side,
        unset_black_queen_side,
        3
    );

    #[must_use]
    pub fn new(
        white_can_castle_king_side: bool,
        white_can_castle_queen_side: bool,

        black_can_castle_king_side: bool,
        black_can_castle_queen_side: bool,
    ) -> Self {
        let mut data = Self(0);
        if white_can_castle_king_side {
            data.set_white_king_side()
        };
        if white_can_castle_queen_side {
            data.set_white_queen_side()
        };
        if black_can_castle_king_side {
            data.set_black_king_side()
        };
        if black_can_castle_queen_side {
            data.set_black_queen_side()
        };
        data
    }
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
    #[must_use]
    pub fn none(&self) -> bool {
        self.0 == 0
    }
    #[must_use]
    pub fn internal_value(&self) -> u8 {
        self.0
    }
}

#[derive(Copy, Clone)]
pub struct GameState {
    pub en_passant_square: Option<Square>,

    pub castling_rights: CastlingRights,

    pub half_move_clock: u64,
    pub full_move_counter: u64,
    pub captured: Option<Piece>,

    pub zobrist_key: Zobrist,
}
