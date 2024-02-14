use crate::{
    board::bit_board::BitBoard,
    move_generator::{
        move_data::{Flag, Move},
        MoveGenerator,
    },
};

use super::{encoded_move::EncodedMove, eval::Eval, Engine};

#[derive(Clone, Copy)]
pub struct MoveGuess {
    guess: i32,
    pub move_data: EncodedMove,
}

const MAX_LEGAL_MOVES: usize = 218;
const MAX_CAPTURES: usize = 74;

pub struct MoveOrderer {}
impl MoveOrderer {
    fn guess_move_value(engine: &Engine, enemy_pawn_attacks: &BitBoard, move_data: &Move) -> i32 {
        let mut score = 0;
        match move_data.flag {
            Flag::EnPassant => return 0,
            Flag::PawnTwoUp => return 5,
            Flag::Castle => return 20,

            Flag::BishopPromotion => score += 300,
            Flag::KnightPromotion => score += 400,
            Flag::RookPromotion => score += 300,
            Flag::QueenPromotion => score += 800,

            Flag::None => {}
        }

        let moving_from = move_data.from;
        let moving_to = move_data.to;

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

    pub fn put_highest_guessed_move_on_top(
        move_guesses: &mut [MoveGuess],
        last_unsorted_index: usize,
    ) -> MoveGuess {
        let (mut index_of_highest_move, mut highest_guess) =
            (last_unsorted_index, move_guesses[last_unsorted_index].guess);
        for index in 0..last_unsorted_index {
            let guess = move_guesses[index].guess;
            if guess > highest_guess {
                highest_guess = guess;
                index_of_highest_move = index;
            }
        }
        if index_of_highest_move != last_unsorted_index {
            move_guesses.swap(index_of_highest_move, last_unsorted_index);
        }
        move_guesses[last_unsorted_index]
    }

    fn guess_capture_value(engine: &Engine, move_data: &Move) -> i32 {
        let flag_score = match move_data.flag {
            Flag::EnPassant => return 0,

            Flag::BishopPromotion => 400,
            Flag::KnightPromotion => 500,
            Flag::RookPromotion => 400,
            Flag::QueenPromotion => 900,

            Flag::None => 0,

            _ => unreachable!()
        };

        let moving_from = move_data.from;
        let moving_to = move_data.to;

        let moving_piece = engine.board.friendly_piece_at(moving_from).unwrap();
        let (moving_from_middle_game_value, moving_from_end_game_value) = {
            if engine.board.white_to_move {
                Eval::get_white_piece_value(moving_piece, moving_from)
            } else {
                Eval::get_black_piece_value(moving_piece, moving_from)
            }
        };

        let capturing = engine.board.enemy_piece_at(moving_to).unwrap();

        let (capturing_middle_game_value, capturing_end_game_value) = {
            if engine.board.white_to_move {
                Eval::get_black_piece_value(capturing, moving_to)
            } else {
                Eval::get_white_piece_value(capturing, moving_to)
            }
        };

        flag_score + Eval::calculate_score(
            Eval::get_phase(engine),
            capturing_middle_game_value - moving_from_middle_game_value,
            capturing_end_game_value - moving_from_end_game_value,
        )
    }

    pub fn get_sorted_moves_captures_only(
        engine: &Engine,
        move_generator: &MoveGenerator,
    ) -> ([MoveGuess; MAX_CAPTURES], usize) {
        let mut move_guesses = [MoveGuess {
            move_data: EncodedMove::NONE,
            guess: 0,
        }; MAX_CAPTURES];
        let mut index = 0;
        move_generator.gen(
            &mut |move_data| {
                let encoded = EncodedMove::new(move_data);
                move_guesses[index] = MoveGuess {
                    move_data: encoded,
                    guess: Self::guess_capture_value(engine, &move_data),
                };
                index += 1
            },
            true,
        );

        (move_guesses, index)
    }

    pub fn get_sorted_moves(
        engine: &Engine,
        move_generator: &MoveGenerator,
        best_move: &EncodedMove,
    ) -> ([MoveGuess; MAX_LEGAL_MOVES], usize) {
        let mut move_guesses = [MoveGuess {
            move_data: EncodedMove::NONE,
            guess: 0,
        }; MAX_LEGAL_MOVES];
        let mut index = 0;
        move_generator.gen(
            &mut |move_data| {
                let encoded = EncodedMove::new(move_data);
                move_guesses[index] = MoveGuess {
                    move_data: encoded,
                    guess: if encoded == *best_move {
                        10000
                    } else {
                        Self::guess_move_value(
                            engine,
                            &move_generator.enemy_pawn_attacks(),
                            &move_data,
                        )
                    },
                };
                index += 1
            },
            false,
        );

        (move_guesses, index)
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
        let (mut move_guesses, move_count) = MoveOrderer::get_sorted_moves(
            &Engine::new(&mut board),
            &move_generator,
            &EncodedMove::NONE,
        );
        let mut index = move_count;
        let mut next_move = || {
            index -= 1;
            let move_guess = MoveOrderer::put_highest_guessed_move_on_top(&mut move_guesses, index);
            println!("{} {}", move_guess.move_data, move_guess.guess);
            (move_guess.move_data, index != 0)
        };
        assert!(
            next_move().0.decode()
                == Move {
                    from: Square::from_notation("c4"),
                    to: Square::from_notation("b5"),
                    flag: Flag::None
                }
        );
        assert!(
            next_move().0.decode()
                == Move {
                    from: Square::from_notation("a7"),
                    to: Square::from_notation("a8"),
                    flag: Flag::QueenPromotion
                }
        );
        while next_move().1 {}
    }
}
