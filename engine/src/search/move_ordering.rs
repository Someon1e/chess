use crate::{
    board::bit_board::BitBoard,
    move_generator::{
        move_data::{Flag, Move},
        MoveGenerator,
    },
};

use super::{encoded_move::EncodedMove, eval::Eval, Search};

#[derive(Clone, Copy)]
pub struct MoveGuess {
    guess: i32,
    pub move_data: EncodedMove,
}

const MAX_LEGAL_MOVES: usize = 218;
const MAX_CAPTURES: usize = 74;

/*
    PV-move of the principal variation from the previous iteration of an iterative deepening framework for the leftmost path, often implicitly done by 2.
    Hash move from hash tables
    Winning captures/promotions
    Equal captures/promotions
    Killer moves (non capture), often with mate killers first
    Non-captures sorted by history heuristic and that like
    Losing captures (* but see below
*/
const HASH_MOVE_BONUS: i32 = 100000;
const WINNING_CAPTURE_BONUS: i32 = 2000;
const PROMOTION_BONUS: i32 = 2000;
const EQUAL_CAPTURE_BONUS: i32 = 1000;
const KILLER_MOVE_BONUS: i32 = 500;
const LOSING_CAPTURE_BONUS: i32 = 50;

pub struct MoveOrderer {}
impl MoveOrderer {
    fn guess_move_value(search: &Search, enemy_pawn_attacks: &BitBoard, move_data: &Move) -> i32 {
        let moving_from = move_data.from;
        let moving_to = move_data.to;

        let mut score = 0;
        match move_data.flag {
            Flag::EnPassant => {
                return search.history_heuristic[search.board.white_to_move as usize]
                    [moving_from.usize()][moving_to.usize()] as i32
            }
            Flag::PawnTwoUp => {}
            Flag::Castle => {
                return search.history_heuristic[search.board.white_to_move as usize]
                    [moving_from.usize()][moving_to.usize()] as i32
            }

            Flag::BishopPromotion => return PROMOTION_BONUS + 300,
            Flag::KnightPromotion => return PROMOTION_BONUS + 400,
            Flag::RookPromotion => return PROMOTION_BONUS + 300,
            Flag::QueenPromotion => return PROMOTION_BONUS + 800,

            Flag::None => {}
        }

        let moving_piece = search.board.friendly_piece_at(moving_from).unwrap();

        // This won't take into account en passant
        if let Some(capturing) = search.board.enemy_piece_at(moving_to) {
            let (moving_from_middle_game_value, moving_from_end_game_value) = {
                if search.board.white_to_move {
                    Eval::get_white_piece_value(moving_piece, moving_from)
                } else {
                    Eval::get_black_piece_value(moving_piece, moving_from)
                }
            };

            let phase = Eval::get_phase(search);

            let (capturing_middle_game_value, capturing_end_game_value) = {
                if search.board.white_to_move {
                    Eval::get_black_piece_value(capturing, moving_to)
                } else {
                    Eval::get_white_piece_value(capturing, moving_to)
                }
            };
            let mut potential_middle_game_value_loss = moving_from_middle_game_value;
            let mut potential_end_game_value_loss = moving_from_end_game_value;
            if !enemy_pawn_attacks.get(&moving_to) {
                potential_middle_game_value_loss /= 2;
                potential_end_game_value_loss /= 2;
            }
            let score_difference = Eval::calculate_score(
                phase,
                capturing_middle_game_value - potential_middle_game_value_loss,
                capturing_end_game_value - potential_end_game_value_loss,
            );
            score += score_difference;
            if score_difference.is_positive() {
                score += WINNING_CAPTURE_BONUS
            } else if score_difference.is_negative() {
                score += LOSING_CAPTURE_BONUS;
            } else {
                score += EQUAL_CAPTURE_BONUS;
            }
        } else {
            score += search.history_heuristic[search.board.white_to_move as usize]
                [moving_from.usize()][moving_to.usize()] as i32;

            if enemy_pawn_attacks.get(&moving_to) {
                score -= 50;
            }
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

    fn guess_capture_value(search: &Search, move_data: &Move) -> i32 {
        let flag_score = match move_data.flag {
            Flag::EnPassant => return 0,

            Flag::BishopPromotion => 400,
            Flag::KnightPromotion => 500,
            Flag::RookPromotion => 400,
            Flag::QueenPromotion => 900,

            Flag::None => 0,

            _ => unreachable!(),
        };

        let moving_from = move_data.from;
        let moving_to = move_data.to;

        let moving_piece = search.board.friendly_piece_at(moving_from).unwrap();
        let (moving_from_middle_game_value, moving_from_end_game_value) = {
            if search.board.white_to_move {
                Eval::get_white_piece_value(moving_piece, moving_from)
            } else {
                Eval::get_black_piece_value(moving_piece, moving_from)
            }
        };

        let capturing = search.board.enemy_piece_at(moving_to).unwrap();

        let (capturing_middle_game_value, capturing_end_game_value) = {
            if search.board.white_to_move {
                Eval::get_black_piece_value(capturing, moving_to)
            } else {
                Eval::get_white_piece_value(capturing, moving_to)
            }
        };

        flag_score
            + Eval::calculate_score(
                Eval::get_phase(search),
                capturing_middle_game_value - moving_from_middle_game_value,
                capturing_end_game_value - moving_from_end_game_value,
            )
    }

    pub fn get_sorted_moves_captures_only(
        search: &Search,
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
                    guess: Self::guess_capture_value(search, &move_data),
                };
                index += 1
            },
            true,
        );

        (move_guesses, index)
    }

    pub fn get_sorted_moves(
        search: &Search,
        move_generator: &MoveGenerator,
        hash_move: EncodedMove,
        killer_move: EncodedMove,
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
                    guess: if encoded == hash_move {
                        HASH_MOVE_BONUS
                    } else {
                        Self::guess_move_value(
                            search,
                            &move_generator.enemy_pawn_attacks(),
                            &move_data,
                        )
                    } + if encoded == killer_move {
                        KILLER_MOVE_BONUS
                    } else {
                        0
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
        move_generator::{
            move_data::{Flag, Move},
            MoveGenerator,
        },
        search::{encoded_move::EncodedMove, move_ordering::MoveOrderer, Search},
    };

    #[test]
    fn move_ordering_works() {
        let mut board = Board::from_fen("8/P6p/6r1/1q1n4/2P3R1/8/2K2k2/8 w - - 0 1");
        let move_generator = MoveGenerator::new(&board);
        let (mut move_guesses, move_count) = MoveOrderer::get_sorted_moves(
            &Search::new(&mut board),
            &move_generator,
            EncodedMove::NONE,
            EncodedMove::NONE,
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
