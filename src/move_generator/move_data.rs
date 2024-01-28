use crate::board::{
    piece::Piece,
    square::Square,
};
use std::fmt;

pub struct Move(u16);

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "From {} to {}, Flag {:?}",
            self.from(),
            self.to(),
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
    pub fn new(from: Square, to: Square) -> Move {
        let mut data: u16 = 0;

        // Squares are 6 bits each
        data |= from.index() as u16;
        data |= (to.index() as u16) << 6;

        Self(data)
    }

    pub fn from(&self) -> Square {
        Square::from_index((self.0 & 0b111111) as i8)
    }
    pub fn to(&self) -> Square {
        Square::from_index(((self.0 >> 6) & 0b111111) as i8)
    }

    pub fn with_flag(from: Square, to: Square, flag: Flag) -> Self {
        let mut data = Self::new(from, to).0;
        data |= (flag as u16) << 12;
        Self(data)
    }

    pub fn flag(&self) -> &Flag {
        &Flag::ALL[((self.0 >> 12) & 0b1111) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::Move;
    use crate::{
        board::square::Square,
        move_generator::move_data::Flag,
    };

    // TODO: add more test cases
    const TEST_MOVES: [(Square, Square, Flag); 2] = [
        (
            Square::from_coords(2, 2),
            Square::from_coords(3, 2),
            Flag::None,
        ),
        (
            Square::from_coords(5, 5),
            Square::from_coords(7, 7),
            Flag::None,
        ),
    ];
    #[test]
    fn move_encoded_correctly() {
        for test_move in TEST_MOVES {
            let (from, to, flag) = test_move;
            let encoded = Move::with_flag(from, to, flag);
            assert_eq!(encoded.from(), from);
            assert_eq!(encoded.to(), to);
            assert_eq!(*encoded.flag(), flag);
        }
    }
}
