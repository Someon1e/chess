use std::sync::OnceLock;

use crate::board::{
    bit_board::BitBoard,
    square::{Square, DIRECTIONS},
};

pub struct PrecomputedPawnAttacks {
    pub white_pawn_attacks_at_square: [BitBoard; 64],
    pub black_pawn_attacks_at_square: [BitBoard; 64],
}

const fn min(a: i8, b: i8) -> i8 {
    if a > b {
        b
    } else {
        a
    }
}

pub const SQUARES_FROM_EDGE: [[i8; 8]; 64] = {
    let mut squares_from_edge = [[0; 8]; 64];

    let mut index = 0;
    loop {
        let square = Square::from_index(index as i8);
        let rank = square.rank();
        let file = square.file();

        squares_from_edge[index] = [
            7 - file,                // right
            7 - rank,                // up
            file,                    // left
            rank,                    // down
            min(7 - rank, file),     // up left
            min(7 - rank, 7 - file), // up right
            min(rank, file),         // down left
            min(rank, 7 - file),     // down right
        ];
        index += 1;
        if index == 64 {
            break;
        }
    }

    squares_from_edge
};

fn calculate_knight_moves_at_square() -> [BitBoard; 64] {
    let mut knight_moves_at_square = [BitBoard::EMPTY; 64];

    for (index, knight_moves) in knight_moves_at_square.iter_mut().enumerate() {
        let square = Square::from_index(index as i8);
        for knight_jump_offset in [15, 17, -17, -15, 10, -6, 6, -10] {
            let move_to = square.offset(knight_jump_offset);
            if move_to.within_bounds()
                && square
                    .file()
                    .abs_diff(move_to.file())
                    .max(square.rank().abs_diff(move_to.rank()))
                    == 2
            {
                knight_moves.set(&move_to);
            }
        }
    }
    knight_moves_at_square
}
pub fn knight_moves_at_square() -> &'static [BitBoard; 64] {
    static COMPUTATION: OnceLock<[BitBoard; 64]> = OnceLock::new();
    COMPUTATION.get_or_init(calculate_knight_moves_at_square)
}

fn calculate_king_moves_at_square() -> [BitBoard; 64] {
    let mut king_moves_at_square = [BitBoard::EMPTY; 64];
    for index in 0..64 {
        let square = Square::from_index(index as i8);

        let king_moves = &mut king_moves_at_square[index];
        for (direction_index, direction) in DIRECTIONS.iter().enumerate() {
            if SQUARES_FROM_EDGE[index][direction_index] != 0 {
                let move_to = square.offset(*direction);

                king_moves.set(&move_to);
            }
        }
    }
    king_moves_at_square
}
pub fn king_moves_at_square() -> &'static [BitBoard; 64] {
    static COMPUTATION: OnceLock<[BitBoard; 64]> = OnceLock::new();
    COMPUTATION.get_or_init(calculate_king_moves_at_square)
}

fn calculate_pawn_attacks() -> PrecomputedPawnAttacks {
    let mut white_pawn_attacks_at_square = [BitBoard::EMPTY; 64];
    let mut black_pawn_attacks_at_square = [BitBoard::EMPTY; 64];

    for index in 0..64 {
        let square = Square::from_index(index as i8);

        let white_pawn_attacks = &mut white_pawn_attacks_at_square[index];
        let black_pawn_attacks = &mut black_pawn_attacks_at_square[index];

        if BitBoard::NOT_A_FILE.get(&square) {
            if !BitBoard::RANK_8.get(&square) {
                white_pawn_attacks.set(&square.up(1).left(1));
            }
            if !BitBoard::RANK_1.get(&square) {
                black_pawn_attacks.set(&square.down(1).left(1));
            }
        }
        if BitBoard::NOT_H_FILE.get(&square) {
            if !BitBoard::RANK_8.get(&square) {
                white_pawn_attacks.set(&square.up(1).right(1));
            }
            if !BitBoard::RANK_1.get(&square) {
                black_pawn_attacks.set(&square.down(1).right(1));
            }
        }
    }

    PrecomputedPawnAttacks {
        white_pawn_attacks_at_square,
        black_pawn_attacks_at_square,
    }
}

pub fn pawn_attacks() -> &'static PrecomputedPawnAttacks {
    static COMPUTATION: OnceLock<PrecomputedPawnAttacks> = OnceLock::new();
    COMPUTATION.get_or_init(calculate_pawn_attacks)
}
