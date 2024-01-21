use crate::{
    board::{piece::Piece, square::Square, Board},
    move_generator::{PsuedoLegalMoveGenerator, move_data::Move},
};

pub struct Engine<'a> {
    move_generator: &'a mut PsuedoLegalMoveGenerator<'a>,
}

impl<'a> Engine<'a> {
    pub fn new(move_generator: &'a mut PsuedoLegalMoveGenerator<'a>) -> Self {
        Self {
            move_generator,
        }
    }
    pub fn board(&mut self) -> &mut Board {
        self.move_generator.board()
    }
    pub fn move_generator(&self) -> &PsuedoLegalMoveGenerator {
        &self.move_generator
    }
    pub fn evaluate(&mut self) -> i32 {
        let mut score = 0;
        for index in 0..64 {
            let square = Square::from_index(index);
            if let Some(piece) = self.board().piece_at(square) {
                score += match piece {
                    Piece::WhitePawn => 100,
                    Piece::WhiteKnight => 300,
                    Piece::WhiteBishop => 350,
                    Piece::WhiteRook => 500,
                    Piece::WhiteQueen => 900,
                    Piece::WhiteKing => 100000,

                    Piece::BlackPawn => -100,
                    Piece::BlackKnight => -300,
                    Piece::BlackBishop => -350,
                    Piece::BlackRook => -500,
                    Piece::BlackQueen => -900,
                    Piece::BlackKing => -100000,
                }
            }
        }
        score
    }
    pub fn nega_max(&mut self, depth: u16) -> i32 {
        if depth == 0 {
            if self.board().white_to_move {
                return self.evaluate();
            }
            return -self.evaluate();
        };

        let mut moves = vec![];
        self.move_generator.gen(&mut moves);

        let mut best_score = i32::MIN;
        for move_data in moves {
            self.board().make_move(&move_data);
            let score = -self.nega_max(depth - 1);
            if score > best_score {
                best_score = score;
            }
            self.board().unmake_move(&move_data);
        }
        return best_score;
    }
    pub fn best_move(&mut self, depth: u16) -> (Option<Move>, i32) {
        let mut moves = vec![];
        self.move_generator.gen(&mut moves);

        let (mut best_move, mut best_score) = (None, i32::MIN);
        for move_data in moves {
            self.board().make_move(&move_data);
            let score = self.nega_max(depth - 1);
            self.board().unmake_move(&move_data);
            if score > best_score {
                (best_move, best_score) = (Some(move_data), score);
            }
        }
        (best_move, best_score)
    }
}
