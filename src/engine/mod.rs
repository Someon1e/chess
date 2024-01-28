use crate::{
    board::{piece::Piece, square::Square, Board},
    move_generator::{move_data::Move, PsuedoLegalMoveGenerator},
};

pub struct Engine<'a> {
    move_generator: &'a mut PsuedoLegalMoveGenerator<'a>,
}

impl<'a> Engine<'a> {
    pub fn new(move_generator: &'a mut PsuedoLegalMoveGenerator<'a>) -> Self {
        Self { move_generator }
    }
    pub fn board(&mut self) -> &mut Board {
        self.move_generator.board()
    }
    pub fn move_generator(&self) -> &PsuedoLegalMoveGenerator {
        self.move_generator
    }

    const PAWN_PIECE_SQUARE_TABLE: [i32; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 50, 50, 50, 50, 50, 50, 50, 50, 10, 10, 20, 30, 30, 20, 10, 10, 5,
        5, 10, 25, 25, 10, 5, 5, 0, 0, 0, 20, 20, 0, 0, 0, 5, -5, -10, 0, 0, -10, -5, 5, 5, 10, 10,
        -20, -20, 10, 10, 5, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    const KNIGHT_PIECE_SQUARE_TABLE: [i32; 64] = [
        -50, -40, -30, -30, -30, -30, -40, -50, -40, -20, 0, 0, 0, 0, -20, -40, -30, 0, 10, 15, 15,
        10, 0, -30, -30, 5, 15, 20, 20, 15, 5, -30, -30, 0, 15, 20, 20, 15, 0, -30, -30, 5, 10, 15,
        15, 10, 5, -30, -40, -20, 0, 5, 5, 0, -20, -40, -50, -40, -30, -30, -30, -30, -40, -50,
    ];

    const BISHOP_PIECE_SQUARE_TABLE: [i32; 64] = [
        -20, -10, -10, -10, -10, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 10, 10, 5,
        0, -10, -10, 5, 5, 10, 10, 5, 5, -10, -10, 0, 10, 10, 10, 10, 0, -10, -10, 10, 10, 10, 10,
        10, 10, -10, -10, 5, 0, 0, 0, 0, 5, -10, -20, -10, -10, -10, -10, -10, -10, -20,
    ];

    const ROOK_PIECE_SQUARE_TABLE: [i32; 64] = [
        0, 0, 0, 0, 0, 0, 0, 0, 5, 10, 10, 10, 10, 10, 10, 5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0,
        0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0, -5, -5, 0, 0, 0, 0, 0, 0,
        -5, 0, 0, 0, 5, 5, 0, 0, 0,
    ];

    const QUEEN_PIECE_SQUARE_TABLE: [i32; 64] = [
        -20, -10, -10, -5, -5, -10, -10, -20, -10, 0, 0, 0, 0, 0, 0, -10, -10, 0, 5, 5, 5, 5, 0,
        -10, -5, 0, 5, 5, 5, 5, 0, -5, 0, 0, 5, 5, 5, 5, 0, -5, -10, 5, 5, 5, 5, 5, 0, -10, -10, 0,
        5, 0, 0, 0, 0, -10, -20, -10, -10, -5, -5, -10, -10, -20,
    ];

    const KING_PIECE_SQUARE_TABLE: [i32; 64] = [
        -30, -40, -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -30, -40,
        -40, -50, -50, -40, -40, -30, -30, -40, -40, -50, -50, -40, -40, -30, -20, -30, -30, -40,
        -40, -30, -30, -20, -10, -20, -20, -20, -20, -20, -20, -10, 20, 20, 0, 0, 0, 0, 20, 20, 20,
        30, 10, 0, 0, 10, 30, 20,
    ];

    pub const FLIP: [usize; 64] = [
        56, 57, 58, 59, 60, 61, 62, 63, 48, 49, 50, 51, 52, 53, 54, 55, 40, 41, 42, 43, 44, 45, 46,
        47, 32, 33, 34, 35, 36, 37, 38, 39, 24, 25, 26, 27, 28, 29, 30, 31, 16, 17, 18, 19, 20, 21,
        22, 23, 8, 9, 10, 11, 12, 13, 14, 15, 0, 1, 2, 3, 4, 5, 6, 7,
    ];

    pub fn evaluate(&mut self) -> i32 {
        let mut score = 0;
        for index in 0..64 {
            let square = Square::from_index(index);
            if let Some(piece) = self.board().piece_at(square) {
                score += match piece {
                    Piece::WhitePawn => 100 + Self::PAWN_PIECE_SQUARE_TABLE[index as usize],
                    Piece::WhiteKnight => 320 + Self::KNIGHT_PIECE_SQUARE_TABLE[index as usize],
                    Piece::WhiteBishop => 330 + Self::BISHOP_PIECE_SQUARE_TABLE[index as usize],
                    Piece::WhiteRook => 500 + Self::ROOK_PIECE_SQUARE_TABLE[index as usize],
                    Piece::WhiteQueen => 900 + Self::QUEEN_PIECE_SQUARE_TABLE[index as usize],
                    Piece::WhiteKing => 20000 + Self::KING_PIECE_SQUARE_TABLE[index as usize],

                    Piece::BlackPawn => {
                        -(100 + Self::PAWN_PIECE_SQUARE_TABLE[Self::FLIP[index as usize]])
                    }
                    Piece::BlackKnight => {
                        -(320 + Self::KNIGHT_PIECE_SQUARE_TABLE[Self::FLIP[index as usize]])
                    }
                    Piece::BlackBishop => {
                        -(330 + Self::BISHOP_PIECE_SQUARE_TABLE[Self::FLIP[index as usize]])
                    }
                    Piece::BlackRook => {
                        -(500 + Self::ROOK_PIECE_SQUARE_TABLE[Self::FLIP[index as usize]])
                    }
                    Piece::BlackQueen => {
                        -(900 + Self::QUEEN_PIECE_SQUARE_TABLE[Self::FLIP[index as usize]])
                    }
                    Piece::BlackKing => {
                        -(20000 + Self::KING_PIECE_SQUARE_TABLE[Self::FLIP[index as usize]])
                    }
                };
            }
        }
        score
    }
    pub fn negamax(&mut self, depth: u16, mut alpha: i32, beta: i32) -> i32 {
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
            best_score = best_score.max(-self.negamax(depth - 1, -beta, -alpha));
            self.board().unmake_move(&move_data);
            alpha = alpha.max(best_score);
            if alpha >= beta {
                break;
            }
        }
        best_score
    }
    pub fn best_move(&mut self, depth: u16) -> (Option<Move>, i32) {
        let mut moves = vec![];
        self.move_generator.gen(&mut moves);

        let (mut best_move, mut best_score) = (None, i32::MIN);
        for move_data in moves {
            self.board().make_move(&move_data);
            let score = -self.negamax(depth - 1, i32::MIN, i32::MAX);
            println!("{} {}", move_data, score);
            self.board().unmake_move(&move_data);
            if score > best_score {
                (best_move, best_score) = (Some(move_data), score);
            }
        }
        (best_move, best_score)
    }
    pub fn can_capture_king(&mut self) -> bool {
        let mut moves = vec![];
        self.move_generator.gen(&mut moves);

        for move_data in moves {
            match move_data.captured() {
                Some(Piece::WhiteKing) | Some(Piece::BlackKing) => return true,
                _ => {}
            }
        }
        false
    }
}
