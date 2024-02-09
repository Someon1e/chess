use crate::board::bit_board::BitBoard;
use crate::board::piece::Piece;
use crate::board::square::{Direction, Square, DIRECTIONS};
use crate::board::Board;

mod maker;
pub mod move_data;
mod precomputed;
mod slider_lookup;

use self::move_data::{Flag, Move};
use self::precomputed::PRECOMPUTED;
use self::slider_lookup::{BISHOP_BLOCKERS, BISHOP_MOVE_MAP, ROOK_BLOCKERS, ROOK_MOVE_MAP};

pub struct MoveGenerator {
    white_to_move: bool,

    king_side: bool,
    queen_side: bool,

    en_passant_square: Option<Square>,

    friendly_piece_bit_board: BitBoard,

    friendly_pawns: BitBoard,
    friendly_knights: BitBoard,
    friendly_bishops: BitBoard,
    friendly_rooks: BitBoard,
    friendly_queens: BitBoard,

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
            PRECOMPUTED.white_pawn_attacks_at_square[from.index() as usize]
        } else {
            PRECOMPUTED.black_pawn_attacks_at_square[from.index() as usize]
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
            let capture_right_offset = if self.white_to_move { -9 } else { 9 };

            let mut capture_right = if self.white_to_move {
                (non_orthogonally_pinned_pawns & not_on_the_right_edge) << 9
            } else {
                (non_orthogonally_pinned_pawns & not_on_the_right_edge) >> 9
            } & self.enemy_piece_bit_board
                & self.capture_mask;
            while !capture_right.is_empty() {
                let capture = capture_right.pop_square();
                let from = capture.offset(capture_right_offset);

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

            let capture_left_offset = if self.white_to_move { -7 } else { 7 };

            let not_on_the_left_edge = if self.white_to_move {
                NOT_A_FILE
            } else {
                NOT_H_FILE
            };

            let mut capture_left = if self.white_to_move {
                (non_orthogonally_pinned_pawns & not_on_the_left_edge) << 7
            } else {
                (non_orthogonally_pinned_pawns & not_on_the_left_edge) >> 7
            } & self.enemy_piece_bit_board
                & self.capture_mask;
            while !capture_left.is_empty() {
                let capture = capture_left.pop_square();
                let from = capture.offset(capture_left_offset);
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
        }

        if let Some(en_passant_square) = self.en_passant_square {
            // En passant

            let capture_position = en_passant_square.down(if self.white_to_move { 1 } else { -1 });
            if self.capture_mask.get(&capture_position) {
                let mut pawns_able_to_en_passant = self.friendly_pawns & {
                    // Generate attacks for an imaginary enemy pawn at the en passant square
                    // The up-left and up-right of en_passant_square are squares that we can en passant from
                    Self::pawn_attack_bit_board(en_passant_square, !self.white_to_move)
                };
                'en_passant_check: while !pawns_able_to_en_passant.is_empty() {
                    let from = pawns_able_to_en_passant.pop_square();
                    if self.orthogonal_pin_rays.get(&from) {
                        continue;
                    }

                    if self.diagonal_pin_rays.get(&from)
                        && !self.diagonal_pin_rays.get(&en_passant_square)
                    {
                        continue;
                    }

                    if self.friendly_king_square.rank() == from.rank() {
                        // Check if en passant will reveal a check
                        // Not covered by pin rays because enemy pawn was blocking
                        // Check by scanning left and right from our king to find enemy queens/rooks that are not obstructed
                        for (direction, distance_from_edge) in DIRECTIONS[2..4]
                            .iter()
                            .zip(&PRECOMPUTED.squares_from_edge[from.index() as usize][2..4])
                        {
                            for count in 1..=*distance_from_edge {
                                let scanner = from.offset(direction * count);
                                if scanner == from || scanner == capture_position {
                                    continue;
                                }
                                if self.enemy_rooks_and_queens_bit_board.get(&scanner) {
                                    continue 'en_passant_check;
                                }
                                if self.occupied_squares.get(&scanner) {
                                    break;
                                };
                            }
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

        let (single_push, down_offset) = if self.white_to_move {
            (
                ((self.friendly_pawns & !self.diagonal_pin_rays) << 8) & self.empty_squares,
                -8,
            )
        } else {
            (
                ((self.friendly_pawns & !self.diagonal_pin_rays) >> 8) & self.empty_squares,
                8,
            )
        };

        {
            // Move pawn one square up

            let mut push_promotions = single_push & self.push_mask & promotion_rank;

            let mut single_push_no_promotions = single_push & self.push_mask & !push_promotions;

            while !single_push_no_promotions.is_empty() {
                let move_to = single_push_no_promotions.pop_square();
                let from = move_to.offset(down_offset);
                if !self.orthogonal_pin_rays.get(&from) || self.orthogonal_pin_rays.get(&move_to) {
                    add_move(Move {
                        from,
                        to: move_to,
                        flag: Flag::None,
                    });
                }
            }
            while !push_promotions.is_empty() {
                let move_to = push_promotions.pop_square();
                let from = move_to.offset(down_offset);
                if !self.orthogonal_pin_rays.get(&from) || self.orthogonal_pin_rays.get(&move_to) {
                    Self::gen_promotions(add_move, from, move_to)
                }
            }
        }

        {
            // Move pawn two squares up
            let (double_push, double_down_offset) = if self.white_to_move {
                (single_push << 8 & self.push_mask, -16)
            } else {
                (single_push >> 8 & self.push_mask, 16)
            };
            let mut double_push = double_push
                & self.empty_squares
                & if self.white_to_move {
                    BitBoard::RANK_4
                } else {
                    BitBoard::RANK_5
                };
            while !double_push.is_empty() {
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
        PRECOMPUTED.knight_moves_at_square[square.index() as usize]
    }

    fn gen_knights(&self, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let mut non_pinned_knights =
            self.friendly_knights & !(self.diagonal_pin_rays | self.orthogonal_pin_rays);

        let mut mask = (self.capture_mask | self.push_mask) & !self.friendly_piece_bit_board;
        if captures_only {
            mask &= self.enemy_piece_bit_board
        }

        while !non_pinned_knights.is_empty() {
            let from = non_pinned_knights.pop_square();
            let mut knight_moves = Self::knight_attack_bit_board(from) & mask;
            while !knight_moves.is_empty() {
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
        if self.orthogonal_pin_rays.get(&from) {
            return;
        }

        let blockers = self.occupied_squares & BISHOP_BLOCKERS[from.index() as usize];
        let possible_moves = BISHOP_MOVE_MAP[from.index() as usize][&(blockers)];
        let mut legal_moves =
            possible_moves & !self.friendly_piece_bit_board & (self.capture_mask | self.push_mask);
        if captures_only {
            legal_moves &= self.enemy_piece_bit_board;
        }
        if self.diagonal_pin_rays.get(&from) {
            legal_moves &= self.diagonal_pin_rays;
        }

        while !legal_moves.is_empty() {
            let move_to = legal_moves.pop_square();
            add_move(Move {
                from,
                to: move_to,
                flag: Flag::None,
            })
        }
    }
    fn gen_rook(&self, from: Square, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        if self.diagonal_pin_rays.get(&from) {
            return;
        }

        let blockers = self.occupied_squares & ROOK_BLOCKERS[from.index() as usize];
        let possible_moves = ROOK_MOVE_MAP[from.index() as usize][&(blockers)];
        let mut legal_moves =
            possible_moves & !self.friendly_piece_bit_board & (self.capture_mask | self.push_mask);
        if captures_only {
            legal_moves &= self.enemy_piece_bit_board;
        }
        if self.orthogonal_pin_rays.get(&from) {
            legal_moves &= self.orthogonal_pin_rays;
        }

        while !legal_moves.is_empty() {
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
    fn calculate_enemy_slider(
        from: Square,

        capture_mask: &mut BitBoard,
        push_mask: &mut BitBoard,

        king_bit_board: &BitBoard,
        occupied_squares: &BitBoard,

        is_in_check: &mut bool,
        is_in_double_check: &mut bool,

        directions: &[Direction],
        squares_from_edge: &[Direction],
    ) -> BitBoard {
        let mut attacked = BitBoard::EMPTY;
        for (direction, distance_from_edge) in directions.iter().zip(squares_from_edge) {
            let mut ray = BitBoard::EMPTY;
            for count in 1..=*distance_from_edge {
                let move_to = from.offset(direction * count);
                if king_bit_board.get(&move_to) {
                    // This piece is checking the king
                    capture_mask.set(&from);
                    *push_mask |= ray;
                    ray.set(&move_to);

                    if *is_in_check {
                        *is_in_double_check = true;
                    }
                    *is_in_check = true;
                } else {
                    ray.set(&move_to);
                    if occupied_squares.get(&move_to) {
                        break;
                    }
                }
            }
            attacked |= ray
        }
        attacked
    }

    fn king_attack_bit_board(square: Square) -> BitBoard {
        PRECOMPUTED.king_moves_at_square[square.index() as usize]
    }
    fn gen_king(&self, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let mut king_moves = Self::king_attack_bit_board(self.friendly_king_square)
            & !(self.friendly_piece_bit_board)
            & !(self.king_danger_bit_board);
        if captures_only {
            king_moves &= self.enemy_piece_bit_board
        }
        while !king_moves.is_empty() {
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

        enemy_bishops: &BitBoard,
        enemy_rooks: &BitBoard,
        enemy_queens: &BitBoard,
        enemy_piece_bit_board: &BitBoard,
    ) -> (BitBoard, BitBoard) {
        let mut orthogonal_pin_rays = BitBoard::EMPTY;
        let mut diagonal_pin_rays = BitBoard::EMPTY;
        for (index, (direction, distance_from_edge)) in DIRECTIONS
            .iter()
            .zip(&PRECOMPUTED.squares_from_edge[friendly_king_square.index() as usize])
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
                    } else {
                        is_friendly_piece_on_ray = true;
                    }
                } else if enemy_piece_bit_board.get(&move_to) {
                    let is_queen = enemy_queens.get(&move_to);
                    let is_rook = enemy_rooks.get(&move_to);
                    let is_bishop = enemy_bishops.get(&move_to);
                    if is_queen || (is_rook_movement && is_rook) || (!is_rook_movement && is_bishop)
                    {
                        if is_friendly_piece_on_ray {
                            // Friendly piece is blocking check, it is pinned
                            if is_rook_movement {
                                orthogonal_pin_rays |= ray
                            } else {
                                diagonal_pin_rays |= ray
                            }
                        }
                    } else {
                        break;
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
            while !enemy_pawns.is_empty() {
                let from = enemy_pawns.pop_square();
                let pawn_attacks = Self::pawn_attack_bit_board(from, !white_to_move);
                if !(pawn_attacks & friendly_king).is_empty() {
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
            while !enemy_knights.is_empty() {
                let from = enemy_knights.pop_square();
                let knight_attacks = Self::knight_attack_bit_board(from);
                if !(knight_attacks & friendly_king).is_empty() {
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
        {
            let mut enemy_bishops = enemy_bishops;
            while !enemy_bishops.is_empty() {
                let from = enemy_bishops.pop_square();
                let dangerous = Self::calculate_enemy_slider(
                    from,
                    &mut capture_mask,
                    &mut push_mask,
                    &friendly_king,
                    &occupied_squares,
                    &mut is_in_check,
                    &mut is_in_double_check,
                    &DIRECTIONS[4..8],
                    &PRECOMPUTED.squares_from_edge[from.index() as usize][4..8],
                );
                king_danger_bit_board |= dangerous
            }
        }
        {
            let mut enemy_rooks = enemy_rooks;
            while !enemy_rooks.is_empty() {
                let from = enemy_rooks.pop_square();
                let dangerous = Self::calculate_enemy_slider(
                    from,
                    &mut capture_mask,
                    &mut push_mask,
                    &friendly_king,
                    &occupied_squares,
                    &mut is_in_check,
                    &mut is_in_double_check,
                    &DIRECTIONS[0..4],
                    &PRECOMPUTED.squares_from_edge[from.index() as usize][0..4],
                );
                king_danger_bit_board |= dangerous
            }
        }
        {
            let mut enemy_queens = enemy_queens;
            while !enemy_queens.is_empty() {
                let from = enemy_queens.pop_square();
                let dangerous = Self::calculate_enemy_slider(
                    from,
                    &mut capture_mask,
                    &mut push_mask,
                    &friendly_king,
                    &occupied_squares,
                    &mut is_in_check,
                    &mut is_in_double_check,
                    &DIRECTIONS[0..8],
                    &PRECOMPUTED.squares_from_edge[from.index() as usize][0..8],
                );
                king_danger_bit_board |= dangerous
            }
        }
        {
            let mut enemy_king = enemy_king;
            while !enemy_king.is_empty() {
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
            &enemy_bishops,
            &enemy_rooks,
            &enemy_queens,
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
            friendly_bishops,
            friendly_rooks,
            friendly_queens,
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
        let mut friendly_bishops = self.friendly_bishops;
        while !friendly_bishops.is_empty() {
            let from = friendly_bishops.pop_square();
            self.gen_bishop(from, add_move, captures_only)
        }
        let mut friendly_rooks = self.friendly_rooks;
        while !friendly_rooks.is_empty() {
            let from = friendly_rooks.pop_square();
            self.gen_rook(from, add_move, captures_only)
        }
        let mut friendly_queens = self.friendly_queens;
        while !friendly_queens.is_empty() {
            let from = friendly_queens.pop_square();
            self.gen_bishop(from, add_move, captures_only);
            self.gen_rook(from, add_move, captures_only);
        }
    }

    pub fn is_in_check(&self) -> bool {
        self.is_in_check
    }
    pub fn enemy_pawn_attacks(&self) -> BitBoard {
        self.enemy_pawn_attacks
    }
}
