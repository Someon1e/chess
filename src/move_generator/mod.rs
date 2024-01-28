use crate::board::bit_board::BitBoard;
use crate::board::piece::{self, Piece};
use crate::board::square::{Square, DIRECTIONS};
use crate::board::Board;

pub mod move_data;
mod precomputed;

use move_data::Move;

use self::move_data::Flag;
use self::precomputed::PrecomputedData;

pub struct PsuedoLegalMoveGenerator<'a> {
    board: &'a mut Board,
    precomputed: PrecomputedData,
}

impl<'a> PsuedoLegalMoveGenerator<'a> {
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
    fn pawn_attack_bit_board(&self, square: Square, white: bool) -> BitBoard {
        if white {
            self.precomputed.white_pawn_attacks_at_square[square.index() as usize]
        } else {
            self.precomputed.black_pawn_attacks_at_square[square.index() as usize]
        }
    }
    fn gen_all_pawn_pushes(
        &self,
        moves: &mut Vec<Move>,
        pawns: &BitBoard,
        friendly_piece_bit_board: &BitBoard,
        enemy_piece_bit_board: &BitBoard,
    ) {
        let empty_squares = (*friendly_piece_bit_board | *enemy_piece_bit_board).not();

        let (single_push, down_offset) = if self.board.white_to_move {
            ((*pawns << 8) & empty_squares, -8)
        } else {
            ((*pawns >> 8) & empty_squares, 8)
        };
        {
            // Move pawn one square up
            let mut push_promotions = single_push
                & (if self.board.white_to_move {
                    BitBoard::RANK_8
                } else {
                    BitBoard::RANK_1
                });

            let mut single_push_no_promotions = single_push & push_promotions.not();
            while !single_push_no_promotions.is_empty() {
                let move_to = single_push_no_promotions.pop_square();
                let from = move_to.offset(down_offset);
                moves.push(Move::new(from, move_to));
            }
            while !push_promotions.is_empty() {
                let move_to = push_promotions.pop_square();
                let from = move_to.offset(down_offset);
                self.gen_promotions(moves, from, move_to)
            }
        }

        {
            // Move pawn two squares up
            let (double_push, double_down_offset) = if self.board.white_to_move {
                (single_push << 8, -16)
            } else {
                (single_push >> 8, 16)
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
                moves.push(Move::with_flag(from, move_to, Flag::PawnTwoUp))
            }
        }
    }
    pub fn gen_pawn(&self, moves: &mut Vec<Move>, square: Square, enemy_pieces: &BitBoard) {
        let mut attacks = self.pawn_attack_bit_board(square, self.board.white_to_move);

        while !attacks.is_empty() {
            let attack = attacks.pop_square();
            if enemy_pieces.get(&attack) {
                if self.is_promotion_rank(attack.rank()) {
                    self.gen_promotions(moves, square, attack)
                } else {
                    moves.push(Move::new(square, attack));
                }
            } else if let Some(en_passant_square) = self.board.game_state.en_passant_square {
                if en_passant_square == attack {
                    moves.push(Move::with_flag(square, attack, Flag::EnPassant));
                }
            }
        }
    }
    fn directional_attack_bit_board(
        &self,
        square: Square,
        friendly_piece_bit_board: &BitBoard,
        enemy_piece_bit_board: &BitBoard,

        directions: &[i8],
        squares_from_edge: &[i8],
    ) -> BitBoard {
        let mut attacked = BitBoard::empty();
        for (direction, distance_from_edge) in directions.iter().zip(squares_from_edge) {
            for count in 1..=*distance_from_edge {
                let move_to = square.offset(direction * count);
                attacked = attacked | move_to.bitboard();
                if friendly_piece_bit_board.get(&move_to) || enemy_piece_bit_board.get(&move_to) {
                    break;
                }
            }
        }
        attacked
    }
    fn gen_directional(
        &self,
        moves: &mut Vec<Move>,
        square: Square,
        friendly_piece_bit_board: &BitBoard,
        enemy_piece_bit_board: &BitBoard,

        directions: &[i8],
        squares_from_edge: &[i8],
    ) {
        for (direction, distance_from_edge) in directions.iter().zip(squares_from_edge) {
            for count in 1..=*distance_from_edge {
                let move_to = square.offset(direction * count);
                if friendly_piece_bit_board.get(&move_to) {
                    break;
                }
                moves.push(Move::new(square, move_to));
                if enemy_piece_bit_board.get(&move_to) {
                    break;
                }
            }
        }
    }
    fn knight_attack_bit_board(&self, square: Square) -> BitBoard {
        self.precomputed.knight_moves_at_square[square.index() as usize]
    }
    
    fn gen_knight(&self, moves: &mut Vec<Move>, square: Square, friendly_pieces: &BitBoard) {
        let mut knight_moves = self.knight_attack_bit_board(square) & friendly_pieces.not();
        while !knight_moves.is_empty() {
            let move_to = knight_moves.pop_square();
            moves.push(Move::new(square, move_to))
        }
    }

