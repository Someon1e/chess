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
    fn gen_promotions(
        &self,
        moves: &mut Vec<Move>,
        from: Square,
        to: Square,
        captured: Option<Piece>,
    ) {
        for promotion in Flag::PROMOTIONS {
            moves.push(Move::with_flag(from, to, captured, promotion));
        }
    }
    fn pawn_attack_bit_board(&self, square: Square, white: bool) -> BitBoard {
        if white {
            self.precomputed.white_pawn_attacks_at_square[square.index() as usize]
        } else {
            self.precomputed.black_pawn_attacks_at_square[square.index() as usize]
        }
    }
    pub fn gen_pawn(&self, moves: &mut Vec<Move>, square: Square, friendly_pieces: &BitBoard) {
        let mut attacks = self.pawn_attack_bit_board(square, self.board.white_to_move);
        let pawn_up = if self.board.white_to_move { 1 } else { -1 };

        while !attacks.is_empty() {
            let attack = attacks.pop_square();
            if let Some(enemy) = self.board.enemy_piece_at(attack) {
                if self.is_promotion_rank(attack.rank()) {
                    self.gen_promotions(moves, square, attack, Some(enemy))
                } else {
                    moves.push(Move::new(square, attack, Some(enemy)));
                }
            } else if let Some(en_passant_square) = self.board.game_state.en_passant_square {
                if en_passant_square == attack {
                    let enemy = self
                        .board
                        .enemy_piece_at(en_passant_square.down(pawn_up))
                        .unwrap(); // TODO: make this only check for pawns
                    moves.push(Move::with_flag(
                        square,
                        attack,
                        Some(enemy),
                        Flag::EnPassant,
                    ));
                }
            }
        }

        if !friendly_pieces.get(&square.up(pawn_up))
            && self.board.enemy_piece_at(square.up(pawn_up)).is_none()
        {
            if self.is_promotion_rank(square.up(pawn_up).rank()) {
                self.gen_promotions(moves, square, square.up(pawn_up), None)
            } else {
                moves.push(Move::new(square, square.up(pawn_up), None));
                let is_starting_rank = if self.board.white_to_move {
                    square.rank() == 1
                } else {
                    square.rank() == 6
                };
                if is_starting_rank
                    && !friendly_pieces.get(&square.up(pawn_up * 2))
                    && self.board.enemy_piece_at(square.up(pawn_up * 2)).is_none()
                {
                    moves.push(Move::with_flag(
                        square,
                        square.up(pawn_up * 2),
                        None,
                        Flag::PawnTwoUp,
                    ))
                }
            }
        }
    }
    fn directional_attack_bit_board(
        &self,
        square: Square,
        friendly_pieces: &BitBoard,

        directions: &[i8],
        squares_from_edge: &[i8],
    ) -> BitBoard {
        let mut attacked = BitBoard::empty();
        for (direction, distance_from_edge) in directions.iter().zip(squares_from_edge) {
            for count in 1..=*distance_from_edge {
                let move_to = square.offset(direction * count);
                attacked = attacked | move_to.bitboard();
                if friendly_pieces.get(&move_to) || self.board.enemy_piece_at(move_to).is_some() {
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
        friendly_pieces: &BitBoard,

        directions: &[i8],
        squares_from_edge: &[i8],
    ) {
        for (direction, distance_from_edge) in directions.iter().zip(squares_from_edge) {
            for count in 1..=*distance_from_edge {
                let move_to = square.offset(direction * count);
                if friendly_pieces.get(&move_to) {
                    break;
                }
                let enemy = self.board.enemy_piece_at(move_to);
                moves.push(Move::new(square, move_to, enemy));
                if enemy.is_some() {
                    break;
                }
            }
        }
    }
    pub fn gen_bishop(&self, moves: &mut Vec<Move>, square: Square, friendly_pieces: &BitBoard) {
        self.gen_directional(
            moves,
            square,
            friendly_pieces,
            &DIRECTIONS[4..8],
            &self.precomputed.squares_from_edge[square.index() as usize][4..8],
        )
    }
    pub fn gen_rook(&self, moves: &mut Vec<Move>, square: Square, friendly_pieces: &BitBoard) {
        self.gen_directional(
            moves,
            square,
            friendly_pieces,
            &DIRECTIONS[0..4],
            &self.precomputed.squares_from_edge[square.index() as usize][0..4],
        )
    }
    pub fn gen_queen(&self, moves: &mut Vec<Move>, square: Square, friendly_pieces: &BitBoard) {
        self.gen_directional(
            moves,
            square,
            friendly_pieces,
            &DIRECTIONS,
            &self.precomputed.squares_from_edge[square.index() as usize],
        )
    }
    fn knight_attack_bit_board(&self, square: Square) -> BitBoard {
        self.precomputed.knight_moves_at_square[square.index() as usize]
    }
    pub fn gen_knight(&self, moves: &mut Vec<Move>, square: Square, friendly_pieces: &BitBoard) {
        let mut knight_moves = self.knight_attack_bit_board(square) & friendly_pieces.not();
        while !knight_moves.is_empty() {
            let move_to = knight_moves.pop_square();
            let enemy = self.board.enemy_piece_at(move_to);
            moves.push(Move::new(square, move_to, enemy))
        }
    }

    fn king_attack_bit_board(&self, square: Square) -> BitBoard {
        self.precomputed.king_moves_at_square[square.index() as usize]
    }
    pub fn gen_king(
        &self,
        moves: &mut Vec<Move>,
        square: Square,
        friendly_pieces: &BitBoard,
        attacked_bit_board: &BitBoard,
    ) {
        let mut king_moves =
            self.king_attack_bit_board(square) & friendly_pieces.not() & attacked_bit_board.not();
        while !king_moves.is_empty() {
            let move_to = king_moves.pop_square();
            let enemy = self.board.enemy_piece_at(move_to);
            moves.push(Move::new(square, move_to, enemy));
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
        if king_side {
            let move_to = square.right(2);
            if self.board.piece_at(square.right(1)).is_none()
                && self.board.piece_at(move_to).is_none()
            {
                moves.push(Move::with_flag(square, move_to, None, Flag::Castle))
            }
        }
        if queen_side {
            let move_to = square.left(2);
            if self.board.piece_at(square.left(1)).is_none()
                && self.board.piece_at(move_to).is_none()
                && self.board.piece_at(square.left(3)).is_none()
            {
                moves.push(Move::with_flag(square, move_to, None, Flag::Castle))
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
                                &DIRECTIONS[4..8],
                                &self.precomputed.squares_from_edge[square.index() as usize][4..8],
                            ),
                        Piece::WhiteRook | Piece::BlackRook => self.directional_attack_bit_board(
                            square,
                            &friendly_piece_bit_board,
                            &DIRECTIONS[0..4],
                            &self.precomputed.squares_from_edge[square.index() as usize][0..4],
                        ),
                        Piece::WhiteQueen | Piece::BlackQueen => self.directional_attack_bit_board(
                            square,
                            &friendly_piece_bit_board,
                            &DIRECTIONS,
                            &self.precomputed.squares_from_edge[square.index() as usize],
                        ),
                        Piece::WhiteKing | Piece::BlackKing => self.king_attack_bit_board(square),
                    }
            }
        }

        for piece in friendly_pieces {
            let mut bit_board = *self.board.get_bit_board(piece);
            while !bit_board.is_empty() {
                let square = bit_board.pop_square();
                match piece {
                    Piece::WhitePawn | Piece::BlackPawn => {
                        self.gen_pawn(moves, square, &friendly_piece_bit_board)
                    }
                    Piece::WhiteKnight | Piece::BlackKnight => {
                        self.gen_knight(moves, square, &friendly_piece_bit_board)
                    }
                    Piece::WhiteBishop | Piece::BlackBishop => {
                        self.gen_bishop(moves, square, &friendly_piece_bit_board)
                    }
                    Piece::WhiteRook | Piece::BlackRook => {
                        self.gen_rook(moves, square, &friendly_piece_bit_board)
                    }
                    Piece::WhiteQueen | Piece::BlackQueen => {
                        self.gen_queen(moves, square, &friendly_piece_bit_board)
                    }
                    Piece::WhiteKing | Piece::BlackKing => self.gen_king(
                        moves,
                        square,
                        &friendly_piece_bit_board,
                        &attacked_bit_board,
                    ),
                }
            }
        }
    }
}
