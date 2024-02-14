use crate::board::{piece::Piece, square::Square};

use super::{eval_data, Engine};

pub struct Eval {}
impl Eval {
    pub fn get_phase(engine: &Engine) -> i32 {
        let mut phase = 0;
        for piece in Piece::ALL_PIECES {
            let bit_board = *engine.board.get_bit_board(piece);
            let piece_index = piece as usize;
            phase += bit_board.count() as i32 * eval_data::PIECE_PHASES[piece_index]
        }
        phase
    }

    fn get_piece_value(piece_index: usize, square_index: usize) -> (i32, i32) {
        let middle_game_piece_score =
            eval_data::MIDDLE_GAME_PIECE_VALUES_WITH_SQUARE[piece_index][square_index];
        let end_game_piece_score =
            eval_data::END_GAME_PIECE_VALUES_WITH_SQUARE[piece_index][square_index];

        (middle_game_piece_score, end_game_piece_score)
    }
    pub fn get_black_piece_value(piece: Piece, square: Square) -> (i32, i32) {
        Self::get_piece_value(piece as usize - 6, square.usize())
    }
    pub fn get_white_piece_value(piece: Piece, square: Square) -> (i32, i32) {
        Self::get_piece_value(piece as usize, square.flip().usize())
    }

    pub fn calculate_score(phase: i32, middle_game_score: i32, end_game_score: i32) -> i32 {
        let middle_game_phase = phase.min(24);
        let end_game_phase = 24 - middle_game_phase;
        (middle_game_score * middle_game_phase + end_game_score * end_game_phase) / 24
    }

    pub fn evaluate(engine: &Engine) -> i32 {
        let mut middle_game_score_white = 0;
        let mut end_game_score_white = 0;

        for piece in Piece::WHITE_PIECES {
            let mut bit_board = *engine.board.get_bit_board(piece);
            while bit_board.is_not_empty() {
                let square = bit_board.pop_square();

                let (middle_game_value, end_game_value) =
                    Self::get_white_piece_value(piece, square);

                middle_game_score_white += middle_game_value;
                end_game_score_white += end_game_value;
            }
        }

        let mut middle_game_score_black = 0;
        let mut end_game_score_black = 0;

        for piece in Piece::BLACK_PIECES {
            let mut bit_board = *engine.board.get_bit_board(piece);
            while bit_board.is_not_empty() {
                let square = bit_board.pop_square();

                let (middle_game_value, end_game_value) =
                    Self::get_black_piece_value(piece, square);

                middle_game_score_black += middle_game_value;
                end_game_score_black += end_game_value;
            }
        }

        let phase = Self::get_phase(engine);
        Self::calculate_score(
            phase,
            middle_game_score_white - middle_game_score_black,
            end_game_score_white - end_game_score_black,
        ) * if engine.board.white_to_move { 1 } else { -1 }
    }
}