    fn king_attack_bit_board(&self, square: Square) -> BitBoard {
        self.precomputed.king_moves_at_square[square.index() as usize]
    }
    fn gen_king(
        &self,
        moves: &mut Vec<Move>,
        square: Square,
        friendly_piece_bit_board: &BitBoard,
        enemy_piece_bit_board: &BitBoard,
        attacked_bit_board: &BitBoard,
    ) {
        let mut king_moves =
            self.king_attack_bit_board(square) & friendly_piece_bit_board.not() & attacked_bit_board.not();
        while !king_moves.is_empty() {
            let move_to = king_moves.pop_square();
            moves.push(Move::new(square, move_to));
        }

        if attacked_bit_board.get(&square) {
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
        if king_side {
            let move_to = square.right(2);
            if !occupied_squares.get(&square.right(1))
                && !occupied_squares.get(&move_to)
            {
                moves.push(Move::with_flag(square, move_to, Flag::Castle))
            }
        }
        if queen_side {
            let move_to = square.left(2);
            if !occupied_squares.get(&square.left(1))
                && !occupied_squares.get(&move_to)
                && !occupied_squares.get(&square.left(3))
            {
                moves.push(Move::with_flag(square, move_to, Flag::Castle))
            }
        }
    }
    pub fn new(board: &'a mut Board) -> Self {
        let precomputed = PrecomputedData::compute();
        Self { board, precomputed }
    }
    pub fn board(&mut self) -> &mut Board {
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

        let mut attacked_bit_board = BitBoard::empty();
        for piece in enemy_pieces {
            let mut bit_board = *self.board.get_bit_board(piece);
            while !bit_board.is_empty() {
                let square = bit_board.pop_square();
                attacked_bit_board = attacked_bit_board
                    | match piece {
                        Piece::WhitePawn | Piece::BlackPawn => {
                            self.pawn_attack_bit_board(square, !self.board.white_to_move)
                        }
                        Piece::WhiteKnight | Piece::BlackKnight => {
                            self.knight_attack_bit_board(square)
                        }
                        Piece::WhiteBishop | Piece::BlackBishop => self
                            .directional_attack_bit_board(
                                square,
                                &friendly_piece_bit_board,
                                &enemy_piece_bit_board,
                                &DIRECTIONS[4..8],
                                &self.precomputed.squares_from_edge[square.index() as usize][4..8],
                            ),
                        Piece::WhiteRook | Piece::BlackRook => self.directional_attack_bit_board(
                            square,
                            &friendly_piece_bit_board,
                            &enemy_piece_bit_board,
                            &DIRECTIONS[0..4],
                            &self.precomputed.squares_from_edge[square.index() as usize][0..4],
                        ),
                        Piece::WhiteQueen | Piece::BlackQueen => self.directional_attack_bit_board(
                            square,
                            &friendly_piece_bit_board,
                            &enemy_piece_bit_board,
                            &DIRECTIONS,
                            &self.precomputed.squares_from_edge[square.index() as usize],
                        ),
                        Piece::WhiteKing | Piece::BlackKing => self.king_attack_bit_board(square),
                    }
            }
        }

        self.gen_all_pawn_pushes(
            moves,
            self.board.get_bit_board(friendly_pieces[0]),
            &friendly_piece_bit_board,
            &enemy_piece_bit_board,
        );
        let mut pawn_bit_board = *self.board.get_bit_board(friendly_pieces[0]);
        while !pawn_bit_board.is_empty() {
            let square = pawn_bit_board.pop_square();
            self.gen_pawn(moves, square, &enemy_piece_bit_board)
        }
        let mut knight_bit_board = *self.board.get_bit_board(friendly_pieces[1]);
        while !knight_bit_board.is_empty() {
            let square = knight_bit_board.pop_square();
            self.gen_knight(moves, square, &friendly_piece_bit_board)
        }
        let mut bishop_bit_board = *self.board.get_bit_board(friendly_pieces[2]);
        while !bishop_bit_board.is_empty() {
            let square = bishop_bit_board.pop_square();
            self.gen_directional(
                moves,
                square,
                &friendly_piece_bit_board,
                &enemy_piece_bit_board,
                &DIRECTIONS[4..8],
                &self.precomputed.squares_from_edge[square.index() as usize][4..8],
            )
        }
        let mut rook_bit_board = *self.board.get_bit_board(friendly_pieces[3]);
        while !rook_bit_board.is_empty() {
            let square = rook_bit_board.pop_square();
            self.gen_directional(
                moves,
                square,
                &friendly_piece_bit_board,
                &enemy_piece_bit_board,
                &DIRECTIONS[0..4],
                &self.precomputed.squares_from_edge[square.index() as usize][0..4],
            )
        }
        let mut queen_bit_board = *self.board.get_bit_board(friendly_pieces[4]);
        while !queen_bit_board.is_empty() {
            let square = queen_bit_board.pop_square();
            self.gen_directional(
                moves,
                square,
                &friendly_piece_bit_board,
                &enemy_piece_bit_board,
                &DIRECTIONS,
                &self.precomputed.squares_from_edge[square.index() as usize],
            )
        }
        let mut king_bit_board = *self.board.get_bit_board(friendly_pieces[5]);
        while !king_bit_board.is_empty() {
            let square = king_bit_board.pop_square();
            self.gen_king(
                moves,
                square,
                &friendly_piece_bit_board,
                &enemy_piece_bit_board,
                &attacked_bit_board,
            )
        }
    }
}
