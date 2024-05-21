use crate::board::bit_board::BitBoard;
use crate::board::piece::Piece;
use crate::board::square::{Square, DIRECTIONS};
use crate::board::Board;

mod maker;
mod pawn_move_generator;
mod precomputed;

/// Move data.
pub mod move_data;

/// Magic keys used for slider lookup.
pub mod slider_keys;

/// Handles slider look up.
pub mod slider_lookup;

use self::move_data::{Flag, Move};
use self::pawn_move_generator::PawnMoveGenerator;
use self::precomputed::{KING_MOVES_AT_SQUARE, KNIGHT_MOVES_AT_SQUARE, SQUARES_FROM_EDGE};
use self::slider_lookup::{
    get_bishop_moves, get_rook_moves, RELEVANT_BISHOP_BLOCKERS, RELEVANT_ROOK_BLOCKERS,
};

/// Legal move generator.
#[allow(clippy::struct_excessive_bools)]
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

    check_mask: BitBoard,
    push_mask: BitBoard,

    enemy_pawn_attacks: BitBoard,
}

impl MoveGenerator {
    fn gen_pawns(&self, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        PawnMoveGenerator::generate(self, add_move, captures_only);
    }
}

impl MoveGenerator {
    const fn knight_attack_bit_board(square: Square) -> BitBoard {
        KNIGHT_MOVES_AT_SQUARE[square.usize()]
    }

    fn gen_knights(&self, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let mut non_pinned_knights =
            self.friendly_knights & !(self.diagonal_pin_rays | self.orthogonal_pin_rays);

        let mut mask = (self.check_mask | self.push_mask) & !self.friendly_piece_bit_board;
        if captures_only {
            mask &= self.enemy_piece_bit_board;
        }

        while non_pinned_knights.is_not_empty() {
            let from = non_pinned_knights.pop_square();
            let mut knight_moves = Self::knight_attack_bit_board(from) & mask;
            while knight_moves.is_not_empty() {
                let to = knight_moves.pop_square();
                add_move(Move {
                    from,
                    to,
                    flag: Flag::None,
                });
            }
        }
    }
}

impl MoveGenerator {
    fn gen_bishop(&self, from: Square, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let blockers = self.occupied_squares & RELEVANT_BISHOP_BLOCKERS[from.usize()];
        let possible_moves = get_bishop_moves(from, blockers);
        let mut legal_moves =
            possible_moves & !self.friendly_piece_bit_board & (self.check_mask | self.push_mask);
        if captures_only {
            legal_moves &= self.enemy_piece_bit_board;
        }
        if self.diagonal_pin_rays.get(&from) {
            legal_moves &= self.diagonal_pin_rays;
        }

        while legal_moves.is_not_empty() {
            let to = legal_moves.pop_square();
            add_move(Move {
                from,
                to,
                flag: Flag::None,
            });
        }
    }
    fn gen_rook(&self, from: Square, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let blockers = self.occupied_squares & RELEVANT_ROOK_BLOCKERS[from.usize()];
        let possible_moves = get_rook_moves(from, blockers);
        let mut legal_moves =
            possible_moves & !self.friendly_piece_bit_board & (self.check_mask | self.push_mask);
        if captures_only {
            legal_moves &= self.enemy_piece_bit_board;
        }
        if self.orthogonal_pin_rays.get(&from) {
            legal_moves &= self.orthogonal_pin_rays;
        }

        while legal_moves.is_not_empty() {
            let to = legal_moves.pop_square();
            add_move(Move {
                from,
                to,
                flag: Flag::None,
            });
        }
    }
}

impl MoveGenerator {
    fn calculate_check(
        white_to_move: bool,
        king_square: Square,
        enemy_pawns: BitBoard,
        enemy_knights: BitBoard,
        enemy_diagonal: BitBoard,
        enemy_orthogonal: BitBoard,
        occupied_squares: BitBoard,
    ) -> BitBoard {
        let mut check_mask = BitBoard::EMPTY;

        let diagonal_blockers = occupied_squares & RELEVANT_BISHOP_BLOCKERS[king_square.usize()];
        let diagonal_attacks = get_bishop_moves(king_square, diagonal_blockers);
        let diagonal_attacker = diagonal_attacks & enemy_diagonal;
        if diagonal_attacker.is_not_empty() {
            check_mask |= diagonal_attacker;
        }

        let orthogonal_blockers = occupied_squares & RELEVANT_ROOK_BLOCKERS[king_square.usize()];
        let orthogonal_attacks = get_rook_moves(king_square, orthogonal_blockers);
        let orthogonal_attacker = orthogonal_attacks & enemy_orthogonal;
        if orthogonal_attacker.is_not_empty() {
            check_mask |= orthogonal_attacker;
        }

        let pawn_check = PawnMoveGenerator::attack_bit_board(king_square, white_to_move);
        let pawn_attacker = pawn_check & enemy_pawns;
        if pawn_attacker.is_not_empty() {
            check_mask |= pawn_attacker;
        }

        let knight_check = Self::knight_attack_bit_board(king_square);
        let knight_attacker = knight_check & enemy_knights;
        if knight_attacker.is_not_empty() {
            check_mask |= knight_attacker;
        }

        check_mask
    }
}

