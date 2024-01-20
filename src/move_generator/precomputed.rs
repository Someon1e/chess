use crate::board::square::Square;

pub struct PrecomputedData {
    pub white_pawn_attacks_at_square: Vec<Vec<Square>>,
    pub black_pawn_attacks_at_square: Vec<Vec<Square>>,
}

impl PrecomputedData {
    pub fn compute() -> Self {
        let mut white_pawn_attacks_at_square = vec![vec![]; 64];
        let mut black_pawn_attacks_at_square = vec![vec![]; 64];

        for index in 0..64 {
            let square = Square::from_index(index as u8);
            let rank = square.rank();
            let file = square.file();

            let white_pawn_attacks = &mut white_pawn_attacks_at_square[index];
            let black_pawn_attacks = &mut black_pawn_attacks_at_square[index];

            if file > 0 {
                if rank < 7 {
                    white_pawn_attacks.push(square.up(1).left(1));
                }
                if rank > 0 {
                    black_pawn_attacks.push(square.down(1).left(1));
                }
            }
            if file < 7 {
                if rank < 7 {
                    white_pawn_attacks.push(square.up(1).right(1));
                }
                if rank > 0 {
                    black_pawn_attacks.push(square.down(1).right(1));
                }
            }
        }

        Self {
            white_pawn_attacks_at_square,
            black_pawn_attacks_at_square,
        }
    }
}
