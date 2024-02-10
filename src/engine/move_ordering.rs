use crate::{
    board::bit_board::BitBoard,
    move_generator::{move_data::Flag, MoveGenerator},
};

use super::{encoded_move::EncodedMove, eval::Eval, Engine};

pub struct MoveOrderer {}
impl MoveOrderer {
    pub fn guess_move_value(
        engine: &Engine,
        enemy_pawn_attacks: &BitBoard,
        move_data: &EncodedMove,
    ) -> i32 {
        let mut score = 0;
        match move_data.flag() {
            Flag::EnPassant => score += 0,
            Flag::PawnTwoUp => score += 0,
            Flag::BishopPromotion => score += 300,
            Flag::KnightPromotion => score += 400,
            Flag::RookPromotion => score += 300,
            Flag::QueenPromotion => score += 800,
            Flag::Castle => score += 0,
            Flag::None => score += 0,
        }

        let moving_from = move_data.from();
        let moving_to = move_data.to();

        if enemy_pawn_attacks.get(&moving_to) {
            score -= 50;
        }

        let moving_piece = engine.board.friendly_piece_at(moving_from).unwrap();
        let (moving_from_middle_game_value, moving_from_end_game_value) = {
            if engine.board.white_to_move {
                Eval::get_white_piece_value(moving_piece, moving_from)
            } else {
                Eval::get_black_piece_value(moving_piece, moving_from)
            }
        };

        let phase = Eval::get_phase(engine);

        // This won't take into account en passant
        if let Some(capturing) = engine.board.enemy_piece_at(moving_to) {
            let (capturing_middle_game_value, capturing_end_game_value) = {
                if engine.board.white_to_move {
                    Eval::get_black_piece_value(capturing, moving_to)
                } else {
                    Eval::get_white_piece_value(capturing, moving_to)
                }
            };

            score += Eval::calculate_score(
                phase,
                capturing_middle_game_value - moving_from_middle_game_value,
                capturing_end_game_value - moving_from_end_game_value,
            );
        } else {
            let (moving_to_middle_game_value, moving_to_end_game_value) = {
                if engine.board.white_to_move {
                    Eval::get_white_piece_value(moving_piece, moving_to)
                } else {
                    Eval::get_black_piece_value(moving_piece, moving_to)
                }
            };
            score += Eval::calculate_score(
                phase,
                moving_to_middle_game_value - moving_from_middle_game_value,
                moving_to_end_game_value - moving_from_end_game_value,
            );
        }
        score
    }

    pub fn get_sorted_moves(
        engine: &Engine,
        move_generator: &MoveGenerator,
        best_move: &EncodedMove,
    ) -> ([EncodedMove; 218], usize) {
        let mut moves = [EncodedMove::NONE; 218];
        let mut index = 0;
        move_generator.gen(
            &mut |move_data| {
                moves[index] = EncodedMove::new(move_data);
                index += 1
            },
            false,
        );

        let actual_moves = &mut moves[0..index];
        // Best moves will be at the back, so iterate in reverse later.
        actual_moves.sort_by_cached_key(|move_data| {
            if *move_data == *best_move {
                return 10000;
            }
            Self::guess_move_value(engine, &move_generator.enemy_pawn_attacks(), move_data)
        });

        (moves, index)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        board::{square::Square, Board},
        engine::{encoded_move::EncodedMove, move_ordering::MoveOrderer, Engine},
        move_generator::{
            move_data::{Flag, Move},
            MoveGenerator,
        },
    };

    #[test]
    fn move_ordering_works() {
        let mut board = Board::from_fen("8/P6p/6r1/1q1n4/2P3R1/8/2K2k2/8 w - - 0 1");
        let move_generator = MoveGenerator::new(&board);
        let (moves, move_count) = MoveOrderer::get_sorted_moves(
            &Engine::new(&mut board),
            &move_generator,
            &EncodedMove::NONE,
        );
        for index in (0..move_count).rev() {
            let move_data = moves[index];
            println!("{move_data}");
        }
        assert!(
            moves[move_count - 1].decode()
                == Move {
                    from: Square::from_notation("c4"),
                    to: Square::from_notation("b5"),
                    flag: Flag::None
                }
        );
        assert!(
            moves[move_count - 2].decode()
                == Move {
                    from: Square::from_notation("a7"),
                    to: Square::from_notation("a8"),
                    flag: Flag::QueenPromotion
                }
        )
    }
}
