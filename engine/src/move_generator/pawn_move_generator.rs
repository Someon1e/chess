use crate::{
    board::{bit_board::BitBoard, square::Square},
    consume_bit_board,
};

use super::{
    move_data::{Flag, Move},
    slider_lookup::{get_rook_moves, relevant_rook_blockers},
    MoveGenerator,
};

pub struct PawnAttacks {
    pub white_pawn_attacks_at_square: [BitBoard; 64],
    pub black_pawn_attacks_at_square: [BitBoard; 64],
}

pub const PAWN_ATTACKS: PawnAttacks = {
    let mut white_pawn_attacks_at_square = [BitBoard::EMPTY; 64];
    let mut black_pawn_attacks_at_square = [BitBoard::EMPTY; 64];

    let mut index = 0;
    loop {
        let square = Square::from_index(index as i8);
        let square_bit = 1 << index;

        let mut white_pawn_attacks = 0;
        let mut black_pawn_attacks = 0;

        if 0xFEFE_FEFE_FEFE_FEFE_u64 & square_bit != 0 {
            // not a file
            if (!0xFF00_0000_0000_0000_u64) & square_bit != 0 {
                // not 8th rank
                white_pawn_attacks |= 1 << square.up(1).left(1).index();
            }
            if (!0xFF) & square_bit != 0 {
                // not 1st rank
                black_pawn_attacks |= 1 << square.down(1).left(1).index();
            }
        }
        if 0x7F7F_7F7F_7F7F_7F7F_u64 & square_bit != 0 {
            // not h file
            if (!0xFF00_0000_0000_0000_u64) & square_bit != 0 {
                // not 8th rank
                white_pawn_attacks |= 1 << square.up(1).right(1).index();
            }
            if (!0xFF) & square_bit != 0 {
                // not 1st rank
                black_pawn_attacks |= 1 << square.down(1).right(1).index();
            }
        }

        white_pawn_attacks_at_square[index] = BitBoard::new(white_pawn_attacks);
        black_pawn_attacks_at_square[index] = BitBoard::new(black_pawn_attacks);

        index += 1;
        if index == 64 {
            break;
        }
    }

    PawnAttacks {
        white_pawn_attacks_at_square,
        black_pawn_attacks_at_square,
    }
};

fn gen_promotions(add_move: &mut dyn FnMut(Move), from: Square, to: Square) {
    for promotion in Flag::PROMOTIONS {
        add_move(Move {
            from,
            to,
            flag: promotion,
        });
    }
}

pub const fn attack_bit_board(from: Square, white: bool) -> BitBoard {
    if white {
        PAWN_ATTACKS.white_pawn_attacks_at_square[from.usize()]
    } else {
        PAWN_ATTACKS.black_pawn_attacks_at_square[from.usize()]
    }
}