impl MoveGenerator {
    fn calculate_enemy_rook(
        from: Square,
        king_square: Square,

        push_mask: &mut BitBoard,

        king_bit_board: BitBoard,
        occupied_squares: BitBoard,
    ) -> BitBoard {
        let rook_blockers_excluding_king =
            (occupied_squares ^ king_bit_board) & RELEVANT_ROOK_BLOCKERS[from.usize()];
        let rook_attacks = get_rook_moves(from, rook_blockers_excluding_king);
        if rook_attacks.overlaps(&king_bit_board) {
            // This piece is checking the king

            let ray = get_rook_moves(
                from,
                occupied_squares & RELEVANT_ROOK_BLOCKERS[from.usize()],
            ) & !king_bit_board
                & get_rook_moves(
                    king_square,
                    from.bit_board() & RELEVANT_ROOK_BLOCKERS[king_square.usize()],
                );

            *push_mask |= ray;
        }
        rook_attacks
    }
    fn calculate_enemy_bishop(
        from: Square,
        king_square: Square,

        push_mask: &mut BitBoard,

        king_bit_board: BitBoard,
        occupied_squares: BitBoard,
    ) -> BitBoard {
        let bishop_blockers_excluding_king =
            (occupied_squares ^ king_bit_board) & RELEVANT_BISHOP_BLOCKERS[from.usize()];
        let bishop_attacks = get_bishop_moves(from, bishop_blockers_excluding_king);
        if bishop_attacks.overlaps(&king_bit_board) {
            // This piece is checking the king

            let ray = get_bishop_moves(
                from,
                occupied_squares & RELEVANT_BISHOP_BLOCKERS[from.usize()],
            ) & !king_bit_board
                & get_bishop_moves(
                    king_square,
                    from.bit_board() & RELEVANT_BISHOP_BLOCKERS[king_square.usize()],
                );

            *push_mask |= ray;
        }
        bishop_attacks
    }

    const fn king_attack_bit_board(square: Square) -> BitBoard {
        KING_MOVES_AT_SQUARE[square.usize()]
    }

    #[allow(clippy::unreadable_literal)]
    fn gen_king(&self, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        let mut king_moves = Self::king_attack_bit_board(self.friendly_king_square)
            & !self.friendly_piece_bit_board
            & !self.king_danger_bit_board;
        if captures_only {
            king_moves &= self.enemy_piece_bit_board;
        }
        while king_moves.is_not_empty() {
            let to = king_moves.pop_square();
            add_move(Move {
                from: self.friendly_king_square,
                to,
                flag: Flag::None,
            });
        }

        if self.is_in_check || captures_only {
            return;
        }

        let cannot_castle_into = self.occupied_squares | self.king_danger_bit_board;
        if self.king_side {
            let to = self.friendly_king_square.right(2);
            let castle_mask = if self.white_to_move {
                BitBoard::new(0b01100000)
            } else {
                BitBoard::new(0b01100000 << 56)
            };

            if !(castle_mask.overlaps(&cannot_castle_into)) {
                add_move(Move {
                    from: self.friendly_king_square,
                    to,
                    flag: Flag::Castle,
                });
            }
        }
        if self.queen_side {
            let to = self.friendly_king_square.left(2);
            let castle_block_mask = if self.white_to_move {
                BitBoard::new(0b00001110)
            } else {
                BitBoard::new(0b00001110 << 56)
            };

            if !castle_block_mask.overlaps(&self.occupied_squares) {
                let castle_mask = if self.white_to_move {
                    BitBoard::new(0b00001100)
                } else {
                    BitBoard::new(0b00001100 << 56)
                };
                if !castle_mask.overlaps(&cannot_castle_into) {
                    add_move(Move {
                        from: self.friendly_king_square,
                        to,
                        flag: Flag::Castle,
                    });
                }
            }
        }
    }
}

