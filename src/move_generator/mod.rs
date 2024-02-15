use crate::board::bit_board::BitBoard;
use crate::board::piece::Piece;
use crate::board::square::{Square, DIRECTIONS};
use crate::board::Board;

mod maker;
pub mod move_data;
mod precomputed;
mod slider_keys;
mod slider_lookup;

use self::move_data::{Flag, Move};
use self::precomputed::{
    king_moves_at_square, knight_moves_at_square, pawn_attacks, SQUARES_FROM_EDGE,
};
use self::slider_lookup::{
    all_rays, get_bishop_moves, get_rook_moves, relevant_bishop_blockers, relevant_rook_blockers,
};

pub struct MoveGenerator {
    white_to_move: bool,

    king_side: bool,
    queen_side: bool,

    en_passant_square: Option<Square>,

    friendly_piece_bit_board: BitBoard,

    friendly_pawns: BitBoard,
    friendly_knights: BitBoard,
    friendly_diagonal: BitBoard,
    friendly_orthogonal: BitBoard,

    friendly_king_square: Square,

    king_danger_bit_board: BitBoard,

    enemy_piece_bit_board: BitBoard,

    enemy_rooks_and_queens_bit_board: BitBoard,

    occupied_squares: BitBoard,
    empty_squares: BitBoard,

    is_in_check: bool,
    is_in_double_check: bool,

    diagonal_pin_rays: BitBoard,
    orthogonal_pin_rays: BitBoard,

    capture_mask: BitBoard,
    push_mask: BitBoard,

    enemy_pawn_attacks: BitBoard,
}

