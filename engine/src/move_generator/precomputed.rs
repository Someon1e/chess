use std::sync::OnceLock;

use crate::board::{bit_board::BitBoard, square::Square};

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

pub const KING_MOVES_AT_SQUARE: [BitBoard; 64] = {
    let mut king_moves_at_square = [BitBoard::EMPTY; 64];
    let mut index = 0;
    loop {
        let square_bit = 1 << index;

        let left = (square_bit & 0x7F7F_7F7F_7F7F_7F7F) << 1;
        let right = (square_bit & 0xFEFE_FEFE_FEFE_FEFE) >> 1;
        let left_right = left | right;
        let attacks = left_right | (left_right | square_bit) >> 8 | (left_right | square_bit) << 8;
        king_moves_at_square[index] = BitBoard::new(attacks);

        index += 1;
        if index == 64 {
            break;
        }
    }
    king_moves_at_square
};