impl MoveGenerator {
    fn calculate_pin_rays(
        friendly_piece_bit_board: BitBoard,
        friendly_king_square: Square,

        enemy_orthogonal: BitBoard,
        enemy_diagonal: BitBoard,
        enemy_piece_bit_board: BitBoard,
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
                let to = friendly_king_square.offset(direction * count);
                ray.set(&to);

                if friendly_piece_bit_board.get(&to) {
                    // Friendly piece blocks ray

                    if is_friendly_piece_on_ray {
                        // This is the second time a friendly piece has blocked the ray
                        // Not pinned.
                        break;
                    }
                    is_friendly_piece_on_ray = true;
                } else if enemy_piece_bit_board.get(&to) {
                    let can_pin = if is_rook_movement {
                        enemy_orthogonal.get(&to)
                    } else {
                        enemy_diagonal.get(&to)
                    };
                    if !can_pin {
                        break;
                    }
                    if is_friendly_piece_on_ray {
                        // Friendly piece is blocking check, it is pinned
                        if is_rook_movement {
                            orthogonal_pin_rays |= ray;
                        } else {
                            diagonal_pin_rays |= ray;
                        }
                    }
                }
            }
        }

        (orthogonal_pin_rays, diagonal_pin_rays)
    }

    /// Creates a move generator for the current position.
    #[must_use]
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

        let enemy_diagonal = enemy_bishops | enemy_queens;
        let enemy_orthogonal = enemy_rooks | enemy_queens;
        let enemy_piece_bit_board =
            enemy_pawns | enemy_knights | enemy_diagonal | enemy_orthogonal | enemy_king;

        let occupied_squares = friendly_piece_bit_board | enemy_piece_bit_board;
        let empty_squares = !occupied_squares;

        let mut king_danger_bit_board = BitBoard::EMPTY;

        let friendly_king_square = friendly_king.first_square();

        let mut check_mask = Self::calculate_check(
            white_to_move,
            friendly_king_square,
            enemy_pawns,
            enemy_knights,
            enemy_diagonal,
            enemy_orthogonal,
            occupied_squares,
        );

        let is_in_check = check_mask.is_not_empty();
        let is_in_double_check = check_mask.more_than_one_bit_set();

        let mut push_mask = BitBoard::EMPTY;

        let enemy_pawn_attacks;
        {
            let not_on_the_right_edge = if white_to_move {
                BitBoard::NOT_A_FILE
            } else {
                BitBoard::NOT_H_FILE
            };
            let not_on_the_left_edge = if white_to_move {
                BitBoard::NOT_H_FILE
            } else {
                BitBoard::NOT_A_FILE
            };

            enemy_pawn_attacks = if board.white_to_move {
                (enemy_pawns & not_on_the_right_edge) >> 9
            } else {
                (enemy_pawns & not_on_the_right_edge) << 9
            } | if white_to_move {
                (enemy_pawns & not_on_the_left_edge) >> 7
            } else {
                (enemy_pawns & not_on_the_left_edge) << 7
            };

            king_danger_bit_board |= enemy_pawn_attacks;
        }
        {
            let mut enemy_knights = enemy_knights;
            while enemy_knights.is_not_empty() {
                let from = enemy_knights.pop_square();
                let knight_attacks = Self::knight_attack_bit_board(from);
                king_danger_bit_board |= knight_attacks;
            }
        }
        {
            let mut enemy_diagonal = enemy_diagonal;
            while enemy_diagonal.is_not_empty() {
                let from = enemy_diagonal.pop_square();
                let dangerous = Self::calculate_enemy_bishop(
                    from,
                    friendly_king_square,
                    &mut push_mask,
                    friendly_king,
                    occupied_squares,
                );
                king_danger_bit_board |= dangerous;
            }
        }
        {
            let mut enemy_orthogonal = enemy_orthogonal;
            while enemy_orthogonal.is_not_empty() {
                let from = enemy_orthogonal.pop_square();
                let dangerous = Self::calculate_enemy_rook(
                    from,
                    friendly_king_square,
                    &mut push_mask,
                    friendly_king,
                    occupied_squares,
                );
                king_danger_bit_board |= dangerous;
            }
        }
        {
            let mut enemy_king = enemy_king;
            while enemy_king.is_not_empty() {
                let from = enemy_king.pop_square();
                king_danger_bit_board |= Self::king_attack_bit_board(from);
            }
        }

        if !is_in_check {
            check_mask = BitBoard::FULL;
            push_mask = BitBoard::FULL;
        }

        let (orthogonal_pin_rays, diagonal_pin_rays) = Self::calculate_pin_rays(
            friendly_piece_bit_board,
            friendly_king_square,
            enemy_orthogonal,
            enemy_diagonal,
            enemy_piece_bit_board,
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
            enemy_rooks_and_queens_bit_board: enemy_rooks | enemy_queens,
            occupied_squares,
            empty_squares,
            is_in_check,
            is_in_double_check,
            diagonal_pin_rays,
            orthogonal_pin_rays,
            check_mask,
            push_mask,
            enemy_pawn_attacks,
        }
    }

    /// Generates all friendly piece moves
    pub fn gen(&self, add_move: &mut dyn FnMut(Move), captures_only: bool) {
        self.gen_king(add_move, captures_only);
        if self.is_in_double_check {
            // Only king can move in double check
            return;
        }

        self.gen_pawns(add_move, captures_only);
        self.gen_knights(add_move, captures_only);
        let mut friendly_diagonal = self.friendly_diagonal & !self.orthogonal_pin_rays;
        while friendly_diagonal.is_not_empty() {
            let from = friendly_diagonal.pop_square();
            self.gen_bishop(from, add_move, captures_only);
        }
        let mut friendly_orthogonal = self.friendly_orthogonal & !self.diagonal_pin_rays;
        while friendly_orthogonal.is_not_empty() {
            let from = friendly_orthogonal.pop_square();
            self.gen_rook(from, add_move, captures_only);
        }
    }

    /// Calculates whether the side to move is in check
    #[must_use]
    pub fn calculate_is_in_check(board: &Board) -> bool {
        let (friendly_pieces, enemy_pieces) = if board.white_to_move {
            (Piece::WHITE_PIECES, Piece::BLACK_PIECES)
        } else {
            (Piece::BLACK_PIECES, Piece::WHITE_PIECES)
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

        let enemy_diagonal = enemy_bishops | enemy_queens;
        let enemy_orthogonal = enemy_rooks | enemy_queens;

        let enemy_piece_bit_board =
            enemy_pawns | enemy_knights | enemy_diagonal | enemy_orthogonal | enemy_king;

        let occupied_squares = friendly_piece_bit_board | enemy_piece_bit_board;

        let king_square = friendly_king.first_square();

        let pawn_check = PawnMoveGenerator::attack_bit_board(king_square, board.white_to_move);
        let pawn_attacker = pawn_check & enemy_pawns;
        if pawn_attacker.is_not_empty() {
            return true;
        }

        let knight_check = Self::knight_attack_bit_board(king_square);
        let knight_attacker = knight_check & enemy_knights;
        if knight_attacker.is_not_empty() {
            return true;
        }

        let diagonal_blockers = occupied_squares & RELEVANT_BISHOP_BLOCKERS[king_square.usize()];
        let diagonal_attacks = get_bishop_moves(king_square, diagonal_blockers);
        let diagonal_attacker = diagonal_attacks & enemy_diagonal;
        if diagonal_attacker.is_not_empty() {
            return true;
        }

        let orthogonal_blockers = occupied_squares & RELEVANT_ROOK_BLOCKERS[king_square.usize()];
        let orthogonal_attacks = get_rook_moves(king_square, orthogonal_blockers);
        let orthogonal_attacker = orthogonal_attacks & enemy_orthogonal;
        if orthogonal_attacker.is_not_empty() {
            return true;
        }

        false
    }

    /// Returns whether the side to move is in check
    #[must_use]
    pub const fn is_in_check(&self) -> bool {
        self.is_in_check
    }

    /// Returns the enemy pawn attacks
    #[must_use]
    pub const fn enemy_pawn_attacks(&self) -> BitBoard {
        self.enemy_pawn_attacks
    }

    /// Returns the enemy piece bit board
    #[must_use]
    pub const fn enemy_piece_bit_board(&self) -> BitBoard {
        self.enemy_piece_bit_board
    }

    /// Returns the friendly pawns bit board
    #[must_use]
    pub const fn friendly_pawns(&self) -> BitBoard {
        self.friendly_pawns
    }

    /// Returns the friendly piece bit board
    #[must_use]
    pub const fn friendly_pieces(&self) -> BitBoard {
        self.friendly_piece_bit_board
    }
}
