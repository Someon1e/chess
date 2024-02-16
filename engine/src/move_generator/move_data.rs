use crate::board::{piece::Piece, square::Square};
use core::fmt;

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
    pub const ALL: [Flag; 8] = [
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

#[derive(PartialEq, Clone, Copy)]
pub struct Move {
    pub from: Square,
    pub to: Square,
    pub flag: Flag,
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "From {} to {}, Flag {:?}", self.from, self.to, self.flag)
    }
}
