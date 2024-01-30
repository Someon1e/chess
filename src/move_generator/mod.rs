use crate::board::bit_board::BitBoard;
use crate::board::piece::{self, Piece};
use crate::board::square::{Square, DIRECTIONS};
use crate::board::Board;

pub mod move_data;
mod precomputed;

use move_data::Move;

use self::move_data::Flag;
use self::precomputed::PrecomputedData;

pub struct MoveGenerator<'a> {
    board: &'a mut Board,
    precomputed: PrecomputedData,
}

impl<'a> MoveGenerator<'a> {
    fn is_promotion_rank(&self, rank: i8) -> bool {
        if self.board.white_to_move {
            rank == 7
        } else {
            rank == 0
        }
    }
    fn gen_promotions(&self, moves: &mut Vec<Move>, from: Square, to: Square) {
        for promotion in Flag::PROMOTIONS {
            moves.push(Move::with_flag(from, to, promotion));
        }
    }
    fn pawn_attack_bit_board(&self, from: Square, white: bool) -> BitBoard {
        if white {
            self.precomputed.white_pawn_attacks_at_square[from.index() as usize]
        } else {
            self.precomputed.black_pawn_attacks_at_square[from.index() as usize]
        }
    }
    fn gen_pawns(
        &self,
        moves: &mut Vec<Move>,
        pawns: &BitBoard,
        capture_mask: &BitBoard,
        push_mask: &BitBoard,
        non_diagonal_pin_rays: &BitBoard,
        diagonal_pin_rays: &BitBoard,
        friendly_piece_bit_board: &BitBoard,
        enemy_piece_bit_board: &BitBoard,
        friendly_king_square: Square,
    ) {
        let pin_rays = *diagonal_pin_rays | *non_diagonal_pin_rays;
        let occupied_squares = *friendly_piece_bit_board | *enemy_piece_bit_board;
        let empty_squares = !occupied_squares;

        let (single_push, down_offset) = if self.board.white_to_move {
            ((*pawns << 8) & empty_squares, -8)
        } else {
            ((*pawns >> 8) & empty_squares, 8)
        };
        {
            // Move pawn one square up
            let mut push_promotions = single_push
                & *push_mask
                & (if self.board.white_to_move {
                    BitBoard::RANK_8
                } else {
                    BitBoard::RANK_1
                });

            let mut single_push_no_promotions = single_push & *push_mask & !push_promotions;
            while !single_push_no_promotions.is_empty() {
                let move_to = single_push_no_promotions.pop_square();
                let from = move_to.offset(down_offset);
                if !pin_rays.get(&from) || pin_rays.get(&move_to) {
                    moves.push(Move::new(from, move_to));
                }
            }
            while !push_promotions.is_empty() {
                let move_to = push_promotions.pop_square();
                let from = move_to.offset(down_offset);
                if !pin_rays.get(&from) || pin_rays.get(&move_to) {
                    self.gen_promotions(moves, from, move_to)
                }
            }
        }

        {
            // Move pawn two squares up
            let (double_push, double_down_offset) = if self.board.white_to_move {
                (single_push << 8 & *push_mask, -16)
            } else {
                (single_push >> 8 & *push_mask, 16)
            };
            let mut double_push = double_push
                & empty_squares
                & if self.board.white_to_move {
                    BitBoard::RANK_4
                } else {
                    BitBoard::RANK_5
                };
            while !double_push.is_empty() {
                let move_to = double_push.pop_square();
                let from = move_to.offset(double_down_offset);
                if !pin_rays.get(&from) || pin_rays.get(&move_to) {
                    moves.push(Move::with_flag(from, move_to, Flag::PawnTwoUp))
                }
            }
        }

        {
            // Captures
            let mut non_non_diagonally_pinned_pawns = *pawns & !(*non_diagonal_pin_rays);
            while !non_non_diagonally_pinned_pawns.is_empty() {
                let from = non_non_diagonally_pinned_pawns.pop_square();
                let is_diagonally_pinned = diagonal_pin_rays.get(&from);

                let mut attacks = self.pawn_attack_bit_board(from, self.board.white_to_move)
                    & (*capture_mask | *push_mask);

                while !attacks.is_empty() {
                    let attack = attacks.pop_square();

                    if is_diagonally_pinned && !diagonal_pin_rays.get(&attack) {
                        continue;
                    }
                    if enemy_piece_bit_board.get(&attack) {
                        if self.is_promotion_rank(attack.rank()) {
                            self.gen_promotions(moves, from, attack)
                        } else {
                            moves.push(Move::new(from, attack));
                        }
                    }
                }
            }
        }

        {
            // En passant

            if let Some(en_passant_square) = self.board.game_state.en_passant_square {
                let capture_position =
                    en_passant_square.down(if self.board.white_to_move { 1 } else { -1 });
                if capture_mask.get(&capture_position) {
                    let mut pawns_able_to_enpassant = *pawns & {
                        // Generate attacks for an imaginary enemy pawn at the en passant square
                        // The up-left and up-right of en_passant_square are squares that we can en passant from
                        self.pawn_attack_bit_board(en_passant_square, !self.board.white_to_move)
                    };

                    let (enemy_rook, enemy_queen) = if self.board.white_to_move {
                        (Piece::BlackRook, Piece::BlackQueen)
                    } else {
                        (Piece::WhiteRook, Piece::WhiteQueen)
                    };
                    let enemy_rooks_and_queens_bit_board = *self.board.get_bit_board(enemy_rook)
                        | *self.board.get_bit_board(enemy_queen);
                    'en_passant_check: while !pawns_able_to_enpassant.is_empty() {
                        let from = pawns_able_to_enpassant.pop_square();
                        if non_diagonal_pin_rays.get(&from) {
                            continue;
                        }

                        if diagonal_pin_rays.get(&from)
                            && !diagonal_pin_rays.get(&en_passant_square)
                        {
                            continue;
                        }

                        if friendly_king_square.rank() == from.rank() {
                            // Check if en passant will reveal a check
                            // Not covered by pin rays because enemy pawn was blocking
                            // Check by scanning left and right from our king to find enemy queens/rooks that are not obstructed
                            for (direction, distance_from_edge) in DIRECTIONS[2..4].iter().zip(
                                &self.precomputed.squares_from_edge[from.index() as usize][2..4],
                            ) {
                                for count in 1..=*distance_from_edge {
                                    let scanner = from.offset(direction * count);
                                    if scanner == from || scanner == capture_position {
                                        continue;
                                    }
                                    if enemy_rooks_and_queens_bit_board.get(&scanner) {
                                        continue 'en_passant_check;
                                    }
                                    if occupied_squares.get(&scanner) {
                                        break;
                                    };
                                }
                            }
                        }

                        moves.push(Move::with_flag(from, en_passant_square, Flag::EnPassant));
                    }
                }
            }
        }
    }
    fn directional_king_danger_bit_board(
        &self,
        from: Square,

        capture_mask: &mut BitBoard,
        push_mask: &mut BitBoard,
        king_bit_board: &BitBoard,
        friendly_piece_bit_board: &BitBoard,
        enemy_piece_bit_board: &BitBoard,

        directions: &[i8],
        squares_from_edge: &[i8],
    ) -> BitBoard {
        let mut attacked = BitBoard::empty();
        for (direction, distance_from_edge) in directions.iter().zip(squares_from_edge) {
            let mut ray = BitBoard::empty();
            for count in 1..=*distance_from_edge {
                let move_to = from.offset(direction * count);
                if king_bit_board.get(&move_to) {
                    // This piece is checking the king
                    capture_mask.set(&from);
                    *push_mask = *push_mask | ray;
                    ray.set(&move_to);
                } else {
                    ray.set(&move_to);
                    if (*friendly_piece_bit_board | *enemy_piece_bit_board).get(&move_to) {
                        break;
                    }
                }
            }
            attacked = attacked | ray
        }
        attacked
    }
    fn gen_directional(
        &self,
        moves: &mut Vec<Move>,
        from: Square,
        capture_mask: &BitBoard,
        push_mask: &BitBoard,
        non_diagonal_pin_rays: &BitBoard,
        diagonal_pin_rays: &BitBoard,
        friendly_piece_bit_board: &BitBoard,
        enemy_piece_bit_board: &BitBoard,

        direction_start_index: usize,
        direction_end_index: usize,
    ) {
        let is_pinned = (*non_diagonal_pin_rays | *diagonal_pin_rays).get(&from);

        let squares_from_edge = &self.precomputed.squares_from_edge[from.index() as usize];
        for index in direction_start_index..direction_end_index {
            let is_rook_movement = (index + direction_start_index) < 4;
            let direction = DIRECTIONS[index];

            for count in 1..=squares_from_edge[index] {
                let move_to = from.offset(direction * count);

                if is_pinned {
                    if is_rook_movement {
                        if !non_diagonal_pin_rays.get(&move_to) {
                            break;
                        }
                    } else {
                        if !diagonal_pin_rays.get(&move_to) {
                            break;
                        }
                    }
                }

                if friendly_piece_bit_board.get(&move_to) {
                    break;
                }
                if (*capture_mask | *push_mask).get(&move_to) {
                    moves.push(Move::new(from, move_to));
                }
                if enemy_piece_bit_board.get(&move_to) {
                    break;
                }
            }
        }
    }
    fn knight_attack_bit_board(&self, square: Square) -> BitBoard {
        self.precomputed.knight_moves_at_square[square.index() as usize]
    }

    fn gen_knights(
        &self,
        moves: &mut Vec<Move>,
        knights: &BitBoard,
        capture_mask: &BitBoard,
        push_mask: &BitBoard,
        non_diagonal_pin_rays: &BitBoard,
        diagonal_pin_rays: &BitBoard,
        friendly_pieces: &BitBoard,
    ) {
        let mut non_pinned_knights = *knights & !(*diagonal_pin_rays | *non_diagonal_pin_rays);
        while !non_pinned_knights.is_empty() {
            let from = non_pinned_knights.pop_square();
            let mut knight_moves = self.knight_attack_bit_board(from)
                & !(*friendly_pieces)
                & (*capture_mask | *push_mask);
            while !knight_moves.is_empty() {
                let move_to = knight_moves.pop_square();
                moves.push(Move::new(from, move_to))
            }
        }
    }

    fn king_attack_bit_board(&self, square: Square) -> BitBoard {
        self.precomputed.king_moves_at_square[square.index() as usize]
    }
    fn gen_king(
        &self,
        moves: &mut Vec<Move>,
        from: Square,
        friendly_piece_bit_board: &BitBoard,
        enemy_piece_bit_board: &BitBoard,
        king_danger_bit_board: &BitBoard,
    ) {
        let mut king_moves = self.king_attack_bit_board(from)
            & !(*friendly_piece_bit_board)
            & !(*king_danger_bit_board);
        while !king_moves.is_empty() {
            let move_to = king_moves.pop_square();
            moves.push(Move::new(from, move_to));
        }

        if king_danger_bit_board.get(&from) {
            return;
        }

        let castling_rights = self.board.game_state.castling_rights;
        let (king_side, queen_side) = if self.board.white_to_move {
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

        let occupied_squares = *friendly_piece_bit_board | *enemy_piece_bit_board;
        let cannot_castle_into = occupied_squares | *king_danger_bit_board;
        if king_side {
            let move_to = from.right(2);
            let castle_mask = if self.board.white_to_move {
                BitBoard::new(0b01100000)
            } else {
                BitBoard::new(0b01100000 << 56)
            };

            if (castle_mask & cannot_castle_into).is_empty() {
                moves.push(Move::with_flag(from, move_to, Flag::Castle))
            }
        }
        if queen_side {
            let move_to = from.left(2);
            let castle_block_mask = if self.board.white_to_move {
                BitBoard::new(0b00001110)
            } else {
                BitBoard::new(0b00001110 << 56)
            };

            if (castle_block_mask & occupied_squares).is_empty() {
                let castle_mask = if self.board.white_to_move {
                    BitBoard::new(0b00001100)
                } else {
                    BitBoard::new(0b00001100 << 56)
                };
                if (castle_mask & cannot_castle_into).is_empty() {
                    moves.push(Move::with_flag(from, move_to, Flag::Castle))
                }
            }
        }
    }
    pub fn new(board: &'a mut Board) -> Self {
        let precomputed = PrecomputedData::compute();
        Self { board, precomputed }
    }
    pub fn board(&self) -> &Board {
        self.board
    }
    pub fn board_mut(&mut self) -> &mut Board {
        self.board
    }

    pub fn gen(&self, moves: &mut Vec<Move>) {
        let (friendly_pieces, enemy_pieces) = if self.board.white_to_move {
            (piece::WHITE_PIECES, piece::BLACK_PIECES)
        } else {
            (piece::BLACK_PIECES, piece::WHITE_PIECES)
        };

        let mut friendly_piece_bit_board = BitBoard::empty();
        for piece in friendly_pieces {
            let bit_board = self.board.get_bit_board(piece);
            friendly_piece_bit_board = friendly_piece_bit_board | *bit_board
        }

        let mut enemy_piece_bit_board = BitBoard::empty();
        for piece in enemy_pieces {
            let bit_board = *self.board.get_bit_board(piece);
            enemy_piece_bit_board = enemy_piece_bit_board | bit_board;
        }

        let mut king_bit_board = *self.board.get_bit_board(friendly_pieces[5]);

        let mut king_danger_bit_board = BitBoard::empty();
        let mut is_in_check = false;
        let mut is_in_double_check = false;
        let mut checkers = BitBoard::empty();

        let mut capture_mask = BitBoard::empty();
        let mut push_mask = BitBoard::empty();

        let king_square = king_bit_board.first_square();

        for piece in enemy_pieces {
            let mut bit_board = *self.board.get_bit_board(piece);
            while !bit_board.is_empty() {
                let from = bit_board.pop_square();
                let dangerous = match piece {
                    Piece::WhitePawn | Piece::BlackPawn => {
                        let pawn_attacks =
                            self.pawn_attack_bit_board(from, !self.board.white_to_move);
                        if pawn_attacks.get(&king_square) {
                            // Pawn is checking the king
                            capture_mask.set(&from)
                        };
                        pawn_attacks
                    }
                    Piece::WhiteKnight | Piece::BlackKnight => {
                        let knight_attacks = self.knight_attack_bit_board(from);
                        if knight_attacks.get(&king_square) {
                            // Knight is checking the king
                            capture_mask.set(&from)
                        };
                        knight_attacks
                    }
                    Piece::WhiteBishop | Piece::BlackBishop => self
                        .directional_king_danger_bit_board(
                            from,
                            &mut capture_mask,
                            &mut push_mask,
                            &mut king_bit_board,
                            &friendly_piece_bit_board,
                            &enemy_piece_bit_board,
                            &DIRECTIONS[4..8],
                            &self.precomputed.squares_from_edge[from.index() as usize][4..8],
                        ),
                    Piece::WhiteRook | Piece::BlackRook => self.directional_king_danger_bit_board(
                        from,
                        &mut capture_mask,
                        &mut push_mask,
                        &mut king_bit_board,
                        &friendly_piece_bit_board,
                        &enemy_piece_bit_board,
                        &DIRECTIONS[0..4],
                        &self.precomputed.squares_from_edge[from.index() as usize][0..4],
                    ),
                    Piece::WhiteQueen | Piece::BlackQueen => self
                        .directional_king_danger_bit_board(
                            from,
                            &mut capture_mask,
                            &mut push_mask,
                            &mut king_bit_board,
                            &friendly_piece_bit_board,
                            &enemy_piece_bit_board,
                            &DIRECTIONS,
                            &self.precomputed.squares_from_edge[from.index() as usize],
                        ),
                    Piece::WhiteKing | Piece::BlackKing => self.king_attack_bit_board(from),
                };
                if !(dangerous & king_bit_board).is_empty() {
                    if is_in_check {
                        is_in_double_check = true;
                    }
                    is_in_check = true;
                    checkers.set(&from)
                }
                king_danger_bit_board = king_danger_bit_board | dangerous
            }
        }

        let mut non_diagonal_pin_rays = BitBoard::empty();
        let mut diagonal_pin_rays = BitBoard::empty();
        for (index, (direction, distance_from_edge)) in DIRECTIONS
            .iter()
            .zip(&self.precomputed.squares_from_edge[king_square.index() as usize])
            .enumerate()
        {
            let is_rook_movement = index < 4;

            let mut ray = BitBoard::empty();
            let mut is_friendly_piece_on_ray = false;
            for count in 1..=*distance_from_edge {
                let move_to = king_square.offset(direction * count);
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
                } else if let Some(enemy_piece) = self.board.enemy_piece_at(move_to) {
                    let is_queen =
                        enemy_piece == Piece::WhiteQueen || enemy_piece == Piece::BlackQueen;
                    let is_rook =
                        enemy_piece == Piece::WhiteRook || enemy_piece == Piece::BlackRook;
                    let is_bishop =
                        enemy_piece == Piece::WhiteBishop || enemy_piece == Piece::BlackBishop;
                    if is_queen || (is_rook_movement && is_rook) || (!is_rook_movement && is_bishop)
                    {
                        if is_friendly_piece_on_ray {
                            // Friendly piece is blocking check, it is pinned
                            if is_rook_movement {
                                non_diagonal_pin_rays = non_diagonal_pin_rays | ray
                            } else {
                                diagonal_pin_rays = diagonal_pin_rays | ray
                            }
                        }
                    } else {
                        break;
                    }
                }
            }
        }

        self.gen_king(
            moves,
            king_square,
            &friendly_piece_bit_board,
            &enemy_piece_bit_board,
            &king_danger_bit_board,
        );
        if is_in_double_check {
            return;
        }
        if !is_in_check {
            capture_mask = !BitBoard::empty();
            push_mask = !BitBoard::empty();
        }

        self.gen_pawns(
            moves,
            self.board.get_bit_board(friendly_pieces[0]),
            &capture_mask,
            &push_mask,
            &non_diagonal_pin_rays,
            &diagonal_pin_rays,
            &friendly_piece_bit_board,
            &enemy_piece_bit_board,
            king_square,
        );
        self.gen_knights(
            moves,
            self.board.get_bit_board(friendly_pieces[1]),
            &capture_mask,
            &push_mask,
            &non_diagonal_pin_rays,
            &diagonal_pin_rays,
            &friendly_piece_bit_board,
        );
        let mut bishop_bit_board = *self.board.get_bit_board(friendly_pieces[2]);
        while !bishop_bit_board.is_empty() {
            let square = bishop_bit_board.pop_square();
            self.gen_directional(
                moves,
                square,
                &capture_mask,
                &push_mask,
                &non_diagonal_pin_rays,
                &diagonal_pin_rays,
                &friendly_piece_bit_board,
                &enemy_piece_bit_board,
                4,
                8,
            )
        }
        let mut rook_bit_board = *self.board.get_bit_board(friendly_pieces[3]);
        while !rook_bit_board.is_empty() {
            let square = rook_bit_board.pop_square();
            self.gen_directional(
                moves,
                square,
                &capture_mask,
                &push_mask,
                &non_diagonal_pin_rays,
                &diagonal_pin_rays,
                &friendly_piece_bit_board,
                &enemy_piece_bit_board,
                0,
                4,
            )
        }
        let mut queen_bit_board = *self.board.get_bit_board(friendly_pieces[4]);
        while !queen_bit_board.is_empty() {
            let square = queen_bit_board.pop_square();
            self.gen_directional(
                moves,
                square,
                &capture_mask,
                &push_mask,
                &non_diagonal_pin_rays,
                &diagonal_pin_rays,
                &friendly_piece_bit_board,
                &enemy_piece_bit_board,
                0,
                8,
            )
        }
    }
}
