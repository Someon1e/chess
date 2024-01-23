use crate::board::square::Square;

#[derive(Copy, Clone)]
pub struct CastlingRights(u8);
impl CastlingRights {
    const WHITE_CAN_CASTLE_KING_SIDE: u8 = 0b0001;
    const WHITE_CAN_CASTLE_QUEEN_SIDE: u8 = 0b0010;

    const BLACK_CAN_CASTLE_KING_SIDE: u8 = 0b0100;
    const BLACK_CAN_CASTLE_QUEEN_SIDE: u8 = 0b1000;

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
    pub fn none(&self) -> bool {
        self.0 == 0
    }

    pub fn get_white_king_side(&self) -> bool {
        self.0 & Self::WHITE_CAN_CASTLE_KING_SIDE == Self::WHITE_CAN_CASTLE_KING_SIDE
    }
    pub fn get_white_queen_side(&self) -> bool {
        self.0 & Self::WHITE_CAN_CASTLE_QUEEN_SIDE == Self::WHITE_CAN_CASTLE_QUEEN_SIDE
    }
    pub fn get_black_king_side(&self) -> bool {
        self.0 & Self::BLACK_CAN_CASTLE_KING_SIDE == Self::BLACK_CAN_CASTLE_KING_SIDE
    }
    pub fn get_black_queen_side(&self) -> bool {
        self.0 & Self::BLACK_CAN_CASTLE_QUEEN_SIDE == Self::BLACK_CAN_CASTLE_QUEEN_SIDE
    }

    pub fn set_white_king_side(&mut self) {
        self.0 |= Self::WHITE_CAN_CASTLE_KING_SIDE
    }
    pub fn set_white_queen_side(&mut self) {
        self.0 |= Self::WHITE_CAN_CASTLE_QUEEN_SIDE
    }
    pub fn set_black_king_side(&mut self) {
        self.0 |= Self::BLACK_CAN_CASTLE_KING_SIDE
    }
    pub fn set_black_queen_side(&mut self) {
        self.0 |= Self::BLACK_CAN_CASTLE_QUEEN_SIDE
    }
}

#[derive(Copy, Clone)]
pub struct GameState {
    pub en_passant_square: Option<Square>,

    pub castling_rights: CastlingRights,

    pub half_move_clock: u64,
    pub full_move_counter: u64,
}
