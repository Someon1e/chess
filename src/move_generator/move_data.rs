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
            "from {} to {}, capturing {:?}, is castle {}, is en passant {}, is pawn two up {}",
            self.from(),
            self.to(),
            self.captured(),
            self.is_castle(),
            self.is_en_passant(),
            self.is_pawn_two_up()
        )
    }
}

impl Move {
    pub fn capture() {
        // TODO
    }
    pub fn new(
        piece: Piece,
        from: Square,
        to: Square,
        captured: Option<Piece>,

        is_en_passant: bool,
        is_pawn_two_up: bool,
        is_castle: bool,
    ) -> Self {
        let mut data: u32 = 0;

        // Squares are 6 bits each
        data |= from.index() as u32;
        data |= (to.index() as u32) << 6;

        // Moving piece is 4 bits
        data |= (piece as u32) << 12;
        // Capturing piece is 4 bits
        if let Some(captured) = captured {
            data |= (captured as u32) << 16;
        } else {
            data |= 0b1111 << 16
        }

        // 1 bit to store if it's an en_passant
        if is_en_passant {
            data |= 1 << 20
        }
        // 1 bit to store if it's an pawn going 2 squares up
        if is_pawn_two_up {
            data |= 1 << 21
        }
        // 1 bit to store if it's a castle
        if is_castle {
            data |= 1 << 22
        }
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
}

#[cfg(test)]
mod tests {
    use super::Move;
    use crate::{board::square::Square, move_generator::Piece};

    const TEST_MOVES: [(Piece, Square, Square, Option<Piece>, bool, bool, bool); 2] = [
        (
            Piece::WhitePawn,
            Square::from_coords(2, 2),
            Square::from_coords(3, 2),
            None,
            false,
            false,
            false,
        ),
        (
            Piece::BlackBishop,
            Square::from_coords(5, 5),
            Square::from_coords(7, 7),
            None,
            false,
            false,
            false,
        ),
    ];
    #[test]
    fn move_encoded_correctly() {
        for test_move in TEST_MOVES {
            let (piece, from, to, captured, is_en_passant, is_pawn_two_up, is_castle) = test_move;
            let encoded = Move::new(
                piece,
                from,
                to,
                captured,
                is_en_passant,
                is_pawn_two_up,
                is_castle,
            );
            assert_eq!(encoded.piece(), piece);
            assert_eq!(encoded.from(), from);
            assert_eq!(encoded.to(), to);
            assert_eq!(encoded.captured(), captured);
            assert_eq!(encoded.is_en_passant(), is_en_passant);
            assert_eq!(encoded.is_pawn_two_up(), is_pawn_two_up);
        }
    }
}
