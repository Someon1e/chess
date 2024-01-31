mod piece_square_table;

use crate::{
    board::{piece::Piece, square::Square, Board},
    move_generator::{move_data::Move, MoveGenerator},
};

pub struct Engine<'a> {
    move_generator: &'a mut MoveGenerator<'a>
}

impl<'a> Engine<'a> {
    pub fn new(move_generator: &'a mut MoveGenerator<'a>) -> Self {
        Self { move_generator }
    }
    pub fn board_mut(&mut self) -> &mut Board {
        self.move_generator.board_mut()
    }
    pub fn board(&self) -> &Board {
        self.move_generator.board()
    }
    pub fn move_generator(&self) -> &MoveGenerator {
        self.move_generator
    }

    fn get_piece_value(&self, piece: &Piece, piece_square_table_index: usize) -> i32 {
        match piece {
            Piece::WhitePawn => 100 + piece_square_table::PAWN[piece_square_table_index],
            Piece::WhiteKnight => 320 + piece_square_table::KNIGHT[piece_square_table_index],
            Piece::WhiteBishop => 330 + piece_square_table::BISHOP[piece_square_table_index],
            Piece::WhiteRook => 500 + piece_square_table::ROOK[piece_square_table_index],
            Piece::WhiteQueen => 900 + piece_square_table::QUEEN[piece_square_table_index],
            Piece::WhiteKing => 20000 + piece_square_table::KING[piece_square_table_index],

            Piece::BlackPawn => {
                -(100 + piece_square_table::PAWN[piece_square_table::FLIP[piece_square_table_index]])
            }
            Piece::BlackKnight => {
                -(320 + piece_square_table::KNIGHT[piece_square_table::FLIP[piece_square_table_index]])
            }
            Piece::BlackBishop => {
                -(330 + piece_square_table::BISHOP[piece_square_table::FLIP[piece_square_table_index]])
            }
            Piece::BlackRook => {
                -(500 + piece_square_table::ROOK[piece_square_table::FLIP[piece_square_table_index]])
            }
            Piece::BlackQueen => {
                -(900 + piece_square_table::QUEEN[piece_square_table::FLIP[piece_square_table_index]])
            }
            Piece::BlackKing => {
                -(20000 + piece_square_table::KING[piece_square_table::FLIP[piece_square_table_index]])
            }
        }
    }

    fn evaluate(&mut self) -> i32 {
        let mut score = 0;
        for index in 0..64 {
            let square = Square::from_index(index);
            if let Some(piece) = self.board().piece_at(square) {
                score += self.get_piece_value(&piece, index as usize);
            }
        }
        score
    }

    fn guess_move_value(&self, move_data: &Move) -> i32 {
        let capturing = self.board().enemy_piece_at(move_data.to());
        // This won't take into account en passant
        if let Some(capturing) = capturing {
            self.get_piece_value(&capturing, move_data.to().index() as usize)
        } else {
            0
        }
    }

    fn sort_moves(&self, moves: &mut Vec<Move>) {
        moves.sort_by_cached_key(|v| {
            self.guess_move_value(v)
        });
    }

    pub fn negamax(&mut self, depth: u16, mut alpha: i32, beta: i32) -> i32 {
        if depth == 0 {
            if self.board().white_to_move {
                return self.evaluate();
            }
            return -self.evaluate();
        };

        let mut moves = Vec::with_capacity(10);
        self.move_generator.gen(&mut moves);
        self.sort_moves(&mut moves);

        let mut best_score = -i32::MAX;

        for move_data in moves {
            self.board_mut().make_move(&move_data);
            best_score = best_score.max(-self.negamax(depth - 1, -beta, -alpha));
            self.board_mut().unmake_move(&move_data);
            alpha = alpha.max(best_score);
            if alpha >= beta {
                break;
            }
        }
        best_score
    }
    pub fn best_move(&mut self, depth: u16, should_cancel: &mut dyn FnMut() -> bool) -> (Option<Move>, i32) {
        let mut moves = Vec::with_capacity(10);
        self.move_generator.gen(&mut moves);
        self.sort_moves(&mut moves);

        let (mut best_move, mut best_score) = (None, -i32::MAX);
        for move_data in moves {
            if should_cancel() {
                break
            }
            self.board_mut().make_move(&move_data);
            let score = -self.negamax(depth - 1, -i32::MAX, i32::MAX);
            println!("{} {}", move_data, score);
            self.board_mut().unmake_move(&move_data);
            if score > best_score {
                (best_move, best_score) = (Some(move_data), score);
            }
        }
        (best_move, best_score)
    }
    pub fn iterative_deepening(&mut self, depth_completed: &mut dyn FnMut(u16, (Option<Move>, i32)), should_cancel: &mut dyn FnMut() -> bool) {
        let mut depth = 0;
        while !should_cancel() {
            depth += 1;
            let best_move = self.best_move(depth, should_cancel);
            if should_cancel() {
                // Incomplete search.
                // TODO: don't throw away result (need to search best move first)
            } else {
                depth_completed(depth, best_move);
            }
        }
    }
}
