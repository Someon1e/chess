use crate::board::{
    piece::{Piece, ALL_PIECES},
    square::Square,
};
use std::fmt;

pub struct Move(u32);

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "From {} to {}, Captured {:?}, Flag {:?}",
            self.from(),
            self.to(),
            self.captured(),
            self.flag()
        )
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Flag {
    None,

    QueenPromotion,
    RookPromotion,
    BishopPromotion,
    KnightPromotion,

    EnPassant,

    PawnTwoUp,

    Castle,
}
impl Flag {
    const ALL: [Flag; 8] = [
        Self::None,
        Self::QueenPromotion,
        Self::RookPromotion,
        Self::BishopPromotion,
        Self::KnightPromotion,
        Self::EnPassant,
        Self::PawnTwoUp,
        Self::Castle,
    ];
    pub const PROMOTIONS: [Flag; 4] = [
        Self::QueenPromotion,
        Self::RookPromotion,
        Self::BishopPromotion,
        Self::KnightPromotion,
    ];

    pub fn get_promotion_piece(&self, white: bool) -> Option<Piece> {
        match self {
            Self::QueenPromotion => Some({
                if white {
                    Piece::WhiteQueen
                } else {
                    Piece::BlackQueen
                }
            }),
            Self::RookPromotion => Some({
                if white {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                }
            }),
            Self::BishopPromotion => Some({
                if white {
                    Piece::WhiteBishop
                } else {
                    Piece::BlackBishop
                }
            }),
            Self::KnightPromotion => Some({
                if white {
                    Piece::WhiteKnight
                } else {
                    Piece::BlackKnight
                }
            }),
            _ => None,
        }
    }
}

impl Move {
    pub fn new(from: Square, to: Square, captured: Option<Piece>) -> Move {
        let mut data: u32 = 0;

        // Squares are 6 bits each
        data |= from.index() as u32;
        data |= (to.index() as u32) << 6;

        if let Some(captured) = captured {
            data |= (captured as u32) << 12;
        } else {
            data |= 0b1111 << 12
        }

        Self(data)
    }

    pub fn from(&self) -> Square {
        Square::from_index((self.0 & 0b111111) as i8)
    }
    pub fn to(&self) -> Square {
        Square::from_index(((self.0 >> 6) & 0b111111) as i8)
    }
    pub fn captured(&self) -> Option<Piece> {
        let encoded = (self.0 >> 12) & 0b1111;
        if encoded == 0b1111 {
            None
        } else {
            Some(ALL_PIECES[encoded as usize])
        }
    }

    pub fn with_flag(from: Square, to: Square, captured: Option<Piece>, flag: Flag) -> Self {
        let mut data = Self::new(from, to, captured).0;
        data |= (flag as u32) << 16;
        Self(data)
    }

    pub fn flag(&self) -> &Flag {
        &Flag::ALL[((self.0 >> 16) & 0b1111) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::Move;
    use crate::{
        board::{piece::Piece, square::Square},
        move_generator::move_data::Flag,
    };

    // TODO: add more test cases
    const TEST_MOVES: [(Square, Square, Option<Piece>, Flag); 2] = [
        (
            Square::from_coords(2, 2),
            Square::from_coords(3, 2),
            Some(Piece::WhiteKing),
            Flag::None,
        ),
        (
            Square::from_coords(5, 5),
            Square::from_coords(7, 7),
            None,
            Flag::None,
        ),
    ];
    #[test]
    fn move_encoded_correctly() {
        for test_move in TEST_MOVES {
            let (from, to, captured, flag) = test_move;
            let encoded = Move::with_flag(from, to, captured, flag);
            assert_eq!(encoded.from(), from);
            assert_eq!(encoded.to(), to);
            assert_eq!(encoded.captured(), captured);
            assert_eq!(*encoded.flag(), flag);
        }
    }
}