impl MoveGenerator {
    fn gen_promotions(add_move: &mut dyn FnMut(Move), from: Square, to: Square) {
        for promotion in Flag::PROMOTIONS {
            add_move(Move {
                from,
                to,
                flag: promotion,
            });
        }
    }
    fn pawn_attack_bit_board(from: Square, white: bool) -> BitBoard {
        if white {
            pawn_attacks().white_pawn_attacks_at_square[from.usize()]
        } else {
            pawn_attacks().black_pawn_attacks_at_square[from.usize()]
        }
    }
    fn gen_pawns(&self, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let promotion_rank = if self.white_to_move {
            BitBoard::RANK_8
        } else {
            BitBoard::RANK_1
        };

        {
            // Captures

            const NOT_A_FILE: BitBoard = BitBoard::new(!0x101010101010101);
            const NOT_H_FILE: BitBoard = BitBoard::new(!(0x101010101010101 << 7));
            let non_orthogonally_pinned_pawns = self.friendly_pawns & !(self.orthogonal_pin_rays);

            let not_on_the_right_edge = if self.white_to_move {
                NOT_H_FILE
            } else {
                NOT_A_FILE
            };
            let not_on_the_left_edge = if self.white_to_move {
                NOT_A_FILE
            } else {
                NOT_H_FILE
            };
            let capture_right_offset = if self.white_to_move { -9 } else { 9 };
            let capture_left_offset = if self.white_to_move { -7 } else { 7 };

            let can_capture = self.enemy_piece_bit_board & self.capture_mask;
            let mut capture_right = if self.white_to_move {
                (non_orthogonally_pinned_pawns & not_on_the_right_edge) << 9
            } else {
                (non_orthogonally_pinned_pawns & not_on_the_right_edge) >> 9
            } & can_capture;

            let mut capture_left = if self.white_to_move {
                (non_orthogonally_pinned_pawns & not_on_the_left_edge) << 7
            } else {
                (non_orthogonally_pinned_pawns & not_on_the_left_edge) >> 7
            } & can_capture;

            macro_rules! captures {
                ($captures:expr, $offset:expr) => {
                    while $captures.is_not_empty() {
                        let capture = $captures.pop_square();
                        let from = capture.offset($offset);

                        let is_diagonally_pinned = self.diagonal_pin_rays.get(&from);

                        if is_diagonally_pinned && !self.diagonal_pin_rays.get(&capture) {
                            continue;
                        }
                        if promotion_rank.get(&capture) {
                            Self::gen_promotions(add_move, from, capture)
                        } else {
                            add_move(Move {
                                from,
                                to: capture,
                                flag: Flag::None,
                            });
                        }
                    }
                };
            }

            captures!(capture_right, capture_right_offset);
            captures!(capture_left, capture_left_offset)
        }

        if let Some(en_passant_square) = self.en_passant_square {
            // En passant

            let capture_position = en_passant_square.down(if self.white_to_move { 1 } else { -1 });
            if self.capture_mask.get(&capture_position) {
                let mut pawns_able_to_en_passant = self.friendly_pawns
                    & {
                        // Generate attacks for an imaginary enemy pawn at the en passant square
                        // The up-left and up-right of en_passant_square are squares that we can en passant from
                        Self::pawn_attack_bit_board(en_passant_square, !self.white_to_move)
                    }
                    & !self.orthogonal_pin_rays;
                'en_passant_check: while pawns_able_to_en_passant.is_not_empty() {
                    let from = pawns_able_to_en_passant.pop_square();

                    if self.diagonal_pin_rays.get(&from)
                        && !self.diagonal_pin_rays.get(&en_passant_square)
                    {
                        continue;
                    }

                    if self.friendly_king_square.rank() == from.rank() {
                        // Check if en passant will reveal a check
                        // Not covered by pin rays because enemy pawn was blocking
                        // Check by scanning left and right from the pawn to find enemy queens/rooks that are not obstructed
                        let right = all_rays()[from.usize()][0];
                        let blocker =
                            right & (self.occupied_squares & !capture_position.bit_board());
                        if blocker.overlaps(&self.enemy_rooks_and_queens_bit_board) {
                            continue 'en_passant_check;
                        }

                        let left = all_rays()[from.usize()][2];
                        let blocker =
                            left & (self.occupied_squares & !capture_position.bit_board());
                        if blocker.overlaps(&self.enemy_rooks_and_queens_bit_board) {
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

        let single_push = if self.white_to_move {
            (self.friendly_pawns & !self.diagonal_pin_rays) << 8
        } else {
            (self.friendly_pawns & !self.diagonal_pin_rays) >> 8
        } & self.empty_squares;
        let one_down_offset = if self.white_to_move { -8 } else { 8 };

        {
            // Move pawn one square up

            let mut push_promotions = single_push & self.push_mask & promotion_rank;

            let mut single_push_no_promotions = single_push & self.push_mask & !push_promotions;

            while single_push_no_promotions.is_not_empty() {
                let move_to = single_push_no_promotions.pop_square();
                let from = move_to.offset(one_down_offset);
                if !self.orthogonal_pin_rays.get(&from) || self.orthogonal_pin_rays.get(&move_to) {
                    add_move(Move {
                        from,
                        to: move_to,
                        flag: Flag::None,
                    });
                }
            }
            while push_promotions.is_not_empty() {
                let move_to = push_promotions.pop_square();
                let from = move_to.offset(one_down_offset);
                if !self.orthogonal_pin_rays.get(&from) || self.orthogonal_pin_rays.get(&move_to) {
                    Self::gen_promotions(add_move, from, move_to)
                }
            }
        }

        {
            // Move pawn two squares up
            let double_push = if self.white_to_move {
                single_push << 8
            } else {
                single_push >> 8
            } & self.push_mask;
            let double_down_offset = one_down_offset * 2;
            let mut double_push = double_push
                & self.empty_squares
                & if self.white_to_move {
                    BitBoard::RANK_4
                } else {
                    BitBoard::RANK_5
                };
            while double_push.is_not_empty() {
                let move_to = double_push.pop_square();
                let from = move_to.offset(double_down_offset);
                if !self.orthogonal_pin_rays.get(&from) || self.orthogonal_pin_rays.get(&move_to) {
                    add_move(Move {
                        from,
                        to: move_to,
                        flag: Flag::PawnTwoUp,
                    })
                }
            }
        }
    }
}

impl MoveGenerator {
    fn knight_attack_bit_board(square: Square) -> BitBoard {
        knight_moves_at_square()[square.usize()]
    }

    fn gen_knights(&self, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let mut non_pinned_knights =
            self.friendly_knights & !(self.diagonal_pin_rays | self.orthogonal_pin_rays);

        let mut mask = (self.capture_mask | self.push_mask) & !self.friendly_piece_bit_board;
        if captures_only {
            mask &= self.enemy_piece_bit_board
        }

        while non_pinned_knights.is_not_empty() {
            let from = non_pinned_knights.pop_square();
            let mut knight_moves = Self::knight_attack_bit_board(from) & mask;
            while knight_moves.is_not_empty() {
                let move_to = knight_moves.pop_square();
                add_move(Move {
                    from,
                    to: move_to,
                    flag: Flag::None,
                })
            }
        }
    }
}

impl MoveGenerator {
    fn gen_bishop(&self, from: Square, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let blockers = self.occupied_squares & relevant_bishop_blockers()[from.usize()];
        let possible_moves = get_bishop_moves(from, blockers);
        let mut legal_moves =
            possible_moves & !self.friendly_piece_bit_board & (self.capture_mask | self.push_mask);
        if captures_only {
            legal_moves &= self.enemy_piece_bit_board;
        }
        if self.diagonal_pin_rays.get(&from) {
            legal_moves &= self.diagonal_pin_rays;
        }

        while legal_moves.is_not_empty() {
            let move_to = legal_moves.pop_square();
            add_move(Move {
                from,
                to: move_to,
                flag: Flag::None,
            })
        }
    }
    fn gen_rook(&self, from: Square, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let blockers = self.occupied_squares & relevant_rook_blockers()[from.usize()];
        let possible_moves = get_rook_moves(from, blockers);
        let mut legal_moves =
            possible_moves & !self.friendly_piece_bit_board & (self.capture_mask | self.push_mask);
        if captures_only {
            legal_moves &= self.enemy_piece_bit_board;
        }
        if self.orthogonal_pin_rays.get(&from) {
            legal_moves &= self.orthogonal_pin_rays;
        }

        while legal_moves.is_not_empty() {
            let move_to = legal_moves.pop_square();
            add_move(Move {
                from,
                to: move_to,
                flag: Flag::None,
            })
        }
    }
}

impl MoveGenerator {
    fn calculate_enemy_rook(
        from: Square,
        king_square: Square,

        capture_mask: &mut BitBoard,
        push_mask: &mut BitBoard,

        king_bit_board: &BitBoard,
        occupied_squares: &BitBoard,

        is_in_check: &mut bool,
        is_in_double_check: &mut bool,
    ) -> BitBoard {
        let rook_blockers_excluding_king =
            (*occupied_squares & !*king_bit_board) & relevant_rook_blockers()[from.usize()];
        let rook_attacks = get_rook_moves(from, rook_blockers_excluding_king);
        if rook_attacks.overlaps(king_bit_board) {
            // This piece is checking the king
            capture_mask.set(&from);

            if *is_in_check {
                *is_in_double_check = true;
            }
            *is_in_check = true;

            let ray = get_rook_moves(
                from,
                *occupied_squares & relevant_rook_blockers()[from.usize()],
            ) & !*king_bit_board
                & get_rook_moves(
                    king_square,
                    from.bit_board() & relevant_rook_blockers()[king_square.usize()],
                );

            *push_mask |= ray;
        }
        rook_attacks
    }
    fn calculate_enemy_bishop(
        from: Square,
        king_square: Square,

        capture_mask: &mut BitBoard,
        push_mask: &mut BitBoard,

        king_bit_board: &BitBoard,
        occupied_squares: &BitBoard,

        is_in_check: &mut bool,
        is_in_double_check: &mut bool,
    ) -> BitBoard {
        let bishop_blockers_excluding_king =
            (*occupied_squares & !*king_bit_board) & relevant_bishop_blockers()[from.usize()];
        let bishop_attacks = get_bishop_moves(from, bishop_blockers_excluding_king);
        if bishop_attacks.overlaps(king_bit_board) {
            // This piece is checking the king
            capture_mask.set(&from);

            if *is_in_check {
                *is_in_double_check = true;
            }
            *is_in_check = true;

            let ray = get_bishop_moves(
                from,
                *occupied_squares & relevant_bishop_blockers()[from.usize()],
            ) & !*king_bit_board
                & get_bishop_moves(
                    king_square,
                    from.bit_board() & relevant_bishop_blockers()[king_square.usize()],
                );

            *push_mask |= ray;
        }
        bishop_attacks
    }

    fn king_attack_bit_board(square: Square) -> BitBoard {
        king_moves_at_square()[square.usize()]
    }
    fn gen_king(&self, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let mut king_moves = Self::king_attack_bit_board(self.friendly_king_square)
            & !(self.friendly_piece_bit_board)
            & !(self.king_danger_bit_board);
        if captures_only {
            king_moves &= self.enemy_piece_bit_board
        }
        while king_moves.is_not_empty() {
            let move_to = king_moves.pop_square();
            add_move(Move {
                from: self.friendly_king_square,
                to: move_to,
                flag: Flag::None,
            });
        }

        if self.is_in_check || captures_only {
            return;
        }

        let cannot_castle_into = self.occupied_squares | self.king_danger_bit_board;
        if self.king_side {
            let move_to = self.friendly_king_square.right(2);
            let castle_mask = if self.white_to_move {
                BitBoard::new(0b01100000)
            } else {
                BitBoard::new(0b01100000 << 56)
            };

            if (castle_mask & cannot_castle_into).is_empty() {
                add_move(Move {
                    from: self.friendly_king_square,
                    to: move_to,
                    flag: Flag::Castle,
                })
            }
        }
        if self.queen_side {
            let move_to = self.friendly_king_square.left(2);
            let castle_block_mask = if self.white_to_move {
                BitBoard::new(0b00001110)
            } else {
                BitBoard::new(0b00001110 << 56)
            };

            if (castle_block_mask & self.occupied_squares).is_empty() {
                let castle_mask = if self.white_to_move {
                    BitBoard::new(0b00001100)
                } else {
                    BitBoard::new(0b00001100 << 56)
                };
                if (castle_mask & cannot_castle_into).is_empty() {
                    add_move(Move {
                        from: self.friendly_king_square,
                        to: move_to,
                        flag: Flag::Castle,
                    })
                }
            }
        }
    }
}

impl MoveGenerator {
    fn calculate_pin_rays(
        friendly_piece_bit_board: &BitBoard,
        friendly_king_square: &Square,

        enemy_orthogonal: &BitBoard,
        enemy_diagonal: &BitBoard,
        enemy_piece_bit_board: &BitBoard,
    ) -> (BitBoard, BitBoard) {
        let mut orthogonal_pin_rays = BitBoard::EMPTY;
        let mut diagonal_pin_rays = BitBoard::EMPTY;
        for (index, (direction, distance_from_edge)) in DIRECTIONS
            .iter()
            .zip(&SQUARES_FROM_EDGE[friendly_king_square.usize()])
            .enumerate()
        {
            let is_rook_movement = index < 4;

            let mut ray = BitBoard::EMPTY;
            let mut is_friendly_piece_on_ray = false;
            for count in 1..=*distance_from_edge {
                let move_to = friendly_king_square.offset(direction * count);
                ray.set(&move_to);

                if friendly_piece_bit_board.get(&move_to) {
                    // Friendly piece blocks ray

                    if is_friendly_piece_on_ray {
                        // This is the second time a friendly piece has blocked the ray
                        // Not pinned.
                        break;
                    }
                    is_friendly_piece_on_ray = true;
                } else if enemy_piece_bit_board.get(&move_to) {
                    let can_pin = if is_rook_movement {
                        enemy_orthogonal.get(&move_to)
                    } else {
                        enemy_diagonal.get(&move_to)
                    };
                    if !can_pin {
                        break;
                    }
                    if is_friendly_piece_on_ray {
                        // Friendly piece is blocking check, it is pinned
                        if is_rook_movement {
                            orthogonal_pin_rays |= ray
                        } else {
                            diagonal_pin_rays |= ray
                        }
                    }
                }
            }
        }
        (orthogonal_pin_rays, diagonal_pin_rays)
    }
    pub fn new(board: &Board) -> Self {
        let white_to_move = board.white_to_move;

        let en_passant_square = board.game_state.en_passant_square;

        let (friendly_pieces, enemy_pieces) = if white_to_move {
            (Piece::WHITE_PIECES, Piece::BLACK_PIECES)
        } else {
            (Piece::BLACK_PIECES, Piece::WHITE_PIECES)
        };

        let castling_rights = board.game_state.castling_rights;
        let (king_side, queen_side) = if white_to_move {
            (
                castling_rights.get_white_king_side(),
                castling_rights.get_white_queen_side(),
            )
        } else {
            (
                castling_rights.get_black_king_side(),
                castling_rights.get_black_queen_side(),
            )
        };

        let friendly_pawns = *board.get_bit_board(friendly_pieces[0]);
        let friendly_knights = *board.get_bit_board(friendly_pieces[1]);
        let friendly_bishops = *board.get_bit_board(friendly_pieces[2]);
        let friendly_rooks = *board.get_bit_board(friendly_pieces[3]);
        let friendly_queens = *board.get_bit_board(friendly_pieces[4]);
        let friendly_king = *board.get_bit_board(friendly_pieces[5]);
        let friendly_diagonal = friendly_bishops | friendly_queens;
        let friendly_orthogonal = friendly_rooks | friendly_queens;

        let friendly_piece_bit_board = friendly_pawns
            | friendly_knights
            | friendly_bishops
            | friendly_rooks
            | friendly_queens
            | friendly_king;

        let enemy_pawns = *board.get_bit_board(enemy_pieces[0]);

        let enemy_knights = *board.get_bit_board(enemy_pieces[1]);
        let enemy_bishops = *board.get_bit_board(enemy_pieces[2]);
        let enemy_rooks = *board.get_bit_board(enemy_pieces[3]);
        let enemy_queens = *board.get_bit_board(enemy_pieces[4]);
        let enemy_king = *board.get_bit_board(enemy_pieces[5]);
        let enemy_piece_bit_board =
            enemy_pawns | enemy_knights | enemy_bishops | enemy_rooks | enemy_queens | enemy_king;

        let occupied_squares = friendly_piece_bit_board | enemy_piece_bit_board;
        let empty_squares = !occupied_squares;

        let mut king_danger_bit_board = BitBoard::EMPTY;
        let mut enemy_pawn_attacks = BitBoard::EMPTY;
        let mut is_in_check = false;
        let mut is_in_double_check = false;

        let mut capture_mask = BitBoard::EMPTY;
        let mut push_mask = BitBoard::EMPTY;

        let friendly_king_square = friendly_king.first_square();

        {
            let mut enemy_pawns = enemy_pawns;
            while enemy_pawns.is_not_empty() {
                let from = enemy_pawns.pop_square();
                let pawn_attacks = Self::pawn_attack_bit_board(from, !white_to_move);
                if pawn_attacks.overlaps(&friendly_king) {
                    // Pawn is checking the king
                    if is_in_check {
                        is_in_double_check = true;
                    }
                    is_in_check = true;
                    capture_mask.set(&from)
                };
                enemy_pawn_attacks |= pawn_attacks;
            }
            king_danger_bit_board |= enemy_pawn_attacks
        }
        {
            let mut enemy_knights = enemy_knights;
            while enemy_knights.is_not_empty() {
                let from = enemy_knights.pop_square();
                let knight_attacks = Self::knight_attack_bit_board(from);
                if knight_attacks.overlaps(&friendly_king) {
                    // Knight is checking the king
                    if is_in_check {
                        is_in_double_check = true;
                    }
                    is_in_check = true;
                    capture_mask.set(&from)
                };
                king_danger_bit_board |= knight_attacks
            }
        }
        let enemy_diagonal = enemy_bishops | enemy_queens;
        {
            let mut enemy_diagonal = enemy_diagonal;
            while enemy_diagonal.is_not_empty() {
                let from = enemy_diagonal.pop_square();
                let dangerous = Self::calculate_enemy_bishop(
                    from,
                    friendly_king_square,
                    &mut capture_mask,
                    &mut push_mask,
                    &friendly_king,
                    &occupied_squares,
                    &mut is_in_check,
                    &mut is_in_double_check,
                );
                king_danger_bit_board |= dangerous
            }
        }
        let enemy_orthogonal = enemy_rooks | enemy_queens;
        {
            let mut enemy_orthogonal = enemy_orthogonal;
            while enemy_orthogonal.is_not_empty() {
                let from = enemy_orthogonal.pop_square();
                let dangerous = Self::calculate_enemy_rook(
                    from,
                    friendly_king_square,
                    &mut capture_mask,
                    &mut push_mask,
                    &friendly_king,
                    &occupied_squares,
                    &mut is_in_check,
                    &mut is_in_double_check,
                );
                king_danger_bit_board |= dangerous
            }
        }
        {
            let mut enemy_king = enemy_king;
            while enemy_king.is_not_empty() {
                let from = enemy_king.pop_square();
                king_danger_bit_board |= Self::king_attack_bit_board(from)
            }
        }

        if !is_in_check {
            capture_mask = BitBoard::FULL;
            push_mask = BitBoard::FULL;
        }

        let (orthogonal_pin_rays, diagonal_pin_rays) = Self::calculate_pin_rays(
            &friendly_piece_bit_board,
            &friendly_king_square,
            &enemy_orthogonal,
            &enemy_diagonal,
            &enemy_piece_bit_board,
        );

        Self {
            white_to_move,
            king_side,
            queen_side,
            en_passant_square,
            friendly_piece_bit_board,
            friendly_pawns,
            friendly_knights,
            friendly_diagonal,
            friendly_orthogonal,
            friendly_king_square,
            king_danger_bit_board,
            enemy_piece_bit_board,
            enemy_rooks_and_queens_bit_board: *board.get_bit_board(enemy_pieces[3])
                | *board.get_bit_board(enemy_pieces[4]),
            occupied_squares,
            empty_squares,
            is_in_check,
            is_in_double_check,
            diagonal_pin_rays,
            orthogonal_pin_rays,
            capture_mask,
            push_mask,
            enemy_pawn_attacks,
        }
    }

    pub fn gen(&self, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        self.gen_king(add_move, captures_only);
        if self.is_in_double_check {
            return;
        }

        self.gen_pawns(add_move, captures_only);
        self.gen_knights(add_move, captures_only);
        let mut friendly_diagonal = (self.friendly_diagonal) & !self.orthogonal_pin_rays;
        while friendly_diagonal.is_not_empty() {
            let from = friendly_diagonal.pop_square();
            self.gen_bishop(from, add_move, captures_only)
        }
        let mut friendly_orthogonal = (self.friendly_orthogonal) & !self.diagonal_pin_rays;
        while friendly_orthogonal.is_not_empty() {
            let from = friendly_orthogonal.pop_square();
            self.gen_rook(from, add_move, captures_only)
        }
    }

    pub fn is_in_check(&self) -> bool {
        self.is_in_check
    }
    pub fn enemy_pawn_attacks(&self) -> BitBoard {
        self.enemy_pawn_attacks
    }
}
