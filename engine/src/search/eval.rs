use crate::board::{piece::Piece, Board};

use super::eval_data::EvalNumber;

pub struct Eval {}
impl Eval {
    pub fn get_phase(board: &Board) -> EvalNumber {
        let mut phase = 0;
        for piece in Piece::ALL_PIECES {
            match piece {
                Piece::WhitePawn | Piece::BlackPawn => {}
                Piece::WhiteKnight | Piece::BlackKnight => {
                    phase += 1 * board.get_bit_board(piece).count();
                }
                Piece::WhiteBishop | Piece::BlackBishop => {
                    phase += 1 * board.get_bit_board(piece).count();
                }
                Piece::WhiteRook | Piece::BlackRook => {
                    phase += 2 * board.get_bit_board(piece).count();
                }
                Piece::WhiteQueen | Piece::BlackQueen => {
                    phase += 4 * board.get_bit_board(piece).count();
                }
                Piece::WhiteKing | Piece::BlackKing => {}
            }
        }
        phase as EvalNumber
    }

    fn get_piece_value(
        middle_game_piece_square_tables: &[[EvalNumber; 64]; 6],
        end_game_piece_square_tables: &[[EvalNumber; 64]; 6],
        piece_index: usize,
        square_index: usize,
    ) -> (EvalNumber, EvalNumber) {
        let middle_game_piece_score = middle_game_piece_square_tables[piece_index][square_index];
        let end_game_piece_score = end_game_piece_square_tables[piece_index][square_index];

        (middle_game_piece_score, end_game_piece_score)
    }

    fn calculate_score(
        phase: EvalNumber,
        middle_game_score: EvalNumber,
        end_game_score: EvalNumber,
    ) -> EvalNumber {
        let middle_game_phase = phase.min(24);
        let end_game_phase = 24 - middle_game_phase;
        (middle_game_score * middle_game_phase + end_game_score * end_game_phase) / 24
    }

    pub fn evaluate(
        middle_game_piece_square_tables: &[[EvalNumber; 64]; 6],
        end_game_piece_square_tables: &[[EvalNumber; 64]; 6],
        board: &Board,
    ) -> EvalNumber {
        let mut total_middle_game_score = 0;
        let mut total_end_game_score = 0;

        for piece in Piece::WHITE_PIECES {
            let mut bit_board = *board.get_bit_board(piece);
            while bit_board.is_not_empty() {
                let square = bit_board.pop_square();

                let (middle_game_value, end_game_value) = Self::get_piece_value(
                    middle_game_piece_square_tables,
                    end_game_piece_square_tables,
                    piece as usize,
                    square.flip().usize(),
                );

                total_middle_game_score += middle_game_value;
                total_end_game_score += end_game_value;
            }
        }

        for piece in Piece::BLACK_PIECES {
            let mut bit_board = *board.get_bit_board(piece);
            while bit_board.is_not_empty() {
                let square = bit_board.pop_square();

                let (middle_game_value, end_game_value) = Self::get_piece_value(
                    middle_game_piece_square_tables,
                    end_game_piece_square_tables,
                    piece as usize - 6,
                    square.usize(),
                );

                total_middle_game_score -= middle_game_value;
                total_end_game_score -= end_game_value;
            }
        }

        let phase = Self::get_phase(board);
        Self::calculate_score(phase, total_middle_game_score, total_end_game_score)
            * if board.white_to_move { 1 } else { -1 }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::Board,
        search::{eval::Eval, eval_data},
    };

    #[test]
    fn advanced_pawn_worth_more() {
        let starting_rank_pawn = Board::from_fen("7k/8/8/8/8/8/4P3/K7 w - - 0 1");
        let one_step_from_promoting_pawn = Board::from_fen("7k/4P3/8/8/8/8/8/K7 w - - 0 1");
        assert!(
            Eval::evaluate(
                &eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLE,
                &eval_data::END_GAME_PIECE_SQUARE_TABLE,
                &one_step_from_promoting_pawn
            ) > Eval::evaluate(
                &eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLE,
                &eval_data::END_GAME_PIECE_SQUARE_TABLE,
                &starting_rank_pawn
            )
        );
    }

    #[test]
    fn centralised_knight_worth_more() {
        let centralised_knight = Board::from_fen("7k/8/8/4n3/8/8/8/K7 b - - 0 1");
        let knight_on_the_edge = Board::from_fen("7k/8/8/8/7n/8/8/K7 b - - 0 1");
        assert!(
            Eval::evaluate(
                &eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLE,
                &eval_data::END_GAME_PIECE_SQUARE_TABLE,
                &centralised_knight
            ) > Eval::evaluate(
                &eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLE,
                &eval_data::END_GAME_PIECE_SQUARE_TABLE,
                &knight_on_the_edge
            )
        );
    }
}
