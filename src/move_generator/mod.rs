use crate::board::piece;
use crate::board::square::Square;
use crate::board::Board;

pub mod move_data;
mod precomputed;

use move_data::Move;

use self::precomputed::PrecomputedData;

pub struct PsuedoLegalMoveGenerator<'a> {
    board: &'a Board,
    moves: Vec<Move>,
    precomputed: PrecomputedData,
}

impl<'a> PsuedoLegalMoveGenerator<'a> {
    fn gen_pawn(&mut self, square: Square) {
        let move_to = if self.board.white_to_move {
            square.up(1)
        } else {
            square.down(1)
        };
        if self.board.piece_at(move_to).is_none() {
            self.moves.push(Move::new(square, move_to))
        }
        let attacks = if self.board.white_to_move {
            &self.precomputed.white_pawn_attacks_at_square[square.index() as usize]
        } else {
            &self.precomputed.black_pawn_attacks_at_square[square.index() as usize]
        };
        for attack in attacks {
            // TODO: test if this works
            if if self.board.white_to_move {
                self.board.black_piece_at(*attack)
            } else {
                self.board.white_piece_at(*attack)
            }
            .is_some()
            {
                self.moves.push(Move::new(square, *attack))
            }
        }
        if self.board.white_to_move {
            // TODO: test if this works
            if square.rank() == 1 {
                self.moves.push(Move::new(square, square.up(2)))
            }
        } else if square.rank() == 7 {
            self.moves.push(Move::new(square, square.down(2)))
        }
        // TODO: en passant
    }
    pub fn gen_king(&mut self, square: Square) {
        // TODO: test if this works
        for move_to in &self.precomputed.king_moves_at_square[square.index() as usize] {
            self.moves.push(Move::new(square, *move_to))
        }

        // TODO: castling
    }
    pub fn new(board: &'a Board) -> Self {
        let moves = Vec::with_capacity(1);
        let precomputed = PrecomputedData::compute();
        Self {
            board,
            moves,
            precomputed,
        }
    }
    pub fn clear(&mut self) {
        self.moves.clear()
    }
    pub fn gen(&mut self) -> &Vec<Move> {
        for index in 0..64 {
            let square = Square::from_index(index);
            let piece = if self.board.white_to_move {
                self.board.white_piece_at(square)
            } else {
                self.board.black_piece_at(square)
            };
            if let Some(piece) = piece {
                match piece {
                    piece::WHITE_PAWN => self.gen_pawn(square),
                    piece::BLACK_PAWN => self.gen_pawn(square),
                    piece::WHITE_KING => self.gen_king(square),
                    piece::BLACK_KING => self.gen_king(square),
                    _ => {}
                }
            }
        }
        &self.moves
    }
}
