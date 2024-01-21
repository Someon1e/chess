use crate::board::square::{Square, DIRECTION_OFFSETS};

pub struct PrecomputedData {
    pub white_pawn_attacks_at_square: Vec<Vec<Square>>,
    pub black_pawn_attacks_at_square: Vec<Vec<Square>>,
    pub king_moves_at_square: Vec<Vec<Square>>,
}

impl PrecomputedData {
    pub fn compute() -> Self {
        let mut white_pawn_attacks_at_square = vec![vec![]; 64];
        let mut black_pawn_attacks_at_square = vec![vec![]; 64];
        let mut king_moves_at_square = vec![vec![]; 64];

        for index in 0..64 {
            let square = Square::from_index(index as i8);
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

            let king_moves = &mut king_moves_at_square[index];
            for direction in DIRECTION_OFFSETS {
                let move_to = square.offset(direction);
                if move_to.within_bounds() {
                    if (file - move_to.file())
                        .abs()
                        .max((rank - move_to.rank()).abs())
                        == 1
                    {
                        king_moves.push(move_to);
                    }
                }
            }
        }

        Self {
            white_pawn_attacks_at_square,
            black_pawn_attacks_at_square,
            king_moves_at_square,
        }
    }
}
