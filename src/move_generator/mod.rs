use crate::board::piece::Piece;
use crate::board::square::{Square, DIRECTIONS};
use crate::board::Board;

pub mod move_data;
mod precomputed;

use move_data::Move;

use self::precomputed::PrecomputedData;

pub struct PsuedoLegalMoveGenerator<'a> {
    board: &'a Board,
    precomputed: PrecomputedData,
}

impl<'a> PsuedoLegalMoveGenerator<'a> {
    pub fn friendly_piece_at(&self, square: Square) -> Option<Piece> {
        if self.board.white_to_move {
            self.board.white_piece_at(square)
        } else {
            self.board.black_piece_at(square)
        }
    }
    pub fn enemy_piece_at(&self, square: Square) -> Option<Piece> {
        if self.board.white_to_move {
            self.board.black_piece_at(square)
        } else {
            self.board.white_piece_at(square)
        }
    }
    fn gen_pawn(&self, moves: &mut Vec<Move>, piece: Piece, square: Square) {
        let move_to = if self.board.white_to_move {
            square.up(1)
        } else {
            square.down(1)
        };
        if self.board.piece_at(move_to).is_none() {
            moves.push(Move::new(piece, square, move_to))
        }
        let attacks = if self.board.white_to_move {
            &self.precomputed.white_pawn_attacks_at_square[square.index() as usize]
        } else {
            &self.precomputed.black_pawn_attacks_at_square[square.index() as usize]
        };
        for attack in attacks {
            // TODO: test if this works
            if self.enemy_piece_at(square).is_some() {
                moves.push(Move::new(piece, square, *attack))
            }
        }
        if self.board.white_to_move {
            // TODO: test if this works
            if square.rank() == 1 {
                moves.push(Move::new(piece, square, square.up(2)))
            }
        } else if square.rank() == 7 {
            moves.push(Move::new(piece, square, square.down(2)))
        }
        // TODO: en passant
    }
    pub fn gen_directional(
        &self,
        moves: &mut Vec<Move>,
        piece: Piece,
        square: Square,
        directions: &[i8],
    ) {
        // TODO: test if this works
        for (direction, distance_from_edge) in directions
            .iter()
            .zip(self.precomputed.squares_from_edge[square.index() as usize])
        {
            for count in 0..distance_from_edge {
                let move_to = square.offset(direction * count);
                if self.friendly_piece_at(move_to).is_some() {
                    break;
                }
                moves.push(Move::new(piece, square, move_to));
                if self.enemy_piece_at(move_to).is_some() {
                    break;
                }
            }
        }
    }
    pub fn gen_king(&self, moves: &mut Vec<Move>, piece: Piece, square: Square) {
        // TODO: test if this works
        for move_to in &self.precomputed.king_moves_at_square[square.index() as usize] {
            if self.friendly_piece_at(*move_to).is_none() {
                moves.push(Move::new(piece, square, *move_to))
            }
        }

        // TODO: castling
    }
    pub fn gen_knight(&self, moves: &mut Vec<Move>, piece: Piece, square: Square) {
        for move_to in &self.precomputed.knight_moves_at_square[square.index() as usize] {
            if self.friendly_piece_at(*move_to).is_none() {
                moves.push(Move::new(piece, square, *move_to))
            }
        }
    }
    pub fn new(board: &'a Board) -> Self {
        let precomputed = PrecomputedData::compute();
        Self {
            board,
            precomputed,
        }
    }
    pub fn gen(&self, moves: &mut Vec<Move>) {
        for index in 0..64 {
            let square = Square::from_index(index);
            let piece = if self.board.white_to_move {
                self.board.white_piece_at(square)
            } else {
                self.board.black_piece_at(square)
            };
            if let Some(piece) = piece {
                match piece {
                    Piece::WhitePawn | Piece::BlackPawn => self.gen_pawn(moves, piece, square),
                    Piece::WhiteKnight | Piece::BlackKnight => {
                        self.gen_knight(moves, piece, square)
                    }
                    Piece::WhiteBishop | Piece::BlackBishop => {
                        self.gen_directional(moves, piece, square, &DIRECTIONS[4..8])
                    }
                    Piece::WhiteRook | Piece::BlackRook => {
                        self.gen_directional(moves, piece, square, &DIRECTIONS[0..4])
                    }
                    Piece::WhiteQueen | Piece::BlackQueen => {
                        self.gen_directional(moves, piece, square, &DIRECTIONS)
                    }
                    Piece::WhiteKing | Piece::BlackKing => self.gen_king(moves, piece, square),
                }
            }
        }
    }
}
