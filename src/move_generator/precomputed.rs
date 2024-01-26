use crate::board::{
    bit_board::BitBoard,
    square::{Square, DIRECTIONS},
};

pub struct PrecomputedData {
    pub white_pawn_attacks_at_square: Vec<Vec<Square>>,
    pub black_pawn_attacks_at_square: Vec<Vec<Square>>,
    pub knight_moves_at_square: Vec<Vec<Square>>,
    pub king_moves_at_square: [BitBoard; 64],
    pub squares_from_edge: [[i8; 8]; 64],
}

impl PrecomputedData {
    pub fn compute() -> Self {
        let mut white_pawn_attacks_at_square = vec![vec![]; 64];
        let mut black_pawn_attacks_at_square = vec![vec![]; 64];
        let mut knight_moves_at_square = vec![vec![]; 64];
        let mut king_moves_at_square = [BitBoard::empty(); 64];
        let mut squares_from_edge = [[0; 8]; 64];

        for index in 0..64 {
            let square = Square::from_index(index as i8);
            let rank = square.rank();
            let file = square.file();

            squares_from_edge[index] = [
                7 - rank,
                rank,
                file,
                7 - file,
                (7 - rank).min(file),
                rank.min(7 - file),
                (7 - rank).min(7 - file),
                rank.min(file),
            ];

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

            let knight_moves = &mut knight_moves_at_square[index];
            for knight_jump_offset in [15, 17, -17, -15, 10, -6, 6, -10] {
                let move_to = square.offset(knight_jump_offset);
                if move_to.within_bounds()
                    && (square.file() - move_to.file())
                        .abs()
                        .max((square.rank() - move_to.rank()).abs())
                        == 2
                {
                    knight_moves.push(move_to)
                }
            }

            let king_moves = &mut king_moves_at_square[index];
            for direction in DIRECTIONS {
                let move_to = square.offset(direction);
                if move_to.within_bounds()
                    && (file - move_to.file())
                        .abs()
                        .max((rank - move_to.rank()).abs())
                        == 1
                {
                    king_moves.set(&move_to);
                }
            }
        }

        Self {
            white_pawn_attacks_at_square,
            black_pawn_attacks_at_square,
            knight_moves_at_square,
            king_moves_at_square,
            squares_from_edge,
        }
    }
}