pub fn generate(
    move_generator: &MoveGenerator,
    add_move: &mut dyn FnMut(Move),
    captures_only: bool,
) {
    let promotion_rank = if move_generator.white_to_move {
        BitBoard::RANK_8
    } else {
        BitBoard::RANK_1
    };

    {
        // Captures

        let non_orthogonally_pinned_pawns =
            move_generator.friendly_pawns & !move_generator.orthogonal_pin_rays;

        let not_on_the_right_edge = if move_generator.white_to_move {
            BitBoard::NOT_H_FILE
        } else {
            BitBoard::NOT_A_FILE
        };
        let not_on_the_left_edge = if move_generator.white_to_move {
            BitBoard::NOT_A_FILE
        } else {
            BitBoard::NOT_H_FILE
        };
        let capture_right_offset = if move_generator.white_to_move { -9 } else { 9 };
        let capture_left_offset = if move_generator.white_to_move { -7 } else { 7 };

        let can_capture = move_generator.enemy_piece_bit_board & move_generator.check_mask;
        let capture_right = if move_generator.white_to_move {
            (non_orthogonally_pinned_pawns & not_on_the_right_edge) << 9
        } else {
            (non_orthogonally_pinned_pawns & not_on_the_right_edge) >> 9
        } & can_capture;

        let capture_left = if move_generator.white_to_move {
            (non_orthogonally_pinned_pawns & not_on_the_left_edge) << 7
        } else {
            (non_orthogonally_pinned_pawns & not_on_the_left_edge) >> 7
        } & can_capture;

        macro_rules! promotion_captures {
                ($captures:expr, $offset:expr) => {
                    consume_bit_board!($captures, capture {
                        let from = capture.offset($offset);

                        let is_diagonally_pinned = move_generator.diagonal_pin_rays.get(&from);

                        if is_diagonally_pinned && !move_generator.diagonal_pin_rays.get(&capture) {
                            continue;
                        }
                        gen_promotions(add_move, from, capture)
                    });
                };
            }

        macro_rules! captures_no_promotions {
                ($captures:expr, $offset:expr) => {
                    consume_bit_board!($captures, capture {
                        let from = capture.offset($offset);

                        let is_diagonally_pinned = move_generator.diagonal_pin_rays.get(&from);

                        if is_diagonally_pinned && !move_generator.diagonal_pin_rays.get(&capture) {
                            continue;
                        }
                        add_move(Move {
                            from,
                            to: capture,
                            flag: Flag::None,
                        });
                    });
                };
            }

        let mut capture_right_promotions = capture_right & promotion_rank;
        let mut capture_right_no_promotions = capture_right & !capture_right_promotions;
        promotion_captures!(capture_right_promotions, capture_right_offset);
        captures_no_promotions!(capture_right_no_promotions, capture_right_offset);

        let mut capture_left_promotions = capture_left & promotion_rank;
        let mut capture_left_no_promotions = capture_left & !capture_left_promotions;
        promotion_captures!(capture_left_promotions, capture_left_offset);
        captures_no_promotions!(capture_left_no_promotions, capture_left_offset);
    }

    if let Some(en_passant_square) = move_generator.en_passant_square {
        // En passant

        let capture_position =
            en_passant_square.down(if move_generator.white_to_move { 1 } else { -1 });
        if move_generator.check_mask.get(&capture_position) {
            let mut pawns_able_to_en_passant = move_generator.friendly_pawns
                & {
                    // Generate attacks for an imaginary enemy pawn at the en passant square
                    // The up-left and up-right of en_passant_square are squares that we can en passant from
                    attack_bit_board(en_passant_square, !move_generator.white_to_move)
                }
                & !move_generator.orthogonal_pin_rays;
            'en_passant_check: while pawns_able_to_en_passant.is_not_empty() {
                let from = pawns_able_to_en_passant.pop_square();

                if move_generator.diagonal_pin_rays.get(&from)
                    && !move_generator.diagonal_pin_rays.get(&en_passant_square)
                {
                    continue;
                }

                if move_generator.friendly_king_square.rank() == from.rank() {
                    // Check if en passant will reveal a check
                    // Not covered by pin rays because enemy pawn was blocking
                    // Check by pretending the king is a rook to find enemy queens/rooks that are not obstructed
                    let unblocked = get_rook_moves(
                        move_generator.friendly_king_square,
                        (move_generator.occupied_squares
                            ^ from.bit_board()
                            ^ capture_position.bit_board())
                            & relevant_rook_blockers(move_generator.friendly_king_square),
                    );

                    if unblocked.overlaps(&move_generator.enemy_orthogonal) {
                        continue 'en_passant_check;
                    }
                }

                add_move(Move {
                    from,
                    to: en_passant_square,
                    flag: Flag::EnPassant,
                });
            }
        }
    }

    if captures_only {
        return;
    }

    let single_push = if move_generator.white_to_move {
        (move_generator.friendly_pawns & !move_generator.diagonal_pin_rays) << 8
    } else {
        (move_generator.friendly_pawns & !move_generator.diagonal_pin_rays) >> 8
    } & move_generator.empty_squares;
    let one_down_offset = if move_generator.white_to_move { -8 } else { 8 };

    {
        // Move pawn one square up

        let mut push_promotions = single_push & move_generator.push_mask & promotion_rank;

        let mut single_push_no_promotions =
            single_push & move_generator.push_mask & !push_promotions;

        consume_bit_board!(single_push_no_promotions, to {
            let from = to.offset(one_down_offset);
            if !move_generator.orthogonal_pin_rays.get(&from)
                || move_generator.orthogonal_pin_rays.get(&to)
            {
                add_move(Move {
                    from,
                    to,
                    flag: Flag::None,
                });
            }
        });

        consume_bit_board!(push_promotions, to {
            let from = to.offset(one_down_offset);
            if !move_generator.orthogonal_pin_rays.get(&from)
                || move_generator.orthogonal_pin_rays.get(&to)
            {
                gen_promotions(add_move, from, to);
            }
        });
    }

    {
        // Move pawn two squares up
        let double_push = if move_generator.white_to_move {
            single_push << 8
        } else {
            single_push >> 8
        } & move_generator.push_mask;
        let double_down_offset = one_down_offset * 2;
        let mut double_push = double_push
            & move_generator.empty_squares
            & if move_generator.white_to_move {
                BitBoard::RANK_4
            } else {
                BitBoard::RANK_5
            };

        consume_bit_board!(double_push, to {
            let from = to.offset(double_down_offset);
            if !move_generator.orthogonal_pin_rays.get(&from)
                || move_generator.orthogonal_pin_rays.get(&to)
            {
                add_move(Move {
                    from,
                    to,
                    flag: Flag::PawnTwoUp,
                });
            }
        });
    }
}
