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
            "from {} to {}, capturing {:?}, is castle {}, is en passant {}, is pawn two up {}, promotion {:?}",
            self.from(),
            self.to(),
            self.captured(),
            self.is_castle(),
            self.is_en_passant(),
            self.is_pawn_two_up(),
            self.get_promotion()
        )
    }
}

#[derive(Debug, PartialEq)]
pub enum Promotion {
    NoPromotion,
    QueenPromotion,
    RookPromotion,
    BishopPromotion,
    KnightPromotion,
}
impl Promotion {
    pub fn to_piece(&self, white: bool) -> Piece {
        match self {
            Promotion::QueenPromotion => {
                if white {
                    Piece::WhiteQueen
                } else {
                    Piece::BlackQueen
                }
            }
            Promotion::RookPromotion => {
                if white {
                    Piece::WhiteRook
                } else {
                    Piece::BlackRook
                }
            }
            Promotion::BishopPromotion => {
                if white {
                    Piece::WhiteBishop
                } else {
                    Piece::BlackBishop
                }
            }
            Promotion::KnightPromotion => {
                if white {
                    Piece::WhiteKnight
                } else {
                    Piece::BlackKnight
                }
            }
            Promotion::NoPromotion => unreachable!(),
        }
    }
}

pub const PROMOTION_ENUM: [Promotion; 5] = [
    Promotion::NoPromotion,
    Promotion::QueenPromotion,
    Promotion::RookPromotion,
    Promotion::BishopPromotion,
    Promotion::KnightPromotion,
];
pub const ALL_PROMOTIONS: [Promotion; 4] = [
    Promotion::QueenPromotion,
    Promotion::RookPromotion,
    Promotion::BishopPromotion,
    Promotion::KnightPromotion,
];

impl Move {
    pub fn new(piece: Piece, from: Square, to: Square, captured: Option<Piece>) -> Move {
        let mut data: u32 = 0;

        // Squares are 6 bits each
        data |= from.index() as u32;
        data |= (to.index() as u32) << 6;

        // Pieces are 4 bits

        data |= (piece as u32) << 12;

        if let Some(captured) = captured {
            data |= (captured as u32) << 16;
        } else {
            data |= 0b1111 << 16
        }

        Self(data)
    }
    pub fn en_passant(piece: Piece, from: Square, to: Square, captured: Piece) -> Self {
        let mut data = Self::new(piece, from, to, Some(captured)).0;
        data |= 1 << 20;
        Self(data)
    }
    pub fn pawn_two_up(piece: Piece, from: Square, to: Square) -> Self {
        let mut data = Self::new(piece, from, to, None).0;
        data |= 1 << 21;
        Self(data)
    }
    pub fn castle(piece: Piece, from: Square, to: Square) -> Self {
        let mut data = Self::new(piece, from, to, None).0;
        data |= 1 << 22;
        Self(data)
    }

    pub fn promotion(
        piece: Piece,
        from: Square,
        to: Square,
        captured: Option<Piece>,
        promotion_type: Promotion,
    ) -> Self {
        let mut data = Self::new(piece, from, to, captured).0;
        data |= (promotion_type as u32) << 23;
        Self(data)
    }

    pub fn from(&self) -> Square {
        Square::from_index((self.0 & 0b111111) as i8)
    }
    pub fn to(&self) -> Square {
        Square::from_index(((self.0 >> 6) & 0b111111) as i8)
    }
    pub fn piece(&self) -> Piece {
        ALL_PIECES[((self.0 >> 12) & 0b1111) as usize]
    }
    pub fn captured(&self) -> Option<Piece> {
        let encoded = (self.0 >> 16) & 0b1111;
        if encoded == 0b1111 {
            None
        } else {
            Some(ALL_PIECES[encoded as usize])
        }
    }
    pub fn is_en_passant(&self) -> bool {
        (self.0 >> 20) & 0b1 == 1
    }
    pub fn is_pawn_two_up(&self) -> bool {
        (self.0 >> 21) & 0b1 == 1
    }
    pub fn is_castle(&self) -> bool {
        (self.0 >> 22) & 0b1 == 1
    }
    pub fn get_promotion(&self) -> &Promotion {
        &PROMOTION_ENUM[((self.0 >> 23) & 0b111) as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::Move;
    use crate::{
        board::square::Square,
        move_generator::{move_data::Promotion, Piece},
    };

    // TODO: add more test cases
    const TEST_MOVES: [(Piece, Square, Square); 2] = [
        (
            Piece::WhitePawn,
            Square::from_coords(2, 2),
            Square::from_coords(3, 2),
        ),
        (
            Piece::BlackBishop,
            Square::from_coords(5, 5),
            Square::from_coords(7, 7),
        ),
    ];
    #[test]
    fn move_encoded_correctly() {
        for test_move in TEST_MOVES {
            let (piece, from, to) = test_move;
            let encoded = Move::new(piece, from, to, None);
            assert_eq!(encoded.piece(), piece);
            assert_eq!(encoded.from(), from);
            assert_eq!(encoded.to(), to);
            assert_eq!(encoded.captured(), None);
            assert_eq!(encoded.is_en_passant(), false);
            assert_eq!(encoded.is_pawn_two_up(), false);
            assert_eq!(*encoded.get_promotion(), Promotion::NoPromotion);
        }
    }
}
