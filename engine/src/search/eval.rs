use crate::board::{piece::Piece, Board};

use super::eval_data::{EvalNumber, PieceSquareTable};

/// Evaluation functions.
pub struct Eval;
impl Eval {
    /// Gets the phase.
    #[must_use]
    pub fn get_phase(board: &Board, phases: &[EvalNumber; 5]) -> EvalNumber {
        let mut phase = 0;

        phase += phases[0]
            * ((*board.get_bit_board(Piece::WhitePawn) | *board.get_bit_board(Piece::BlackPawn))
                .count()) as EvalNumber;

        phase += phases[1]
            * ((*board.get_bit_board(Piece::WhiteKnight)
                | *board.get_bit_board(Piece::BlackKnight))
            .count()) as EvalNumber;

        phase += phases[2]
            * ((*board.get_bit_board(Piece::WhiteBishop)
                | *board.get_bit_board(Piece::BlackBishop))
            .count()) as EvalNumber;

        phase += phases[3]
            * ((*board.get_bit_board(Piece::WhiteRook) | *board.get_bit_board(Piece::BlackRook))
                .count()) as EvalNumber;

        phase += phases[4]
            * ((*board.get_bit_board(Piece::WhiteQueen) | *board.get_bit_board(Piece::BlackQueen))
                .count()) as EvalNumber;

        phase
    }

    #[must_use]
    const fn get_piece_value(
        middle_game_piece_square_tables: &PieceSquareTable,
        end_game_piece_square_tables: &PieceSquareTable,
        piece_index: usize,
        square_index: usize,
    ) -> (i16, i16) {
        let middle_game_piece_score =
            middle_game_piece_square_tables[piece_index * 64 + square_index];
        let end_game_piece_score = end_game_piece_square_tables[piece_index * 64 + square_index];

        (middle_game_piece_score, end_game_piece_score)
    }

    #[must_use]
    fn calculate_score(
        phase: EvalNumber,
        total_phase: EvalNumber,
        middle_game_score: EvalNumber,
        end_game_score: EvalNumber,
    ) -> EvalNumber {
        let middle_game_phase = phase.min(total_phase);
        let end_game_phase = total_phase - middle_game_phase;
        (middle_game_score * middle_game_phase + end_game_score * end_game_phase) / total_phase
    }

    /// Returns an estimated score of the position for the side playing.
    #[must_use]
    pub fn evaluate(
        middle_game_piece_square_tables: &PieceSquareTable,
        end_game_piece_square_tables: &PieceSquareTable,
        phases: &[EvalNumber; 5],
        board: &Board,
    ) -> EvalNumber {
        #[rustfmt::skip]
        let total_phase = {
            phases[0] * 16
            + phases[1] * 4
            + phases[2] * 4
            + phases[3] * 4
            + phases[4] * 2
        };

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

                total_middle_game_score += i32::from(middle_game_value);
                total_end_game_score += i32::from(end_game_value);
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

                total_middle_game_score -= i32::from(middle_game_value);
                total_end_game_score -= i32::from(end_game_value);
            }
        }

        let phase = Self::get_phase(board, phases);
        Self::calculate_score(
            phase,
            total_phase,
            total_middle_game_score,
            total_end_game_score,
        ) * if board.white_to_move { 1 } else { -1 }
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
                &eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLES,
                &eval_data::END_GAME_PIECE_SQUARE_TABLES,
                &eval_data::PHASES,
                &one_step_from_promoting_pawn
            ) > Eval::evaluate(
                &eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLES,
                &eval_data::END_GAME_PIECE_SQUARE_TABLES,
                &eval_data::PHASES,
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
                &eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLES,
                &eval_data::END_GAME_PIECE_SQUARE_TABLES,
                &eval_data::PHASES,
                &centralised_knight
            ) > Eval::evaluate(
                &eval_data::MIDDLE_GAME_PIECE_SQUARE_TABLES,
                &eval_data::END_GAME_PIECE_SQUARE_TABLES,
                &eval_data::PHASES,
                &knight_on_the_edge
            )
        );
    }
}
